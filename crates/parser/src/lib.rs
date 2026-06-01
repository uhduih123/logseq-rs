//! Logseq Markdown / Org-mode Parser
//!
//! 解析 Markdown/Org 文件为 Logseq 的 Block 模型。
//! 处理 Logseq 特有语法：[[wikilinks]]、((block-refs))、#tags、TODO 标记等。

pub mod markdown;
pub mod org;

use chrono::{DateTime, Utc};
use logseq_core::model::{Block, BlockFormat, BlockMarker, Priority};
use uuid::Uuid;

/// 解析上下文 —— 跟踪解析过程中的状态
#[derive(Debug, Clone, Default)]
pub struct ParseContext {
    /// 当前页面 ID
    pub page_id: Option<Uuid>,
    /// 当前格式
    pub format: BlockFormat,
    /// 已解析的块列表
    pub blocks: Vec<Block>,
    /// 块 ID → 索引映射（用于建立父子关系）
    pub block_index: Vec<Uuid>,
    /// 当前层级栈（用于处理嵌套列表）
    pub level_stack: Vec<usize>,
}

impl ParseContext {
    pub fn new(page_id: Option<Uuid>, format: BlockFormat) -> Self {
        Self {
            page_id,
            format,
            blocks: Vec::new(),
            block_index: Vec::new(),
            level_stack: Vec::new(),
        }
    }

    /// 解析行内文本中的特殊语法
    pub fn parse_inline(text: &str) -> InlineParts {
        let mut parts = InlineParts::default();

        // 提取 [[wikilinks]]
        let re_wiki = regex_lite::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        for cap in re_wiki.captures_iter(text) {
            parts.wikilinks.push(cap[1].to_string());
        }

        // 提取 ((block-references))
        let re_block_ref = regex_lite::Regex::new(r"\(\(([^)]+)\)\)").unwrap();
        for cap in re_block_ref.captures_iter(text) {
            parts.block_refs.push(cap[1].to_string());
        }

        // 提取 #tags（中英文标签）
        let re_tag = regex_lite::Regex::new(r"#([\w\u4e00-\u9fff-]+)").unwrap();
        for cap in re_tag.captures_iter(text) {
            parts.tags.push(cap[1].to_string());
        }

        // 提取任务标记
        let text_upper = text.to_uppercase();
        if text_upper.contains("TODO") {
            parts.marker = Some(BlockMarker::Todo);
        } else if text_upper.contains("LATER") {
            parts.marker = Some(BlockMarker::Later);
        } else if text_upper.contains("DONE") {
            parts.marker = Some(BlockMarker::Done);
        } else if text_upper.contains("NOW") {
            parts.marker = Some(BlockMarker::Now);
        } else if text_upper.contains("WAITING") {
            parts.marker = Some(BlockMarker::Waiting);
        } else if text_upper.contains("CANCELED") {
            parts.marker = Some(BlockMarker::Canceled);
        } else if text_upper.contains("IN-PROGRESS") {
            parts.marker = Some(BlockMarker::InProgress);
        }

        // 提取优先级 [#A] [#B] [#C]
        if text.contains("[#A]") || text.contains("[#A]") {
            parts.priority = Some(Priority::A);
        } else if text.contains("[#B]") {
            parts.priority = Some(Priority::B);
        } else if text.contains("[#C]") {
            parts.priority = Some(Priority::C);
        }

        // 提取 SCHEDULED / DEADLINE
        let re_scheduled = regex_lite::Regex::new(r"SCHEDULED:\s*<([^>]+)>").unwrap();
        if let Some(cap) = re_scheduled.captures(text) {
            parts.scheduled = parse_org_date(&cap[1]);
        }
        let re_deadline = regex_lite::Regex::new(r"DEADLINE:\s*<([^>]+)>").unwrap();
        if let Some(cap) = re_deadline.captures(text) {
            parts.deadline = parse_org_date(&cap[1]);
        }

        // 去除标记后得到纯文本内容
        parts.clean_text = strip_markers(text);

        parts
    }
}

/// 行内解析结果
#[derive(Debug, Clone, Default)]
pub struct InlineParts {
    /// 去除标记后的纯文本
    pub clean_text: String,
    /// wiki 链接目标 [[page]]
    pub wikilinks: Vec<String>,
    /// 块引用 ((block-id))
    pub block_refs: Vec<String>,
    /// 标签 #tag
    pub tags: Vec<String>,
    /// 任务标记
    pub marker: Option<BlockMarker>,
    /// 优先级
    pub priority: Option<Priority>,
    /// 计划日期
    pub scheduled: Option<DateTime<Utc>>,
    /// 截止日期
    pub deadline: Option<DateTime<Utc>>,
}

/// 去掉行内的标记语法，提取纯文本
fn strip_markers(text: &str) -> String {
    let text = regex_lite::Regex::new(r"TODO\s+|LATER\s+|DONE\s+|NOW\s+|WAITING\s+|CANCELED\s+|IN-PROGRESS\s+")
        .unwrap()
        .replace(text, "")
        .to_string();
    let text = regex_lite::Regex::new(r"\[#[ABC]\]\s*")
        .unwrap()
        .replace(&text, "")
        .to_string();
    let text = regex_lite::Regex::new(r"SCHEDULED:\s*<[^>]+>\s*")
        .unwrap()
        .replace(&text, "")
        .to_string();
    let text = regex_lite::Regex::new(r"DEADLINE:\s*<[^>]+>\s*")
        .unwrap()
        .replace(&text, "")
        .to_string();
    text.trim().to_string()
}

/// 解析 Org-mode 日期格式 <YYYY-MM-DD Day>
fn parse_org_date(s: &str) -> Option<DateTime<Utc>> {
    let re = regex_lite::Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
    let cap = re.captures(s)?;
    let year: i32 = cap[1].parse().ok()?;
    let month: u32 = cap[2].parse().ok()?;
    let day: u32 = cap[3].parse().ok()?;
    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inline_basic() {
        let parts = ParseContext::parse_inline("TODO Fix the [[bug]] with [[project/setup]] #urgent");
        assert_eq!(parts.marker, Some(BlockMarker::Todo));
        assert_eq!(parts.wikilinks, vec!["bug", "project/setup"]);
        assert_eq!(parts.tags, vec!["urgent"]);
    }

    #[test]
    fn test_parse_inline_priority() {
        let parts = ParseContext::parse_inline("DONE [#A] Submit report");
        assert_eq!(parts.marker, Some(BlockMarker::Done));
        assert_eq!(parts.priority, Some(Priority::A));
    }

    #[test]
    fn test_strip_markers() {
        let clean = strip_markers("TODO [#A] Fix the bug");
        assert_eq!(clean, "Fix the bug");
    }
}
