// 编辑器状态管理
// 管理焦点、选区、编辑模式和 undo/redo

import { create } from "zustand";
import { immer } from "zustand/middleware/immer";

interface EditAction {
  type: "insert" | "update" | "delete" | "move";
  blockId: string;
  oldValue?: string;
  newValue?: string;
  oldParent?: string | null;
  newParent?: string | null;
  timestamp: number;
}

interface EditorState {
  // 焦点
  focusedBlockId: string | null;
  editingBlockId: string | null;

  // 选区
  selectedBlockIds: string[];

  // 编辑模式
  editContent: string;

  // Undo/Redo
  undoStack: EditAction[];
  redoStack: EditAction[];

  // 操作
  setFocusedBlock: (id: string | null) => void;
  setEditingBlock: (id: string | null) => void;
  setEditContent: (content: string) => void;
  toggleBlockSelection: (id: string) => void;
  clearSelection: () => void;

  pushUndo: (action: EditAction) => void;
  undo: () => EditAction | null;
  redo: () => EditAction | null;
}

export const useEditorStore = create<EditorState>()(
  immer((set, get) => ({
    focusedBlockId: null,
    editingBlockId: null,
    selectedBlockIds: [],
    editContent: "",
    undoStack: [],
    redoStack: [],

    setFocusedBlock: (id) => {
      set((s) => { s.focusedBlockId = id; });
    },

    setEditingBlock: (id) => {
      set((s) => { s.editingBlockId = id; });
    },

    setEditContent: (content) => {
      set((s) => { s.editContent = content; });
    },

    toggleBlockSelection: (id) => {
      set((s) => {
        const idx = s.selectedBlockIds.indexOf(id);
        if (idx >= 0) {
          s.selectedBlockIds.splice(idx, 1);
        } else {
          s.selectedBlockIds.push(id);
        }
      });
    },

    clearSelection: () => {
      set((s) => { s.selectedBlockIds = []; });
    },

    pushUndo: (action) => {
      set((s) => {
        s.undoStack.push(action);
        s.redoStack = [];
      });
    },

    undo: () => {
      const state = get();
      if (state.undoStack.length === 0) return null;
      const action = state.undoStack[state.undoStack.length - 1];
      set((s) => {
        s.undoStack.pop();
        s.redoStack.push(action);
      });
      return action;
    },

    redo: () => {
      const state = get();
      if (state.redoStack.length === 0) return null;
      const action = state.redoStack[state.redoStack.length - 1];
      set((s) => {
        s.redoStack.pop();
        s.undoStack.push(action);
      });
      return action;
    },
  }))
);
