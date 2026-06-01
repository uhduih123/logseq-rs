//! Markdown 文件解析器
//!
//! 使用 pulldown-cmark 解析 Markdown 语法，提取为 Logseq Block 树结构。
//! 处理：
//! - 标题层级 -> 页面/块层级
//! - 嵌套列表 -> 父子块关系  
//! - YAML frontmatter -> 块属性
//! - 行内特殊语法（wiki 链接、块引用、标签等）

use crate::ParseContext;
use logseq_core::model::{Block, BlockFormat, BlockProperties};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use uuid::Uuid;

/// 解析整段 Markdown 文本为块列表
pub fn parse_markdown(text: &str, page_id: Option<Uuid>) -> Vec<Block> {
    let mut ctx = ParseContext::new(page_id, BlockFormat::Markdown);
    let mut current_text = String::new();
    let mut current_meta = BlockMeta::default();
    let _in_frontmatter = false;
    let _frontmatter_yaml = String::new();
    let mut first_block = true;

    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES;

    let parser = Parser::new_ext(text, options);

    for event in parser {
        match event {
            Event::Start(tag) => {
                // 先提交之前的文本块
                if !current_text.trim().is_empty() {
                    let (block, _) = finish_block(
                        &current_text,
                        &current_meta,
                        &mut ctx,
                        first_block,
                    );
                    ctx.blocks.push(block);
                    first_block = false;
                    current_text.clear();
                }

                match tag {
                    Tag::Heading { level, .. } => {
                        current_meta.heading_level = Some(match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        });
                    }
                    Tag::List(ord) => {
                        current_meta.list_depth += 1;
                        current_meta.ordered = ord.is_some();
                    }
                    Tag::Item => {
                        current_meta.in_list_item = true;
                    }
                    Tag::CodeBlock(kind) => {
                        current_meta.in_code_block = true;
                        use pulldown_cmark::CodeBlockKind;
                        current_meta.code_lang = match kind {
                            CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                            CodeBlockKind::Indented => None,
                        };
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                match tag {
                    TagEnd::Heading(_) => {
                        // 标题结束后立即提交
                        if !current_text.trim().is_empty() {
                            let (block, _) = finish_block(
                                &current_text,
                                &current_meta,
                                &mut ctx,
                                first_block,
                            );
                            ctx.blocks.push(block);
                            first_block = false;
                            current_text.clear();
                        }
                        current_meta = BlockMeta::default();
                    }
                    TagEnd::List(_) => {
                        current_meta.list_depth = current_meta.list_depth.saturating_sub(1);
                    }
                    TagEnd::Item => {
                        if !current_text.trim().is_empty() {
                            let (block, _) = finish_block(
                                &current_text,
                                &current_meta,
                                &mut ctx,
                                first_block,
                            );
                            ctx.blocks.push(block);
                            first_block = false;
                            current_text.clear();
                        }
                        current_meta.in_list_item = false;
                    }
                    TagEnd::CodeBlock => {
                        if !current_text.is_empty() {
                            // 代码块内容作为 body
                            if let Some(last) = ctx.blocks.last_mut() {
                                last.body = current_text.clone();
                            }
                        }
                        current_meta.in_code_block = false;
                        current_meta.code_lang = None;
                        current_text.clear();
                    }
                    _ => {}
                }
            }
            Event::Text(text) | Event::Code(text) => {
                current_text.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak => {
                current_text.push('\n');
            }
            _ => {}
        }
    }

    // 提交最后的文本块
    if !current_text.trim().is_empty() {
        let (block, _) = finish_block(&current_text, &current_meta, &mut ctx, first_block);
        ctx.blocks.push(block);
    }

    // 第二遍：建立父子关系
    build_tree(&mut ctx.blocks);

    ctx.blocks
}

/// 块元数据（解析过程中的临时状态）
#[derive(Debug, Clone, Default)]
struct BlockMeta {
    heading_level: Option<usize>,
    list_depth: usize,
    ordered: bool,
    in_list_item: bool,
    in_code_block: bool,
    code_lang: Option<String>,
}

/// 根据当前文本和元数据构造一个 Block
fn finish_block(
    text: &str,
    meta: &BlockMeta,
    ctx: &mut ParseContext,
    _is_page_title: bool,
) -> (Block, usize) {
    let inline = ParseContext::parse_inline(text);

    let level = meta.heading_level.unwrap_or(meta.list_depth);

    let block = Block {
        uuid: Uuid::new_v4(),
        title: inline.clean_text.clone(),
        level,
        page: ctx.page_id,
        tags: inline.tags,
        format: ctx.format,
        marker: inline.marker,
        priority: inline.priority,
        scheduled: inline.scheduled,
        deadline: inline.deadline,
        ..Default::default()
    };

    // 记录 UUID
    ctx.block_index.push(block.uuid);

    (block, level)
}

/// 根据层级关系建立树的父子引用
fn build_tree(blocks: &mut [Block]) {
    if blocks.is_empty() {
        return;
    }

    // 用栈跟踪每级的最近父块
    // (level, block_idx)
    let mut level_stack: Vec<(usize, usize)> = Vec::new();

    for i in 0..blocks.len() {
        let current_level = blocks[i].level;

        // 弹出所有不小于当前 level 的项
        while let Some(&(stack_level, _)) = level_stack.last() {
            if stack_level >= current_level {
                level_stack.pop();
            } else {
                break;
            }
        }

        // 设置父块和左兄弟
        if let Some(&(_, parent_idx)) = level_stack.last() {
            let parent_uuid = blocks[parent_idx].uuid;
            blocks[i].parent = Some(parent_uuid);

            // 设置 left（同一父块的前一个同级块）
            for j in (0..i).rev() {
                if blocks[j].parent == Some(parent_uuid) && blocks[j].level == current_level {
                    blocks[i].left = Some(blocks[j].uuid);
                    break;
                }
            }
        }

        level_stack.push((current_level, i));
    }
}

/// 解析 YAML frontmatter 为块属性
pub fn parse_frontmatter(yaml: &str) -> BlockProperties {
    let mut props = BlockProperties::default();

    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            props.order.push(key.clone());
            props.map.insert(key, value);
        }
    }

    props
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown() {
        let md = r#"# Hello World
- Item 1
- Item 2
  - Sub item
"#;
        let blocks = parse_markdown(md, None);
        assert!(!blocks.is_empty());

        // 第一个块是 H1
        let title = &blocks[0];
        assert_eq!(title.level, 1);
        assert_eq!(title.title, "Hello World");

        // Item 1
        let item1 = &blocks[1];
        assert_eq!(item1.title, "Item 1");
        assert_eq!(item1.level, 1);
    }

    #[test]
    fn test_parse_with_todo() {
        let md = "- TODO Fix the [[bug]] #urgent\n- DONE [#A] Deploy";
        let blocks = parse_markdown(md, None);
        assert_eq!(blocks.len(), 2);

        let first = &blocks[0];
        assert_eq!(first.marker, Some(logseq_core::model::BlockMarker::Todo));
        assert_eq!(first.tags, vec!["urgent"]);

        let second = &blocks[1];
        assert_eq!(second.marker, Some(logseq_core::model::BlockMarker::Done));
    }

    #[test]
    fn test_tree_structure() {
        let md = r#"# Page
- Parent
  - Child
  - Child 2
- Another Parent
"#;
        let blocks = parse_markdown(md, None);

        // Parent (index 1) 应该是 page 下第一个一级块
        let parent = &blocks[1];
        assert_eq!(parent.level, 1);
        assert_eq!(parent.title, "Parent");

        // Child (index 2) 的 parent 应该是 Parent
        let child = &blocks[2];
        assert_eq!(child.level, 2);
        assert_eq!(child.parent, Some(parent.uuid));
    }
}
