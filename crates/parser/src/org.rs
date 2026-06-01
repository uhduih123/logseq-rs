//! Org-mode 文件解析器
//!
//! TODO: Phase 2 实现 — MVP 阶段只支持 Markdown

use logseq_core::model::Block;
use uuid::Uuid;

/// 解析 Org-mode 文本为块列表（占位，Phase 2 实现）
pub fn parse_org(text: &str, page_id: Option<Uuid>) -> Vec<Block> {
    let _ = (text, page_id);
    // Phase 2: 使用 orgize crate 实现完整解析
    vec![]
}
