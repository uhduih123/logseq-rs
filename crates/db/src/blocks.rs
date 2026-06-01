//! 块 CRUD 操作

use crate::Database;
use logseq_core::model::{Block, BlockFormat, BlockMarker, BlockProperties, Priority};
use rusqlite::params;
use uuid::Uuid;

impl Database {
    /// 插入单个块
    pub fn insert_block(&self, block: &Block) -> rusqlite::Result<()> {
        let marker_str = block.marker.as_ref().map(|m| match m {
            BlockMarker::Todo => "TODO",
            BlockMarker::Later => "LATER",
            BlockMarker::Done => "DONE",
            BlockMarker::Now => "NOW",
            BlockMarker::Waiting => "WAITING",
            BlockMarker::Canceled => "CANCELED",
            BlockMarker::InProgress => "IN-PROGRESS",
            BlockMarker::Custom(s) => s.as_str(),
        });

        let priority_str = block.priority.as_ref().map(|p| match p {
            Priority::A => "A",
            Priority::B => "B",
            Priority::C => "C",
        });

        let format_str = match block.format {
            BlockFormat::Markdown => "markdown",
            BlockFormat::Org => "org",
        };

        let properties_json = serde_json::to_string(&block.properties).unwrap_or_default();

        self.conn.execute(
            "INSERT OR REPLACE INTO blocks
             (id, title, body, level, left_id, parent_id, page_id, format,
              marker, priority, scheduled, deadline, collapsed, container,
              pre_block, properties, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                block.uuid.to_string(),
                block.title,
                block.body,
                block.level,
                block.left.map(|u| u.to_string()),
                block.parent.map(|u| u.to_string()),
                block.page.map(|u| u.to_string()),
                format_str,
                marker_str,
                priority_str,
                block.scheduled.map(|d| d.to_rfc3339()),
                block.deadline.map(|d| d.to_rfc3339()),
                block.collapsed as i32,
                block.container as i32,
                block.pre_block as i32,
                properties_json,
                block.created_at.to_rfc3339(),
                block.updated_at.to_rfc3339(),
            ],
        )?;

        // 插入标签
        for tag in &block.tags {
            self.conn.execute(
                "INSERT INTO tags (block_id, tag) VALUES (?1, ?2)",
                params![block.uuid.to_string(), tag],
            )?;
        }

        Ok(())
    }

    /// 批量插入块
    pub fn insert_blocks(&self, blocks: &[Block]) -> rusqlite::Result<()> {
        for block in blocks {
            self.insert_block(block)?;
        }
        Ok(())
    }

