//! Logseq Core Model
//! 
//! 精确映射 Logseq 的数据模型，保持与现有 graph 文件格式 100% 兼容。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 块格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BlockFormat {
    #[default]
    Markdown,
    Org,
}

/// 任务标记：TODO / LATER / DONE / NOW / WAITING 等
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockMarker {
    Todo,
    Later,
    Done,
    Now,
    Waiting,
    Canceled,
    InProgress,
    Custom(String),
}

impl std::fmt::Display for BlockMarker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockMarker::Todo => write!(f, "TODO"),
            BlockMarker::Later => write!(f, "LATER"),
            BlockMarker::Done => write!(f, "DONE"),
            BlockMarker::Now => write!(f, "NOW"),
            BlockMarker::Waiting => write!(f, "WAITING"),
            BlockMarker::Canceled => write!(f, "CANCELED"),
            BlockMarker::InProgress => write!(f, "IN-PROGRESS"),
            BlockMarker::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// 优先级 A/B/C
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    A,
    B,
    C,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::A => write!(f, "A"),
            Priority::B => write!(f, "B"),
            Priority::C => write!(f, "C"),
        }
    }
}

/// 块属性（键值对）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockProperties {
    pub map: HashMap<String, String>,
    /// 保持 Logseq 属性插入顺序
    pub order: Vec<String>,
    /// 属性的原始文本值（未解析的）
    pub text_values: HashMap<String, String>,
}

/// 核心数据结构：块（Block）
/// 
/// 映射自 Logseq 的 `:block/*` 属性族
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// 块唯一标识符（UUID v4）
    pub uuid: Uuid,
    /// 块标题/内容文本（Markdown 或 Org 格式）
    pub title: String,
    /// 块体内容（用于嵌入块的内容区）
    pub body: String,
    /// 层级深度（0 = 根块/页面标题，1 = 一级子块，...）
    pub level: usize,
    /// 左侧兄弟块 ID（用于有序列表排序）
    pub left: Option<Uuid>,
    /// 父块 ID
    pub parent: Option<Uuid>,
    /// 所属页面 ID
    pub page: Option<Uuid>,
    /// 被引用的块 ID 列表（双向链接目标）
    pub refs: Vec<Uuid>,
    /// 路径引用（如 [[page/subpage]]）
    pub path_refs: Vec<Uuid>,
    /// 标签列表（#tag）
    pub tags: Vec<String>,
    /// 任务状态标记
    pub marker: Option<BlockMarker>,
    /// 优先级
    pub priority: Option<Priority>,
    /// 计划日期（SCHEDULED）
    pub scheduled: Option<DateTime<Utc>>,
    /// 截止日期（DEADLINE）
    pub deadline: Option<DateTime<Utc>>,
    /// 是否折叠
    pub collapsed: bool,
    /// 块格式
    pub format: BlockFormat,
    /// 是否为容器块（如 QUOTE、SRC、EXAMPLE 等）
    pub container: bool,
    /// 自定义属性
    pub properties: BlockProperties,
    /// 是否为 pre-block（前置元数据块）
    pub pre_block: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后修改时间
    pub updated_at: DateTime<Utc>,
}

impl Default for Block {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4(),
            title: String::new(),
            body: String::new(),
            level: 0,
            left: None,
            parent: None,
            page: None,
            refs: Vec::new(),
            path_refs: Vec::new(),
            tags: Vec::new(),
            marker: None,
            priority: None,
            scheduled: None,
            deadline: None,
            collapsed: false,
            format: BlockFormat::Markdown,
            container: false,
            properties: BlockProperties::default(),
            pre_block: false,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 页面
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Page {
    /// 页面 ID（文件名 stem）
    pub id: Uuid,
    /// 页面名称（文件名）
    pub name: String,
    /// 页面标题（H1 第一行）
    pub title: Option<String>,
    /// 是否为 journal 页面
    pub is_journal: bool,
    /// journal 日期（仅 journal 页面有）
    pub journal_day: Option<i64>,
    /// 页面属性
    pub properties: BlockProperties,
    /// 命名空间（如 "projects/2024"）
    pub namespace: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后修改时间
    pub updated_at: DateTime<Utc>,
}

/// Graph 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphConfig {
    /// Graph 根目录路径
    pub path: String,
    /// 首选格式：Markdown 或 Org
    pub preferred_format: BlockFormat,
    /// 是否启用 journal
    pub journal_enabled: bool,
    /// journal 页面命名模板
    pub journal_template: String,
    /// 是否显示前置元数据
    pub show_pre_block: bool,
    /// 自定义日期格式
    pub date_format: String,
}
