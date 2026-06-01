//! Graph 管线：文件系统 → 解析 → 数据库
//!
//! 核心集成层：
//! 1. 扫描 graph 目录中的 .md / .org 文件
//! 2. 调用 parser 解析为 Block/Page
//! 3. 存入 SQLite 数据库
//! 4. 监听文件变更自动更新

use crate::Database;
use logseq_core::model::{BlockProperties, GraphConfig, Page};
use logseq_parser::markdown;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 打开一个 graph：扫描所有文件，解析并入库
pub fn open_graph(db: &Database, root_path: &Path) -> Result<GraphConfig, String> {
    let config = GraphConfig {
        path: root_path.to_string_lossy().to_string(),
        ..Default::default()
    };

    // 扫描所有 .md 和 .org 文件
    let mut pages_processed = 0;
    let mut blocks_processed = 0;

    for entry in walkdir::WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            e.file_type().is_file()
                && (name.ends_with(".md") || name.ends_with(".org"))
        })
    {
        let file_path = entry.path();
        let file_name = file_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // 读取文件内容
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // 判断是否为 journal（文件名是日期格式 YYYY_MM_DD）
        let is_journal = is_journal_name(&file_name);
        let journal_day = if is_journal {
            parse_journal_day(&file_name)
        } else {
            None
        };

        // 创建或更新 Page
        let page_uuid = Uuid::new_v4();
        let now = chrono::Utc::now();

        let page = Page {
            id: page_uuid,
            name: file_name.clone(),
            title: None, // 解析后会从第一个 H1 设置
            is_journal,
            journal_day,
            namespace: None,
            properties: BlockProperties::default(),
            created_at: now,
            updated_at: now,
        };

        // 解析内容为块
        let mut blocks = markdown::parse_markdown(&content, Some(page_uuid));

        // 第一个 H1 块作为页面标题
        let page_title = blocks
            .iter()
            .find(|b| b.level == 1)
            .map(|b| b.title.clone());

        // 获取 namespace（文件路径相对于 root 的目录部分）
        let namespace = file_path
            .parent()
            .and_then(|p| p.strip_prefix(root_path).ok())
            .and_then(|p| {
                let s = p.to_string_lossy().to_string();
                if s.is_empty() { None } else { Some(s) }
            });

        // 更新并写入 Page
        let page = Page {
            title: page_title,
            namespace,
            ..page
        };
        db.upsert_page(&page).map_err(|e| e.to_string())?;

        // 写入所有 Block
        for block in &mut blocks {
            block.page = Some(page_uuid);
            db.insert_block(block).map_err(|e| e.to_string())?;

            // 重建引用（[[wikilinks]] 和 ((block-refs))）
            db.rebuild_refs_for_block(block.uuid, &block.title)
                .map_err(|e| e.to_string())?;
        }

        pages_processed += 1;
        blocks_processed += blocks.len();
    }

    tracing::info!(
        "Graph opened: {} pages, {} blocks",
        pages_processed,
        blocks_processed
    );

    Ok(config)
}

/// 监听文件变更，自动重新解析
pub fn watch_graph(
    db: Arc<Mutex<Database>>,
    root_path: PathBuf,
) -> Result<notify::RecommendedWatcher, String> {
    let db_clone = Arc::clone(&db);

    let mut watcher = notify::recommended_watcher(move |event: Result<Event, notify::Error>| {
        if let Ok(event) = event {
            match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) => {
                    for path in &event.paths {
                        let ext = path.extension().map(|e| e.to_string_lossy().to_string());
                        if matches!(ext.as_deref(), Some("md") | Some("org")) {
                            if let Ok(db) = db_clone.lock() {
                                handle_file_change(&db, path);
                            }
                        }
                    }
                }
                EventKind::Remove(_) => {
                    // TODO: 处理文件删除
                }
                _ => {}
            }
        }
    })
    .map_err(|e| e.to_string())?;

    watcher
        .watch(&root_path, RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    Ok(watcher)
}