    /// 通过 UUID 查询块
    pub fn get_block(&self, id: Uuid) -> rusqlite::Result<Option<Block>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, body, level, left_id, parent_id, page_id, format,
                    marker, priority, scheduled, deadline, collapsed, container,
                    pre_block, properties, created_at, updated_at
             FROM blocks WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id.to_string()], |row| {
            Ok(BlockRow {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                level: row.get(3)?,
                left_id: row.get(4)?,
                parent_id: row.get(5)?,
                page_id: row.get(6)?,
                format: row.get(7)?,
                marker: row.get(8)?,
                priority: row.get(9)?,
                scheduled: row.get(10)?,
                deadline: row.get(11)?,
                collapsed: row.get(12)?,
                container: row.get(13)?,
                pre_block: row.get(14)?,
                properties: row.get(15)?,
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?;

        if let Some(row) = rows.next() {
            let row = row?;
            let tags = self.get_block_tags(id)?;
            Ok(Some(row_to_block(row, tags)))
        } else {
            Ok(None)
        }
    }

    /// 查询某页面下的所有块
    pub fn get_page_blocks(&self, page_id: Uuid) -> rusqlite::Result<Vec<Block>> {
        let mut stmt = self.conn.prepare(
            "SELECT b.id, b.title, b.body, b.level, b.left_id, b.parent_id, b.page_id, b.format,
                    b.marker, b.priority, b.scheduled, b.deadline, b.collapsed, b.container,
                    b.pre_block, b.properties, b.created_at, b.updated_at
             FROM blocks b WHERE b.page_id = ?1
             ORDER BY b.level, b.created_at"
        )?;

        let rows = stmt.query_map(params![page_id.to_string()], |row| {
            Ok(BlockRow {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                level: row.get(3)?,
                left_id: row.get(4)?,
                parent_id: row.get(5)?,
                page_id: row.get(6)?,
                format: row.get(7)?,
                marker: row.get(8)?,
                priority: row.get(9)?,
                scheduled: row.get(10)?,
                deadline: row.get(11)?,
                collapsed: row.get(12)?,
                container: row.get(13)?,
                pre_block: row.get(14)?,
                properties: row.get(15)?,
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?;

        let mut blocks = Vec::new();
        for row in rows {
            let row = row?;
            let block_id = Uuid::parse_str(&row.id).unwrap_or_default();
            let tags = self.get_block_tags(block_id)?;
            blocks.push(row_to_block(row, tags));
        }
        Ok(blocks)
    }

    /// 删除块及其所有后代块（递归）
    pub fn delete_block(&self, id: Uuid) -> rusqlite::Result<()> {
        // 用递归 CTE 收集所有后代块 ID
        let mut stmt = self.conn.prepare(
            "WITH RECURSIVE descendants(uuid) AS (
                SELECT ?1
                UNION ALL
                SELECT b.id FROM blocks b
                JOIN descendants d ON b.parent_id = d.uuid
            )
            SELECT uuid FROM descendants"
        )?;

        let ids: Vec<String> = stmt
            .query_map(params![id.to_string()], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // 删除所有后代块的标签和引用
        for child_id in &ids {
            self.conn.execute(
                "DELETE FROM tags WHERE block_id = ?1",
                params![child_id],
            )?;
            self.conn.execute(
                "DELETE FROM refs WHERE source_id = ?1 OR target_id = ?1",
                params![child_id],
            )?;
        }

        // 批量删除所有块
        for child_id in &ids {
            self.conn.execute(
                "DELETE FROM blocks WHERE id = ?1",
                params![child_id],
            )?;
        }

        Ok(())
    }

    /// 更新块的文本内容
    pub fn update_block_title(&self, id: Uuid, title: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE blocks SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, chrono::Utc::now().to_rfc3339(), id.to_string()],
        )?;
        Ok(())
    }

    /// 获取块的标签
    fn get_block_tags(&self, block_id: Uuid) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT tag FROM tags WHERE block_id = ?1"
        )?;
        let tags: Vec<String> = stmt.query_map(params![block_id.to_string()], |row| {
            row.get(0)
        })?.filter_map(|r| r.ok()).collect();
        Ok(tags)
    }
}

/// 数据库行 → Block 转换的中间结构
struct BlockRow {
    id: String,
    title: String,
    body: String,
    level: i64,
    left_id: Option<String>,
    parent_id: Option<String>,
    page_id: Option<String>,
    format: String,
    marker: Option<String>,
    priority: Option<String>,
    scheduled: Option<String>,
    deadline: Option<String>,
    collapsed: i64,
    container: i64,
    pre_block: i64,
    properties: String,
    created_at: String,
    updated_at: String,
}

fn row_to_block(row: BlockRow, tags: Vec<String>) -> Block {
    let parse_uuid = |s: &Option<String>| {
        s.as_ref().and_then(|s| Uuid::parse_str(s).ok())
    };

    let parse_datetime = |s: &Option<String>| {
        s.as_ref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
        })
    };

    let marker = row.marker.as_deref().map(|m| match m {
        "TODO" => BlockMarker::Todo,
        "LATER" => BlockMarker::Later,
        "DONE" => BlockMarker::Done,
        "NOW" => BlockMarker::Now,
        "WAITING" => BlockMarker::Waiting,
        "CANCELED" => BlockMarker::Canceled,
        "IN-PROGRESS" => BlockMarker::InProgress,
        s => BlockMarker::Custom(s.to_string()),
    });

    let priority = row.priority.as_deref().map(|p| match p {
        "A" => Priority::A,
        "B" => Priority::B,
        "C" => Priority::C,
        _ => Priority::A,
    });

    let format = match row.format.as_str() {
        "org" => BlockFormat::Org,
        _ => BlockFormat::Markdown,
    };

    let properties: BlockProperties = serde_json::from_str(&row.properties).unwrap_or_default();

    Block {
        uuid: Uuid::parse_str(&row.id).unwrap_or_default(),
        title: row.title,
        body: row.body,
        level: row.level as usize,
        left: parse_uuid(&row.left_id),
        parent: parse_uuid(&row.parent_id),
        page: parse_uuid(&row.page_id),
        tags,
        refs: Vec::new(),      // 由 links 模块填充
        path_refs: Vec::new(), // 由 links 模块填充
        marker,
        priority,
        scheduled: parse_datetime(&row.scheduled),
        deadline: parse_datetime(&row.deadline),
        collapsed: row.collapsed != 0,
        format,
        container: row.container != 0,
        pre_block: row.pre_block != 0,
        properties,
        created_at: parse_datetime(&Some(row.created_at)).unwrap_or_default(),
        updated_at: parse_datetime(&Some(row.updated_at)).unwrap_or_default(),
    }
}
