//! Logseq 搜索 crate（Phase 2 实现 tantivy 全文搜索）
//!
//! MVP 阶段：简单字符串匹配搜索

use logseq_core::model::Block;

/// 简单的内存搜索（MVP 阶段）
/// Phase 2 替换为 tantivy 全文搜索引擎
pub fn search_blocks<'a>(blocks: &'a [Block], query: &str) -> Vec<&'a Block> {
    let query_lower = query.to_lowercase();
    blocks
        .iter()
        .filter(|b| b.title.to_lowercase().contains(&query_lower))
        .collect()
}
