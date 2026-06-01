// App 主组件
// 组合侧边栏、顶部栏、大纲编辑器和搜索面板

import { useEffect } from "react";
import { Sidebar } from "./components/sidebar/Sidebar";
import { Topbar } from "./components/topbar/Topbar";
import { Outliner } from "./components/outliner/Outliner";
import { SearchPanel } from "./components/search/SearchPanel";
import { ContextMenu } from "./components/common/ContextMenu";
import { useGraphStore } from "./stores/graph";
import { useUIStore } from "./stores/ui";

function App() {
  const graphPath = useGraphStore((s) => s.graphPath);
  const openGraph = useGraphStore((s) => s.openGraph);
  const error = useGraphStore((s) => s.error);
  const theme = useUIStore((s) => s.theme);

  // 初始化主题
  useEffect(() => {
    document.documentElement.className = theme;
  }, [theme]);

  // 全局快捷键
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const meta = e.metaKey || e.ctrlKey;

      // ⌘K / Ctrl+K → 搜索
      if (meta && e.key === "k") {
        e.preventDefault();
        useUIStore.getState().openSearch();
      }

      // ⌘P / Ctrl+P → 命令面板
      if (meta && e.key === "p") {
        e.preventDefault();
        useUIStore.getState().openCommandPalette();
      }

      // ⌘B / Ctrl+B → 切换侧边栏
      if (meta && e.key === "b") {
        e.preventDefault();
        useUIStore.getState().toggleSidebar();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // 没有打开 Graph 时显示欢迎界面
  if (!graphPath) {
    return (
      <div className="h-screen flex flex-col items-center justify-center bg-[var(--ls-bg)] text-[var(--ls-text)]">
        <h1 className="text-2xl font-light mb-2">Logseq RS</h1>
        <p className="text-sm text-[var(--ls-text-secondary)] mb-6">
          Rust + Tauri 重构版
        </p>
        <button
          onClick={() => {
            // TODO: 用 Tauri dialog 选择目录
            const path = prompt("请输入 graph 目录路径:");
            if (path) {
              openGraph(path);
            }
          }}
          className="px-4 py-2 bg-[var(--ls-primary)] text-white rounded-md text-sm hover:opacity-90"
        >
          打开 Graph
        </button>
        {error && (
          <p className="mt-4 text-sm text-red-500">{error}</p>
        )}
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-[var(--ls-bg)]">
      <Topbar title={graphPath} />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <main className="flex-1 overflow-hidden">
          <Outliner />
        </main>
      </div>
      <SearchPanel />
      <ContextMenu />
    </div>
  );
}

export default App;
