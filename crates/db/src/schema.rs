//! 数据库 Schema
//!
//! 定义 SQLite 表结构，精确映射 Logseq 的块/页面/属性/引用模型。

use rusqlite::{Connection, Result};

/// 创建所有表
pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- 页面表
        CREATE TABLE IF NOT EXISTS pages (
            id          TEXT PRIMARY KEY,  -- UUID
            name        TEXT NOT NULL,     -- 文件名
            title       TEXT,              -- H1 标题
            is_journal  INTEGER DEFAULT 0, -- 是否为 journal
            journal_day INTEGER,          -- journal 日期 (YYYYMMDD)
            namespace   TEXT,             -- 命名空间路径
            properties  TEXT DEFAULT '{}', -- JSON: 页面属性
            created_at  TEXT NOT NULL,     -- ISO8601
            updated_at  TEXT NOT NULL      -- ISO8601
        );

        -- 页面名称唯一索引
        CREATE UNIQUE INDEX IF NOT EXISTS idx_pages_name ON pages(name);

        -- 块表
        CREATE TABLE IF NOT EXISTS blocks (
            id          TEXT PRIMARY KEY,  -- UUID
            title       TEXT NOT NULL,     -- 块文本内容
            body        TEXT DEFAULT '',   -- 块体内容
            level       INTEGER DEFAULT 0, -- 层级
            left_id     TEXT,              -- 左兄弟 UUID
            parent_id   TEXT,              -- 父块 UUID
            page_id     TEXT,              -- 所属页面 UUID
            format      TEXT DEFAULT 'markdown', -- 'markdown' | 'org'
            marker      TEXT,              -- 'TODO'|'DONE'|'LATER'|NULL
            priority    TEXT,              -- 'A'|'B'|'C'|NULL
            scheduled   TEXT,              -- ISO8601 timestamp
            deadline    TEXT,              -- ISO8601 timestamp
            collapsed   INTEGER DEFAULT 0, -- 是否折叠
            container   INTEGER DEFAULT 0, -- 是否容器块
            pre_block   INTEGER DEFAULT 0, -- 是否前置元数据块
            properties  TEXT DEFAULT '{}',  -- JSON: 块属性
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL,
            FOREIGN KEY (page_id) REFERENCES pages(id)
        );

        -- 块索引
        CREATE INDEX IF NOT EXISTS idx_blocks_page ON blocks(page_id);
        CREATE INDEX IF NOT EXISTS idx_blocks_parent ON blocks(parent_id);
        CREATE INDEX IF NOT EXISTS idx_blocks_left ON blocks(left_id);

        -- 标签表（多对多：block <-> tag）
        CREATE TABLE IF NOT EXISTS tags (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            block_id  TEXT NOT NULL,
            tag       TEXT NOT NULL,
            FOREIGN KEY (block_id) REFERENCES blocks(id)
        );
        CREATE INDEX IF NOT EXISTS idx_tags_block ON tags(block_id);
        CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);

        -- 引用表（多对多：block -> ref-target）
        CREATE TABLE IF NOT EXISTS refs (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id TEXT NOT NULL,  -- 引用源块 UUID
            target_id TEXT,           -- 目标块 UUID（块引用）
            target_page TEXT,         -- 目标页面名（[[wikilink]]）
            FOREIGN KEY (source_id) REFERENCES blocks(id)
        );
        CREATE INDEX IF NOT EXISTS idx_refs_source ON refs(source_id);
        CREATE INDEX IF NOT EXISTS idx_refs_target ON refs(target_id);
        CREATE INDEX IF NOT EXISTS idx_refs_target_page ON refs(target_page);

        -- 属性键值表（支持类型化属性）
        CREATE TABLE IF NOT EXISTS properties (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_id TEXT NOT NULL,   -- 块或页面 UUID
            key       TEXT NOT NULL,
            value     TEXT NOT NULL,
            FOREIGN KEY (entity_id) REFERENCES blocks(id)
        );
        CREATE INDEX IF NOT EXISTS idx_props_entity ON properties(entity_id);
        CREATE INDEX IF NOT EXISTS idx_props_key ON properties(key);
        "
    )?;

    Ok(())
}
