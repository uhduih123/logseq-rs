# Logseq-RS

用 **Rust + TypeScript + Tauri** 重写的 [Logseq](https://github.com/logseq/logseq) 开源知识管理工具。

> 🚧 MVP 阶段 — 核心管线已跑通，Tauri GUI 待集成

## 架构

```
crates/core      → 数据模型 (Block/Page/Graph)
crates/parser    → Markdown 解析器 (pulldown-cmark)
crates/db        → SQLite 持久化 + 文件管线
crates/search    → 全文搜索
crates/cli       → 命令行工具
src/             → React 前端 (TypeScript + Zustand)
src-tauri/       → Tauri 桌面壳
```

## 技术栈

| 层 | 技术 |
|---|---|
| 核心 | Rust (+ serde, rusqlite, pulldown-cmark, notify) |
| 前端 | React 19 + TypeScript + Zustand + Tailwind CSS |
| 桌面 | Tauri 2.x |
| 构建 | Cargo workspace + Vite |

## 快速开始

```bash
# 编译
cargo build

# 测试（18 tests）
cargo test

# CLI 演示
./target/debug/logseq test
./target/debug/logseq open ~/my-notes
./target/debug/logseq list
./target/debug/logseq search "关键词"

# 前端（需要 Node.js ≥22.12）
npm install
npm run dev
```

## CLI 命令

```
logseq open <dir>       打开 graph 目录
logseq list             列出所有页面
logseq show <页面名>    显示页面内容
logseq search <关键词>  搜索块
logseq parse <文件>     解析单个 Markdown 文件
logseq test             运行自检
```

## 进度

- [x] Phase 1 MVP：大纲编辑器核心
  - [x] Block/Page 数据模型
  - [x] Markdown 解析（Logseq 语法）
  - [x] SQLite 持久化（递归删除、缩进、反向链接）
  - [x] 文件扫描 → 入库管线
  - [x] 文件变更监听
  - [x] Journal 日期支持
  - [x] React 前端（Outliner + Sidebar + SearchPanel）
  - [x] CLI 命令行工具
  - [x] 18 个测试全绿
  - [ ] Tauri GUI 集成（缺 GTK3 系统库）
- [ ] Phase 2：完整知识管理（代码高亮、LaTeX、图视图、闪卡）
- [ ] Phase 3：插件系统 + 同步 + 发布
- [ ] Phase 4：移动端 + 性能优化

## License

MIT
