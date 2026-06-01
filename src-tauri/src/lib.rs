//! Logseq-RS Tauri 后端入口
//!
//! 注册所有 Tauri IPC 命令，桥接前端调用到 Rust crates。

use logseq_core::model::{Block, GraphConfig, Page};
use logseq_db::Database;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

/// 应用状态
struct AppState {
    db: Mutex<Option<Database>>,
    graph_path: Mutex<Option<String>>,
}

// ── Graph 操作 ──

#[tauri::command]
fn open_graph(path: String, state: State<AppState>) -> Result<GraphConfig, String> {
    let db_path = PathBuf::from(&path).join("logseq-rs.db");
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;

    let config = GraphConfig {
        path: path.clone(),
        ..Default::default()
    };

    *state.db.lock().unwrap() = Some(db);
    *state.graph_path.lock().unwrap() = Some(path);

    Ok(config)
}

#[tauri::command]
fn list_pages(state: State<AppState>) -> Result<Vec<Page>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;
    db.get_all_pages().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_page_blocks(page_id: String, state: State<AppState>) -> Result<Vec<Block>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;
    let pid = Uuid::parse_str(&page_id).map_err(|e| e.to_string())?;
    db.get_page_blocks(pid).map_err(|e| e.to_string())
}

// ── 块操作 ──

#[tauri::command]
fn insert_block(
    title: String,
    page_id: String,
    parent_id: Option<String>,
    left_id: Option<String>,
    level: usize,
    state: State<AppState>,
) -> Result<Block, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;

    let page_uuid = Uuid::parse_str(&page_id).map_err(|e| e.to_string())?;
    let parent_uuid = parent_id.and_then(|s| Uuid::parse_str(&s).ok());
    let left_uuid = left_id.and_then(|s| Uuid::parse_str(&s).ok());

    let block = Block {
        uuid: Uuid::new_v4(),
        title,
        level,
        page: Some(page_uuid),
        parent: parent_uuid,
        left: left_uuid,
        ..Default::default()
    };

    db.insert_block(&block).map_err(|e| e.to_string())?;
    Ok(block)
}

#[tauri::command]
fn update_block(id: String, title: String, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;
    let uid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    db.update_block_title(uid, &title).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_block(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;
    let uid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    db.delete_block(uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn indent_block(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;
    let uid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    db.indent_block(uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn outdent_block(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;
    let uid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    db.outdent_block(uid).map_err(|e| e.to_string())
}

#[tauri::command]
fn move_block(
    id: String,
    new_parent_id: Option<String>,
    new_left_id: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;
    let uid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let new_parent = new_parent_id.and_then(|s| Uuid::parse_str(&s).ok());
    let new_left = new_left_id.and_then(|s| Uuid::parse_str(&s).ok());
    db.move_block(uid, new_parent, new_left).map_err(|e| e.to_string())
}

// ── 页面操作 ──

#[tauri::command]
fn create_page(name: String, state: State<AppState>) -> Result<Page, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;

    let now = chrono::Utc::now();
    let page = Page {
        id: Uuid::new_v4(),
        name: name.clone(),
        title: Some(name),
        is_journal: false,
        journal_day: None,
        namespace: None,
        properties: Default::default(),
        created_at: now,
        updated_at: now,
    };

    db.upsert_page(&page).map_err(|e| e.to_string())?;
    Ok(page)
}

#[tauri::command]
fn delete_page(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;
    let uid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    db.delete_page(uid).map_err(|e| e.to_string())
}

// ── 搜索 ──

#[tauri::command]
fn search_blocks(query: String, state: State<AppState>) -> Result<Vec<Block>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;

    let filter = logseq_db::query::QueryFilter {
        keyword: Some(query),
        ..Default::default()
    };

    db.query_blocks(filter).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_backlinks(block_id: String, state: State<AppState>) -> Result<Vec<Block>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No graph opened")?;

    let uid = Uuid::parse_str(&block_id).map_err(|e| e.to_string())?;
    let backlink_ids = db.get_backlinks(uid).map_err(|e| e.to_string())?;

    let mut blocks = Vec::new();
    for id in backlink_ids {
        if let Ok(Some(block)) = db.get_block(id) {
            blocks.push(block);
        }
    }
    Ok(blocks)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db: Mutex::new(None),
            graph_path: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            open_graph,
            list_pages,
            get_page_blocks,
            insert_block,
            update_block,
            delete_block,
            indent_block,
            outdent_block,
            move_block,
            create_page,
            delete_page,
            search_blocks,
            get_backlinks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Logseq-RS");
}
