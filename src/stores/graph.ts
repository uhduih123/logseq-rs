// Graph 状态管理
// 管理当前图谱的页面和块数据

import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { Block, Page, GraphConfig } from "../types/graph";
import * as ipc from "../ipc/commands";

interface GraphState {
  // 图谱配置
  config: GraphConfig | null;
  graphPath: string | null;

  // 数据
  pages: Record<string, Page>;
  blocks: Record<string, Block>;
  currentPageId: string | null;

  // 加载状态
  loading: boolean;
  error: string | null;

  // 操作
  openGraph: (path: string) => Promise<void>;
  loadPages: () => Promise<void>;
  loadPageBlocks: (pageId: string) => Promise<void>;
  setCurrentPage: (pageId: string) => void;

  // 本地块操作（乐观更新）
  addBlock: (block: Block) => void;
  updateBlockTitle: (id: string, title: string) => void;
  removeBlock: (id: string) => void;
  moveBlockLocal: (id: string, newParent: string | null, newLeft: string | null) => void;
}

export const useGraphStore = create<GraphState>()(
  immer((set, get) => ({
    config: null,
    graphPath: null,
    pages: {},
    blocks: {},
    currentPageId: null,
    loading: false,
    error: null,

    openGraph: async (path: string) => {
      set((s) => { s.loading = true; s.error = null; });
      try {
        const config = await ipc.openGraph(path);
        const pages = await ipc.listPages();
        const pagesMap: Record<string, Page> = {};
        for (const p of pages) pagesMap[p.id] = p;
        set((s) => {
          s.config = config;
          s.graphPath = path;
          s.pages = pagesMap;
          s.loading = false;
        });
      } catch (e: any) {
        set((s) => { s.error = e?.message ?? "Unknown error"; s.loading = false; });
      }
    },

    loadPages: async () => {
      const pages = await ipc.listPages();
      const pagesMap: Record<string, Page> = {};
      for (const p of pages) pagesMap[p.id] = p;
      set((s) => { s.pages = pagesMap; });
    },

    loadPageBlocks: async (pageId: string) => {
      set((s) => { s.loading = true; });
      try {
        const blocks = await ipc.getPageBlocks(pageId);
        const blocksMap: Record<string, Block> = {};
        for (const b of blocks) blocksMap[b.uuid] = b;
        set((s) => {
          s.blocks = blocksMap;
          s.currentPageId = pageId;
          s.loading = false;
        });
      } catch (e: any) {
        set((s) => { s.error = e?.message; s.loading = false; });
      }
    },

    setCurrentPage: (pageId) => {
      set((s) => { s.currentPageId = pageId; });
    },

    addBlock: (block) => {
      set((s) => { s.blocks[block.uuid] = block; });
    },

    updateBlockTitle: (id, title) => {
      set((s) => {
        if (s.blocks[id]) {
          s.blocks[id].title = title;
          s.blocks[id].updatedAt = new Date().toISOString();
        }
      });
    },

    removeBlock: (id) => {
      set((s) => {
        // 递归删除所有子块
        const removeChildren = (parentId: string) => {
          Object.values(s.blocks).forEach((b) => {
            if (b.parent === parentId) {
              removeChildren(b.uuid);
              delete s.blocks[b.uuid];
            }
          });
        };
        removeChildren(id);
        delete s.blocks[id];
      });
    },

    moveBlockLocal: (id, newParent, newLeft) => {
      set((s) => {
        const block = s.blocks[id];
        if (block) {
          block.parent = newParent;
          block.left = newLeft;
          if (newParent) {
            const parent = s.blocks[newParent];
            block.level = parent ? parent.level + 1 : 0;
          } else {
            block.level = 0;
          }
        }
      });
    },
  }))
);
