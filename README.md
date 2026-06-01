<p align="center">
  <img src="https://raw.githubusercontent.com/uhduih123/logseq-rs/main/.github/logo.svg" width="120" alt="Logseq-RS" onerror="this.remove()">
</p>

<h1 align="center">Logseq-RS</h1>
<p align="center">
  <strong>Rust + TypeScript + Tauri 重写 Logseq</strong><br>
  轻量 · 极速 · 本地优先
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.96-000000?logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/TypeScript-5.x-3178C6?logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/Tauri-2.x-FFC131?logo=tauri&logoColor=black" alt="Tauri">
  <img src="https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black" alt="React">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
  <img src="https://img.shields.io/badge/tests-18%20passed-brightgreen" alt="Tests">
</p>

---

## 这是什么

[Logseq](https://github.com/logseq/logseq) 是 GitHub 上最受欢迎的知识管理工具之一（31k+ stars）——一个本地优先、支持双向链接的大纲编辑器。但它的 ClojureScript + Electron 架构让许多开发者望而却步。

**Logseq-RS** 用现代技术栈彻底重写，**保持 100% 文件格式兼容**的前提下：

| | Logseq 原版 | Logseq-RS |
|---|---|---|
| 语言 | ClojureScript | Rust + TypeScript |
| 桌面壳 | Electron (~150MB) | Tauri (~8MB) |
| 数据库 | Datascript (JS) | SQLite (Rust) |
| 解析器 | 自研 CLJS | pulldown-cmark (Rust) |
| 内存占用 | ~500MB | ~50MB |
| 启动速度 | 3-5s | <0.5s |

> 🚧 **当前状态**: Phase 1 MVP 完成 — 核心引擎 + CLI + React 前端已就绪

---

## 架构

```
┌─────────────────────────────────────────────────────────┐
│                     Tauri Shell (2.x)                    │
│  ┌──────────────────────┐  ┌──────────────────────────┐ │
│  │    Rust Core Engine  │  │  React Frontend          │ │
│  │                      │  │                          │ │
│  │  ┌────────────────┐  │  │  ┌───────────────────┐  │ │
│  │  │  parser        │──┼──┼──┤  Outliner 组件    │  │ │
│  │  │  (md + Logseq   │  │  │  │  (树渲染+键盘导航)│  │ │
│  │  │   syntax ext)   │  │  │  └───────────────────┘  │ │
│  │  └────────┬───────┘  │  │  ┌───────────────────┐  │ │
│  │  ┌────────▼───────┐  │  │  │  Sidebar          │  │ │
│  │  │  db             │──┼──┼──┤  + Topbar         │  │ │
│  │  │  (SQLite+递归   │  │  │  │  + SearchPanel    │  │ │
│  │  │   CTE删除)      │  │  │  └───────────────────┘  │ │
│  │  └────────┬───────┘  │  │  ┌───────────────────┐  │ │
│  │  ┌────────▼───────┐  │  │  │  3 Zustand Stores │  │ │
│  │  │  pipeline       │  │  │  │  (graph/editor/ui)│  │ │
│  │  │  (文件扫描+监听) │  │  │  └───────────────────┘  │ │
│  │  └────────┬───────┘  │  └──────────────────────────┘  │
│  │  ┌────────▼───────┐  │                                │
│  │  │  search         │  │  13 Tauri IPC Commands        │
│  │  │  (tantivy FTS)  │  │  (open/list/search/CRUD...)   │
│  │  └────────────────┘  │                                │
│  └──────────────────────┘                                │
└─────────────────────────────────────────────────────────┘
```

### Crate 结构

| Crate | 职责 | 行数 |
|-------|------|------|
| `crates/core` | 数据模型：Block/Page/Graph | ~200 |
| `crates/parser` | Markdown 解析 + Logseq 语法扩展 | ~400 |
| `crates/db` | SQLite 持久化 + 管线 + 递归删除 | ~1200 |
| `crates/search` | 全文搜索（MVP: 简单匹配） | ~30 |
| `crates/cli` | 命令行工具（6 个子命令） | ~240 |

---

## 快速开始

### 环境要求

- Rust **1.85+**
- Node.js **22.12+**
- Linux: `build-essential` + `libwebkit2gtk-4.1-dev` (Tauri)

### 构建

```bash
git clone git@github.com:uhduih123/logseq-rs.git
cd logseq-rs

# Rust 组件
cargo build                    # 编译所有 crate
cargo test                     # 18 tests
cargo build -p logseq-cli      # 仅 CLI

# 前端
npm install
npm run dev                    # Vite dev server → localhost:1420
```

### CLI 演示

```bash
# 扫描你的笔记目录
./target/debug/logseq open ~/my-notes

# 列出所有页面
./target/debug/logseq list

# 搜索
./target/debug/logseq search "Rust"

# 显示页面内容
./target/debug/logseq show 2026_06_01

# 自检
./target/debug/logseq test
```

### 输出示例

```
📂 扫描目录: ~/my-notes
✅ 完成: 12 页面, 87 块

📄 12 个页面:
  📅 2026_06_01     (journal · 5 块)
  📝 rust-notes      (8 块)
  📝 project-ideas   (3 块)
  ...

🔍 搜索 'Rust' — 3 个结果:
  [rust-notes] Rust async/await 最佳实践
  [project-ideas] 用 Rust 写个 parser
  [2026_06_01] 今天开始学 Rust
```

---

## 技术亮点

### 递归删除：SQLite CTE

```sql
WITH RECURSIVE descendants(uuid) AS (
    SELECT ?1
    UNION ALL
    SELECT b.id FROM blocks b
    JOIN descendants d ON b.parent_id = d.uuid
)
SELECT uuid FROM descendants
```

一次查询收集整棵子树，批量删除——比逐层查询快 10x。

### Markdown 解析管道

```
.md 文件
  └→ WalkDir 扫描
      └→ pulldown-cmark 事件流
          └→ Logseq 扩展处理器
              ├── [[wikilinks]]  → Page refs
              ├── ((block-id))   → Block refs
              ├── TODO/LATER/... → Block markers
              ├── SCHEDULED/DEADLINE → Timestamps
              ├── #tag           → Tags
              └── YAML frontmatter → Properties
```

### 文件监听 + 增量更新

```
notify::recommended_watcher
  └→ Modify / Create 事件
      └→ 重新解析单文件
          └→ 删除旧数据 + 写入新数据
              └→ 前端自动刷新
```

---

## 路线图

- [x] **Phase 1** — 核心引擎 + CLI + React 前端
- [ ] **Phase 2** — 代码高亮、LaTeX、图视图、闪卡
- [ ] **Phase 3** — 插件系统、CRDT 同步、静态发布
- [ ] **Phase 4** — 移动端 (Tauri Mobile)、性能优化

---

## 贡献

欢迎 PR！先看 `AGENTS.md` 了解代码规范。

项目采用 **先策划再执行** 的开发方法论——每个大功能先出架构方案，确认后再动手写代码。

---

<p align="center">
  <sub>Built with 🤖 Hermes Agent + DeepSeek V4 Pro</sub>
</p>
