// UI 状态管理
// 管理主题、侧边栏、弹窗等

import { create } from "zustand";
import { immer } from "zustand/middleware/immer";

interface UIState {
  // 主题
  theme: "light" | "dark";

  // 侧边栏
  sidebarOpen: boolean;
  sidebarTab: "pages" | "favorites" | "recent";

  // 搜索
  searchOpen: boolean;
  searchQuery: string;

  // 弹窗
  commandPaletteOpen: boolean;

  // 右键菜单
  contextMenu: {
    open: boolean;
    x: number;
    y: number;
    blockId: string | null;
  } | null;

  // 操作
  toggleTheme: () => void;
  toggleSidebar: () => void;
  setSidebarTab: (tab: "pages" | "favorites" | "recent") => void;
  openSearch: () => void;
  closeSearch: () => void;
  setSearchQuery: (q: string) => void;
  openCommandPalette: () => void;
  closeCommandPalette: () => void;
  showContextMenu: (x: number, y: number, blockId: string) => void;
  hideContextMenu: () => void;
}

export const useUIStore = create<UIState>()(
  immer((set) => ({
    theme: "light",
    sidebarOpen: true,
    sidebarTab: "pages",
    searchOpen: false,
    searchQuery: "",
    commandPaletteOpen: false,
    contextMenu: null,

    toggleTheme: () => set((s) => {
      s.theme = s.theme === "light" ? "dark" : "light";
      document.documentElement.className = s.theme;
    }),

    toggleSidebar: () => set((s) => { s.sidebarOpen = !s.sidebarOpen; }),
    setSidebarTab: (tab) => set((s) => { s.sidebarTab = tab; }),

    openSearch: () => set((s) => { s.searchOpen = true; }),
    closeSearch: () => set((s) => { s.searchOpen = false; s.searchQuery = ""; }),
    setSearchQuery: (q) => set((s) => { s.searchQuery = q; }),

    openCommandPalette: () => set((s) => { s.commandPaletteOpen = true; }),
    closeCommandPalette: () => set((s) => { s.commandPaletteOpen = false; }),

    showContextMenu: (x, y, blockId) => set((s) => {
      s.contextMenu = { open: true, x, y, blockId };
    }),
    hideContextMenu: () => set((s) => { s.contextMenu = null; }),
  }))
);
