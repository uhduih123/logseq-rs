//! 页面 CRUD 操作

use crate::Database;
use logseq_core::model::Page;
use rusqlite::params;
use uuid::Uuid;

impl Database {
    /// 插入或更新页面
    pub fn upsert_page(&self, page: &Page) -> rusqlite::Result<()> {
        let properties_json = serde_json::to_string(&page.properties).unwrap_or_default();

        self.conn.execute(
            "INSERT OR REPLACE INTO pages
             (id, name, title, is_journal, journal_day, namespace, properties, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                page.id.to_string(),
                page.name,
                page.title,
                page.is_journal as i32,
                page.journal_day,
                page.namespace,
                properties_json,
                page.created_at.to_rfc3339(),
                page.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// 通过 ID 获取页面
    pub fn get_page(&self, id: Uuid) -> rusqlite::Result<Option<Page>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, title, is_journal, journal_day, namespace, properties, created_at, updated_at
             FROM pages WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })?;

        if let Some(row) = rows.next() {
            let (id, name, title, is_journal, journal_day, namespace, props, created, updated) = row?;
            Ok(Some(Page {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                name,
                title,
                is_journal: is_journal != 0,
                journal_day,
                namespace,
                properties: serde_json::from_str(&props).unwrap_or_default(),
                created_at: chrono::DateTime::parse_from_rfc3339(&created)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_default(),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取所有页面
    pub fn get_all_pages(&self) -> rusqlite::Result<Vec<Page>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, title, is_journal, journal_day, namespace, properties, created_at, updated_at
             FROM pages ORDER BY name"
        )?;

        let mut pages = Vec::new();
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })?;

        for row in rows {
            let (id, name, title, is_journal, journal_day, namespace, props, created, updated) = row?;
            pages.push(Page {
                id: Uuid::parse_str(&id).unwrap_or_default(),
                name,
                title,
                is_journal: is_journal != 0,
                journal_day,
                namespace,
                properties: serde_json::from_str(&props).unwrap_or_default(),
                created_at: chrono::DateTime::parse_from_rfc3339(&created)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_default(),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated)
                    .map(|d| d.with_timezone(&chrono::Utc))
                    .unwrap_or_default(),
            });
        }
        Ok(pages)
    }

    /// 删除页面及其所有块
    pub fn delete_page(&self, id: Uuid) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM blocks WHERE page_id = ?1",
            params![id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM pages WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }
}