/// 处理单个文件变更
fn handle_file_change(db: &Database, path: &Path) {
    let file_name = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // 查找或创建 Page
    let existing_pages = match db.get_all_pages() {
        Ok(p) => p,
        Err(_) => return,
    };

    let page = existing_pages.into_iter().find(|p| p.name == file_name);

    let page_uuid = match page {
        Some(p) => {
            // 删除该页面的所有旧块
            let _ = db.delete_page(p.id);
            p.id
        }
        None => {
            let uuid = Uuid::new_v4();
            let now = chrono::Utc::now();
            let new_page = Page {
                id: uuid,
                name: file_name,
                title: None,
                is_journal: false,
                journal_day: None,
                namespace: None,
                properties: BlockProperties::default(),
                created_at: now,
                updated_at: now,
            };
            let _ = db.upsert_page(&new_page);
            uuid
        }
    };

    // 重新解析
    let blocks = markdown::parse_markdown(&content, Some(page_uuid));

    let page_title = blocks
        .iter()
        .find(|b| b.level == 1)
        .map(|b| b.title.clone());

    let now = chrono::Utc::now();
    let updated_page = Page {
        id: page_uuid,
        title: page_title,
        updated_at: now,
        ..Default::default()
    };
    let _ = db.upsert_page(&updated_page);

    for block in &blocks {
        let _ = db.insert_block(block);
        let _ = db.rebuild_refs_for_block(block.uuid, &block.title);
    }
}

// ── Journal 辅助函数 ──

/// 判断文件名是否为 journal 日期格式
fn is_journal_name(name: &str) -> bool {
    // Logseq journal 格式: YYYY_MM_DD
    let parts: Vec<&str> = name.split('_').collect();
    if parts.len() != 3 {
        return false;
    }
    parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

/// 解析 journal 日期为 YYYYMMDD 整数
fn parse_journal_day(name: &str) -> Option<i64> {
    let digits: String = name.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 8 {
        digits.parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_open_graph_with_markdown_files() {
        // 创建临时目录
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // 创建测试 .md 文件
        let page1 = root.join("test_page.md");
        std::fs::write(
            &page1,
            "# Hello World\n- Item 1\n- Item 2 [[other page]]\n  - Sub item #tag",
        )
        .unwrap();

        // 创建 journal 文件
        let journal = root.join("2026_06_01.md");
        std::fs::write(
            &journal,
            "- TODO Morning standup\n- DONE Write code",
        )
        .unwrap();

        // 打开 graph
        let db = Database::in_memory().unwrap();
        let config = open_graph(&db, root).unwrap();

        assert_eq!(config.path, root.to_string_lossy());

        // 验证页面
        let pages = db.get_all_pages().unwrap();
        assert_eq!(pages.len(), 2);

        let test_page = pages.iter().find(|p| p.name == "test_page").unwrap();
        assert!(!test_page.is_journal);
        assert_eq!(test_page.title.as_deref(), Some("Hello World"));

        let journal_page = pages.iter().find(|p| p.name == "2026_06_01").unwrap();
        assert!(journal_page.is_journal);
        assert_eq!(journal_page.journal_day, Some(20260601));

        // 验证块
        let blocks = db.get_page_blocks(test_page.id).unwrap();
        assert!(blocks.len() >= 3);

        // 验证标签
        let has_tag = blocks.iter().any(|b| b.tags.contains(&"tag".to_string()));
        assert!(has_tag);
    }

    #[test]
    fn test_is_journal_name() {
        assert!(is_journal_name("2026_06_01"));
        assert!(is_journal_name("2024_12_31"));
        assert!(!is_journal_name("test_page"));
        assert!(!is_journal_name("2026-06-01"));
        assert!(!is_journal_name(""));
    }

    #[test]
    fn test_parse_journal_day() {
        assert_eq!(parse_journal_day("2026_06_01"), Some(20260601));
        assert_eq!(parse_journal_day("2024_12_31"), Some(20241231));
        assert_eq!(parse_journal_day("not_a_date"), None);
    }
}
