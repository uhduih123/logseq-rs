// 顶部工具栏

import { useGraphStore } from "../stores/graph";
import { useUIStore } from "../stores/ui";

interface TopbarProps {
  title?: string;
}

export function Topbar({ title }: TopbarProps) {
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const openSearch = useUIStore((s) => s.openSearch);
  const openCommandPalette = useUIStore((s) => s.openCommandPalette);
  const toggleTheme = useUIStore((s) => s.toggleTheme);
  const theme = useUIStore((s) => s.theme);

  return (
    <div className="topbar h-10 bg-[var(--ls-bg)] border-b border-[var(--ls-border)] flex items-center px-3 gap-2">
      {/* Toggle sidebar */}
      <button
        onClick={toggleSidebar}
        className="w-7 h-7 flex items-center justify-center rounded hover:bg-[var(--ls-bg-secondary)] text-[var(--ls-text-secondary)]"
      >
        {sidebarOpen ? "◀" : "▶"}
      </button>

      {/* Title */}
      <div className="flex-1 text-sm font-medium text-[var(--ls-text)] truncate">
        {title || "Logseq-RS"}
      </div>

      {/* Search */}
      <button
        onClick={openSearch}
        className="px-3 py-1 text-xs rounded bg-[var(--ls-bg-secondary)] border border-[var(--ls-border)] text-[var(--ls-text-secondary)] hover:text-[var(--ls-text)]"
      >
        ⌘K 搜索
      </button>

      {/* Theme toggle */}
      <button
        onClick={toggleTheme}
        className="w-7 h-7 flex items-center justify-center rounded hover:bg-[var(--ls-bg-secondary)] text-[var(--ls-text-secondary)]"
        title="切换主题"
      >
        {theme === "dark" ? "☀️" : "🌙"}
      </button>
    </div>
  );
}
