pub mod model;

use model::{Block, Page, GraphConfig};
use std::collections::HashMap;
use uuid::Uuid;

/// Graph 是核心数据结构——管理一个知识图谱中的所有页面和块
pub struct Graph {
    pub config: GraphConfig,
    /// 所有页面 (page-uuid -> Page)
    pub pages: HashMap<Uuid, Page>,
    /// 所有块 (block-uuid -> Block)
    pub blocks: HashMap<Uuid, Block>,
    /// 页面 -> 块列表（保持顺序）
    pub page_blocks: HashMap<Uuid, Vec<Uuid>>,
    /// 标签 -> 块列表
    pub tag_index: HashMap<String, Vec<Uuid>>,
}

impl Graph {
    pub fn new(config: GraphConfig) -> Self {
        Self {
            config,
            pages: HashMap::new(),
            blocks: HashMap::new(),
            page_blocks: HashMap::new(),
            tag_index: HashMap::new(),
        }
    }

    /// 添加一个块
    pub fn add_block(&mut self, block: Block) {
        // 更新标签索引
        for tag in &block.tags {
            self.tag_index
                .entry(tag.clone())
                .or_default()
                .push(block.uuid);
        }
        // 更新 page_blocks 映射
        if let Some(page_id) = block.page {
            self.page_blocks
                .entry(page_id)
                .or_default()
                .push(block.uuid);
        }
        self.blocks.insert(block.uuid, block);
    }

    /// 获取某页面的所有块（按树形顺序）
    pub fn get_page_blocks(&self, page_id: Uuid) -> Vec<&Block> {
        self.page_blocks
            .get(&page_id)
            .map(|ids| ids.iter().filter_map(|id| self.blocks.get(id)).collect())
            .unwrap_or_default()
    }

    /// 通过标签查找块
    pub fn find_by_tag(&self, tag: &str) -> Vec<&Block> {
        self.tag_index
            .get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.blocks.get(id)).collect())
            .unwrap_or_default()
    }

    /// 查找被其他块引用的块（反向链接）
    pub fn find_backlinks(&self, block_id: Uuid) -> Vec<&Block> {
        self.blocks
            .values()
            .filter(|b| b.refs.contains(&block_id))
            .collect()
    }

    /// 移除一个块
    pub fn remove_block(&mut self, block_id: Uuid) {
        if let Some(block) = self.blocks.remove(&block_id) {
            // 清理标签索引
            for tag in &block.tags {
                if let Some(ids) = self.tag_index.get_mut(tag) {
                    ids.retain(|id| *id != block_id);
                }
            }
            // 清理 page_blocks 映射
            if let Some(page_id) = block.page {
                if let Some(ids) = self.page_blocks.get_mut(&page_id) {
                    ids.retain(|id| *id != block_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_find_block() {
        let config = GraphConfig::default();
        let mut graph = Graph::new(config);

        let mut block = Block::default();
        block.title = "Hello World".to_string();
        block.tags = vec!["test".to_string()];
        let block_id = block.uuid;

        graph.add_block(block);

        assert_eq!(graph.blocks.len(), 1);
        assert_eq!(graph.find_by_tag("test").len(), 1);
        assert_eq!(graph.find_by_tag("test")[0].uuid, block_id);
    }
}
