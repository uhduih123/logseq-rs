//! 双向链接和引用管理

use crate::Database;
use rusqlite::params;
use uuid::Uuid;

impl Database {
    /// 插入引用记录（source 引用 target）
    pub fn insert_ref(&self, source_id: Uuid, target_id: Option<Uuid>, target_page: Option<&str>) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO refs (source_id, target_id, target_page) VALUES (?1, ?2, ?3)",
            params![
                source_id.to_string(),
                target_id.map(|u| u.to_string()),
                target_page,
            ],
        )?;
        Ok(())
    }

    /// 获取引用某个块的所有源块（反向链接 / backlinks）
    pub fn get_backlinks(&self, block_id: Uuid) -> rusqlite::Result<Vec<Uuid>> {
        let mut stmt = self.conn.prepare(
            "SELECT source_id FROM refs WHERE target_id = ?1"
        )?;
        let ids: Vec<Uuid> = stmt
            .query_map(params![block_id.to_string()], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();
        Ok(ids)
    }

    /// 获取引用某个页面的所有源块
    pub fn get_page_backlinks(&self, page_name: &str) -> rusqlite::Result<Vec<Uuid>> {
        let mut stmt = self.conn.prepare(
            "SELECT source_id FROM refs WHERE target_page = ?1"
        )?;
        let ids: Vec<Uuid> = stmt
            .query_map(params![page_name], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();
        Ok(ids)
    }

    /// 重建一个块的引用（解析标题中的 [[wikilinks]] 和 ((block-refs))）
    pub fn rebuild_refs_for_block(&self, block_id: Uuid, title: &str) -> rusqlite::Result<()> {
        // 先清除旧引用
        self.conn.execute(
            "DELETE FROM refs WHERE source_id = ?1",
            params![block_id.to_string()],
        )?;

        // 提取 [[wikilinks]] 
        let re_wiki = regex_lite::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        for cap in re_wiki.captures_iter(title) {
            let page_name = &cap[1];
            self.insert_ref(block_id, None, Some(page_name))?;
        }

        // 提取 ((block-refs))
        let re_block = regex_lite::Regex::new(r"\(\(([^)]+)\)\)").unwrap();
        for cap in re_block.captures_iter(title) {
            if let Ok(target_id) = Uuid::parse_str(&cap[1]) {
                self.insert_ref(block_id, Some(target_id), None)?;
            }
        }

        Ok(())
    }
}
