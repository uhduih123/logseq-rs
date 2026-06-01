//! 图查询 DSL
//!
//! 提供类似 Logseq 高级查询（query_dsl）的查询能力。
//! 当前 MVP 阶段提供基础查询，后续扩展为完整 Datalog 风格查询。

use crate::Database;
use logseq_core::model::Block;

/// 查询条件
#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    /// 按标签筛选
    pub tags: Vec<String>,
    /// 按任务标记筛选
    pub marker: Option<String>,
    /// 按优先级筛选
    pub priority: Option<String>,
    /// 全文搜索关键词
    pub keyword: Option<String>,
    /// 限制返回数量
    pub limit: Option<usize>,
}

impl Database {
    /// 按条件查询块
    pub fn query_blocks(&self, filter: QueryFilter) -> rusqlite::Result<Vec<Block>> {
        let mut sql = String::from(
            "SELECT DISTINCT b.id FROM blocks b"
        );
        let mut conditions: Vec<String> = Vec::new();
        let mut _params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        // 标签筛选（需要 JOIN tags 表）
        if !filter.tags.is_empty() {
            sql.push_str(" JOIN tags t ON b.id = t.block_id");
            let tag_placeholders: Vec<String> = filter
                .tags
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect();
            conditions.push(format!("t.tag IN ({})", tag_placeholders.join(",")));
        }

        if let Some(ref marker) = filter.marker {
            conditions.push(format!("b.marker = '{}'", marker.replace('\'', "''")));
        }

        if let Some(ref priority) = filter.priority {
            conditions.push(format!("b.priority = '{}'", priority.replace('\'', "''")));
        }

        if let Some(ref keyword) = filter.keyword {
            let escaped = keyword.replace('\'', "''");
            conditions.push(format!("b.title LIKE '%{}%'", escaped));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY b.updated_at DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut stmt = self.conn.prepare(&sql)?;

        // 构建参数
        let mut param_values: Vec<String> = Vec::new();
        for tag in &filter.tags {
            param_values.push(tag.clone());
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();

        let ids: Vec<String> = stmt
            .query_map(rusqlite::params_from_iter(param_refs), |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // 逐个加载完整 Block
        let mut blocks = Vec::new();
        for id_str in ids {
            if let Ok(id) = uuid::Uuid::parse_str(&id_str) {
                if let Ok(Some(block)) = self.get_block(id) {
                    blocks.push(block);
                }
            }
        }

        Ok(blocks)
    }
}

#[cfg(test)]
mod tests {
    // 测试需要在有数据库的环境下运行
}
