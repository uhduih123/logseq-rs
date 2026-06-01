// 侧边栏组件

import { useGraphStore } from "../stores/graph";
import { useUIStore } from "../stores/ui";

export function Sidebar() {
  const pages = useGraphStore((s) => s.pages);
  const currentPageId = useGraphStore((s) => s.currentPageId);
  const loadPageBlocks = useGraphStore((s) => s.loadPageBlocks);
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const sidebarTab = useUIStore((s) => s.sidebarTab);
  const setSidebarTab = useUIStore((s) => s.setSidebarTab);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);

  if (!sidebarOpen) return null;

  const pageList = Object.values(pages);

  return (
    <div className="sidebar w-60 bg-[var(--ls-bg-secondary)] border-r border-[var(--ls-border)] flex flex-col">
      {/* Tabs */}
      <div className="flex border-b border-[var(--ls-border)]">
        {(["pages", "favorites"] as const).map((tab) => (
          <button
            key={tab}
            className={`
              flex-1 py-2 text-xs font-medium
              ${sidebarTab === tab
                ? "text-[var(--ls-primary)] border-b-2 border-[var(--ls-primary)]"
                : "text-[var(--ls-text-secondary)] hover:text-[var(--ls-text)]"
              }
            `}
            onClick={() => setSidebarTab(tab)}
          >
            {tab === "pages" ? "页面" : "收藏"}
          </button>
        ))}
      </div>

      {/* Page list */}
      <div className="flex-1 overflow-y-auto p-2">
        {pageList.length === 0 ? (
          <div className="text-xs text-[var(--ls-text-secondary)] p-2">
            暂无页面
          </div>
        ) : (
          pageList.map((p) => (
            <div
              key={p.id}
              className={`
                page-item px-2 py-1 rounded text-sm cursor-pointer truncate
                ${currentPageId === p.id
                  ? "bg-[var(--ls-primary-bg)] text-[var(--ls-primary)] font-medium"
                  : "text-[var(--ls-text)] hover:bg-[var(--ls-bg)]"
                }
              `}
              onClick={() => loadPageBlocks(p.id)}
            >
              {p.title || p.name}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
