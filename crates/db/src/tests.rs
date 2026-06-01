//! 数据库 CRUD 集成测试

#[cfg(test)]
mod tests {
    use logseq_core::model::{Block, Page};
    use crate::Database;
    use uuid::Uuid;

    fn make_test_page(name: &str) -> Page {
        let now = chrono::Utc::now();
        Page {
            id: Uuid::new_v4(),
            name: name.to_string(),
            title: Some(format!("Title for {}", name)),
            is_journal: false,
            journal_day: None,
            namespace: None,
            properties: Default::default(),
            created_at: now,
            updated_at: now,
        }
    }

    fn make_test_block(page_id: Uuid, title: &str, level: usize) -> Block {
        Block {
            uuid: Uuid::new_v4(),
            title: title.to_string(),
            level,
            page: Some(page_id),
            ..Default::default()
        }
    }

    #[test]
    fn test_page_crud() {
        let db = Database::in_memory().unwrap();

        // Create
        let page = make_test_page("test_page");
        db.upsert_page(&page).unwrap();

        // Read
        let fetched = db.get_page(page.id).unwrap().unwrap();
        assert_eq!(fetched.name, "test_page");
        assert_eq!(fetched.title.as_deref(), Some("Title for test_page"));

        // Read all
        let all = db.get_all_pages().unwrap();
        assert_eq!(all.len(), 1);

        // Delete
        db.delete_page(page.id).unwrap();
        let all = db.get_all_pages().unwrap();
        assert_eq!(all.len(), 0);
    }

    #[test]
    fn test_block_crud() {
        let db = Database::in_memory().unwrap();

        let page = make_test_page("blocks_test");
        db.upsert_page(&page).unwrap();

        // Create
        let block = make_test_block(page.id, "Hello", 0);
        db.insert_block(&block).unwrap();

        // Read
        let fetched = db.get_block(block.uuid).unwrap().unwrap();
        assert_eq!(fetched.title, "Hello");
        assert_eq!(fetched.page, Some(page.id));

        // Update
        db.update_block_title(block.uuid, "Updated").unwrap();
        let fetched = db.get_block(block.uuid).unwrap().unwrap();
        assert_eq!(fetched.title, "Updated");

        // Delete
        db.delete_block(block.uuid).unwrap();
        assert!(db.get_block(block.uuid).unwrap().is_none());
    }

    #[test]
    fn test_hierarchical_blocks() {
        let db = Database::in_memory().unwrap();

        let page = make_test_page("hierarchy");
        db.upsert_page(&page).unwrap();

        // 创建父子块
        let root = Block {
            title: "Root".to_string(),
            level: 0,
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&root).unwrap();

        let child = Block {
            title: "Child".to_string(),
            level: 1,
            parent: Some(root.uuid),
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&child).unwrap();

        let grandchild = Block {
            title: "Grandchild".to_string(),
            level: 2,
            parent: Some(child.uuid),
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&grandchild).unwrap();

        // 读取页面所有块
        let blocks = db.get_page_blocks(page.id).unwrap();
        assert_eq!(blocks.len(), 3);

        // 验证父子关系
        let fetched_child = blocks.iter().find(|b| b.title == "Child").unwrap();
        assert_eq!(fetched_child.parent, Some(root.uuid));
        assert_eq!(fetched_child.level, 1);

        // 删除父块应同时删除子块
        db.delete_block(root.uuid).unwrap();
        let blocks = db.get_page_blocks(page.id).unwrap();
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_tags() {
        let db = Database::in_memory().unwrap();

        let page = make_test_page("tags_test");
        db.upsert_page(&page).unwrap();

        let block = Block {
            title: "Item with tags".to_string(),
            tags: vec!["rust".to_string(), "tauri".to_string(), "testing".to_string()],
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&block).unwrap();

        let fetched = db.get_block(block.uuid).unwrap().unwrap();
        assert_eq!(fetched.tags.len(), 3);
        assert!(fetched.tags.contains(&"rust".to_string()));
    }

    #[test]
    fn test_indent_outdent() {
        let db = Database::in_memory().unwrap();

        let page = make_test_page("indent_test");
        db.upsert_page(&page).unwrap();

        // 创建两个同级块
        let block1 = Block {
            title: "Block 1".to_string(),
            level: 0,
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&block1).unwrap();

        let block2 = Block {
            title: "Block 2".to_string(),
            level: 0,
            left: Some(block1.uuid),
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&block2).unwrap();

        // 缩进 block2
        db.indent_block(block2.uuid).unwrap();
        let indented = db.get_block(block2.uuid).unwrap().unwrap();
        assert_eq!(indented.level, 1);
        assert_eq!(indented.parent, Some(block1.uuid));

        // 反缩进
        db.outdent_block(block2.uuid).unwrap();
        let outdented = db.get_block(block2.uuid).unwrap().unwrap();
        assert_eq!(outdented.level, 0);
        assert_eq!(outdented.parent, None);
    }

    #[test]
    fn test_move_block() {
        let db = Database::in_memory().unwrap();

        let page = make_test_page("move_test");
        db.upsert_page(&page).unwrap();

        let parent1 = Block {
            title: "Parent 1".to_string(),
            level: 0,
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&parent1).unwrap();

        let parent2 = Block {
            title: "Parent 2".to_string(),
            level: 0,
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&parent2).unwrap();

        let child = Block {
            title: "Child".to_string(),
            level: 1,
            parent: Some(parent1.uuid),
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&child).unwrap();

        // 移动到 parent2 下
        db.move_block(child.uuid, Some(parent2.uuid), None).unwrap();
        let moved = db.get_block(child.uuid).unwrap().unwrap();
        assert_eq!(moved.parent, Some(parent2.uuid));
    }

    #[test]
    fn test_backlinks() {
        let db = Database::in_memory().unwrap();

        let page = make_test_page("links_test");
        db.upsert_page(&page).unwrap();

        let target = Block {
            title: "Target Block".to_string(),
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&target).unwrap();

        // 创建引用块
        let source = Block {
            title: format!("Refers to (({}))", target.uuid),
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&source).unwrap();
        db.rebuild_refs_for_block(source.uuid, &source.title).unwrap();

        // 验证反向链接
        let backlinks = db.get_backlinks(target.uuid).unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0], source.uuid);
    }

    #[test]
    fn test_query_by_tag() {
        let db = Database::in_memory().unwrap();

        let page = make_test_page("query_test");
        db.upsert_page(&page).unwrap();

        let b1 = Block {
            title: "Rust item".to_string(),
            tags: vec!["rust".to_string()],
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&b1).unwrap();

        let b2 = Block {
            title: "Tauri item".to_string(),
            tags: vec!["tauri".to_string()],
            page: Some(page.id),
            ..Default::default()
        };
        db.insert_block(&b2).unwrap();

        // 按标签查询
        let filter = crate::query::QueryFilter {
            tags: vec!["rust".to_string()],
            ..Default::default()
        };
        let results = db.query_blocks(filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust item");
    }
}
