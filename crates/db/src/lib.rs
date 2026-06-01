//! Logseq 数据库层
//!
//! SQLite 持久化存储，提供块和页面的 CRUD 操作。
//! 替代 Logseq 现有的 Datascript（内存）+ SQLite（WASM）双层架构。

pub mod schema;
pub mod blocks;
pub mod pages;
pub mod links;
pub mod query;
pub mod pipeline;
mod tests;

use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;
use uuid::Uuid;

/// 数据库句柄
pub struct Database {
    conn: Connection,
}

impl Database {
    /// 打开或创建数据库
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// 在内存中创建数据库（用于测试）
    pub fn in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// 初始化 Schema
    fn init_schema(&self) -> SqlResult<()> {
        schema::create_tables(&self.conn)
    }

    // ── 结构修改操作 ──

    /// 缩进块（增加 level，设 parent 为前一同级块）
    pub fn indent_block(&self, block_id: Uuid) -> SqlResult<()> {
        let block = self.get_block(block_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        if let Some(left_id) = block.left {
            let new_level = block.level + 1;
            self.conn.execute(
                "UPDATE blocks SET level = ?1, parent_id = ?2, updated_at = ?3 WHERE id = ?4",
                params![new_level, left_id.to_string(), chrono::Utc::now().to_rfc3339(), block_id.to_string()],
            )?;
        }
        Ok(())
    }

    /// 反缩进块（减少 level，设 parent 为祖父块）
    pub fn outdent_block(&self, block_id: Uuid) -> SqlResult<()> {
        let block = self.get_block(block_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        if let Some(parent_id) = block.parent {
            if let Ok(Some(parent)) = self.get_block(parent_id) {
                let grandparent = parent.parent;
                let new_level = block.level.saturating_sub(1);
                self.conn.execute(
                    "UPDATE blocks SET level = ?1, parent_id = ?2, updated_at = ?3 WHERE id = ?4",
                    params![new_level, grandparent.map(|u| u.to_string()), chrono::Utc::now().to_rfc3339(), block_id.to_string()],
                )?;
            }
        }
        Ok(())
    }

    /// 移动块到新位置
    pub fn move_block(&self, block_id: Uuid, new_parent_id: Option<Uuid>, new_left_id: Option<Uuid>) -> SqlResult<()> {
        let new_level = if let Some(pid) = new_parent_id {
            self.get_block(pid)?.map(|b| b.level + 1).unwrap_or(0)
        } else {
            0
        };
        self.conn.execute(
            "UPDATE blocks SET parent_id = ?1, left_id = ?2, level = ?3, updated_at = ?4 WHERE id = ?5",
            params![new_parent_id.map(|u| u.to_string()), new_left_id.map(|u| u.to_string()), new_level, chrono::Utc::now().to_rfc3339(), block_id.to_string()],
        )?;
        Ok(())
    }
}
