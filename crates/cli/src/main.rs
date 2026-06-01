//! Logseq-RS CLI — 命令行工具
//!
//! 无需 GUI 即可验证核心功能：
//!   logseq open <dir>      打开 graph 目录
//!   logseq list            列出所有页面
//!   logseq show <page>     显示页面内容
//!   logseq search <query>  搜索块
//!   logseq parse <file>    解析单个文件

use logseq_db::{pipeline, query::QueryFilter, Database};
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "open" => {
            if args.len() < 3 {
                eprintln!("用法: logseq open <目录路径>");
                return;
            }
            cmd_open(&args[2]);
        }
        "list" => cmd_list(),
        "show" => {
            if args.len() < 3 {
                eprintln!("用法: logseq show <页面名>");
                return;
            }
            cmd_show(&args[2]);
        }
        "search" => {
            if args.len() < 3 {
                eprintln!("用法: logseq search <关键词>");
                return;
            }
            cmd_search(&args[2]);
        }
        "parse" => {
            if args.len() < 3 {
                eprintln!("用法: logseq parse <文件路径>");
                return;
            }
            cmd_parse(&args[2]);
        }
        "test" => cmd_test(),
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Logseq-RS CLI v0.1.0");
    println!("用法:");
    println!("  logseq open <dir>       打开 graph 目录");
    println!("  logseq list             列出所有页面");
    println!("  logseq show <页面名>    显示页面块");
    println!("  logseq search <关键词>  搜索块");
    println!("  logseq parse <文件>     解析单个 markdown 文件");
    println!("  logseq test             运行自检");
}

fn cmd_open(path: &str) {
    let db_path = std::path::Path::new(path).join(".logseq-rs.db");
    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(_) => {
            eprintln!("打开数据库失败");
            return;
        }
    };

    println!("📂 扫描目录: {}", path);
    match pipeline::open_graph(&db, std::path::Path::new(path)) {
        Ok(config) => {
            let pages = db.get_all_pages().unwrap_or_default();
            let block_count: usize = pages
                .iter()
                .map(|p| db.get_page_blocks(p.id).unwrap_or_default().len())
                .sum();
            println!("✅ 完成: {} 页面, {} 块", pages.len(), block_count);
            println!("   路径: {}", config.path);
        }
        Err(e) => eprintln!("❌ 打开失败: {}", e),
    }
}

fn cmd_list() {
    let cwd = env::current_dir().unwrap_or_default();
    let db_path = cwd.join(".logseq-rs.db");

    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(_) => {
            eprintln!("找不到数据库，请先运行: logseq open <目录>");
            return;
        }
    };

    let pages = db.get_all_pages().unwrap_or_default();
    println!("📄 {} 个页面:\n", pages.len());

    for page in &pages {
        let icon = if page.is_journal { "📅" } else { "📝" };
        let blocks = db.get_page_blocks(page.id).unwrap_or_default();
        println!("  {} {}  ({} 块)", icon, page.name, blocks.len());
        if let Some(ref title) = page.title {
            println!("     {} {}", title, if blocks.len() > 0 { "..." } else { "" });
        }
    }
}

fn cmd_show(name: &str) {
    let cwd = env::current_dir().unwrap_or_default();
    let db_path = cwd.join(".logseq-rs.db");

    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(_) => {
            eprintln!("找不到数据库");
            return;
        }
    };

    let pages = db.get_all_pages().unwrap_or_default();
    let page = pages.iter().find(|p| p.name == name);

    match page {
        Some(page) => {
            println!("📄 {}\n", page.title.as_deref().unwrap_or(&page.name));
            let blocks = db.get_page_blocks(page.id).unwrap_or_default();
            for block in &blocks {
                let indent = "  ".repeat(block.level);
                let marker = block.marker.as_ref().map(|m| format!("{} ", m)).unwrap_or_default();
                let priority = block.priority.as_ref().map(|p| format!("[#{}] ", p)).unwrap_or_default();
                println!("{}{}{}{}", indent, marker, priority, block.title);

                // 显示标签
                if !block.tags.is_empty() {
                    let tags: Vec<String> = block.tags.iter().map(|t| format!("#{}", t)).collect();
                    println!("{}  {}", indent, tags.join(" "));
                }
            }
        }
        None => println!("页面 '{}' 不存在", name),
    }
}

fn cmd_search(query: &str) {
    let cwd = env::current_dir().unwrap_or_default();
    let db_path = cwd.join(".logseq-rs.db");

    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(_) => {
            eprintln!("找不到数据库");
            return;
        }
    };

    let filter = QueryFilter {
        keyword: Some(query.to_string()),
        limit: Some(20),
        ..Default::default()
    };

    match db.query_blocks(filter) {
        Ok(blocks) => {
            println!("🔍 搜索 '{}' — {} 个结果:\n", query, blocks.len());
            for block in &blocks {
                let pages = db.get_all_pages().unwrap_or_default();
                let page_name = block.page
                    .and_then(|pid| pages.iter().find(|p| p.id == pid))
                    .map(|p| p.name.as_str())
                    .unwrap_or("?");
                println!("  [{}] {}", page_name, block.title);
            }
        }
        Err(e) => eprintln!("搜索失败: {}", e),
    }
}

fn cmd_parse(file: &str) {
    let path = std::path::Path::new(file);
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("读取文件失败");
            return;
        }
    };

    let blocks = logseq_parser::markdown::parse_markdown(&content, None);
    println!("📄 解析 {} — {} 个块:\n", file, blocks.len());

    for block in &blocks {
        let indent = "  ".repeat(block.level);
        let marker = block.marker.as_ref().map(|m| format!("{} ", m)).unwrap_or_default();
        println!("{}{}{}", indent, marker, block.title);
        if !block.tags.is_empty() {
            println!("{}  #{}", indent, block.tags.join(" #"));
        }
    }
}

fn cmd_test() {
    println!("🧪 Logseq-RS 自检\n");

    // 测试解析器
    let md = "# Hello\n- TODO Item 1 #test\n- Item 2 [[link]]";
    let blocks = logseq_parser::markdown::parse_markdown(md, None);
    println!("✅ 解析器: {} 个块", blocks.len());
    assert_eq!(blocks.len(), 3);

    // 测试数据库
    let db = Database::in_memory().unwrap();
    let page = logseq_core::model::Page {
        id: uuid::Uuid::new_v4(),
        name: "test".to_string(),
        ..Default::default()
    };
    db.upsert_page(&page).unwrap();
    let pages = db.get_all_pages().unwrap();
    assert_eq!(pages.len(), 1);
    println!("✅ 数据库: 读写正常");

    // 测试搜索
    let results = logseq_search::search_blocks(&blocks, "item");
    assert_eq!(results.len(), 2);
    println!("✅ 搜索: {} 个结果", results.len());

    println!("\n🎉 全部通过！");
}
