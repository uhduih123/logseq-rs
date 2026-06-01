// 大纲编辑器 —— 核心组件
// 递归渲染块树，处理键盘导航、编辑、缩进等

import { useCallback, useEffect, useRef, useState } from "react";
import { useGraphStore } from "../stores/graph";
import { useEditorStore } from "../stores/editor";
import { useUIStore } from "../stores/ui";
import { buildBlockTree, flattenTree, type FlatBlock } from "../editor/block-tree";
import * as ipc from "../ipc/commands";
import type { Block } from "../types/graph";

export function Outliner() {
  const blocks = useGraphStore((s) => s.blocks);
  const loading = useGraphStore((s) => s.loading);
  const focusedBlockId = useEditorStore((s) => s.focusedBlockId);
  const containerRef = useRef<HTMLDivElement>(null);

  // 构建树
  const blockList = Object.values(blocks);
  const tree = buildBlockTree(blockList);
  const flatList = flattenTree(tree);

  // 确保焦点块可见
  useEffect(() => {
    if (focusedBlockId) {
      const el = document.getElementById(`block-${focusedBlockId}`);
      el?.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [focusedBlockId]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-[var(--ls-text-secondary)]">
        加载中...
      </div>
    );
  }

  if (flatList.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-[var(--ls-text-secondary)]">
        空白页面 — 开始输入吧
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="outliner flex-1 overflow-y-auto px-4 py-2"
      onClick={() => useEditorStore.getState().setFocusedBlock(null)}
    >
      {flatList.map((item, idx) => (
        <BlockRow key={item.block.uuid} item={item} index={idx} />
      ))}
    </div>
  );
}

// ── 单个块行组件 ──

function BlockRow({ item, index }: { item: FlatBlock; index: number }) {
  const { block, depth, hasChildren, expanded } = item;
  const focusedBlockId = useEditorStore((s) => s.focusedBlockId);
  const editingBlockId = useEditorStore((s) => s.editingBlockId);
  const editContent = useEditorStore((s) => s.editContent);
  const setFocusedBlock = useEditorStore((s) => s.setFocusedBlock);
  const setEditingBlock = useEditorStore((s) => s.setEditingBlock);
  const setEditContent = useEditorStore((s) => s.setEditContent);
  const pushUndo = useEditorStore((s) => s.pushUndo);
  const updateBlockTitle = useGraphStore((s) => s.updateBlockTitle);
  const addBlock = useGraphStore((s) => s.addBlock);
  const removeBlock = useGraphStore((s) => s.removeBlock);
  const showContextMenu = useUIStore((s) => s.showContextMenu);

  const isFocused = focusedBlockId === block.uuid;
  const isEditing = editingBlockId === block.uuid;
  const inputRef = useRef<HTMLInputElement>(null);

  // 进入编辑模式时自动聚焦
  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  // 处理块点击
  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      setFocusedBlock(block.uuid);
      if (!isEditing) {
        setEditContent(block.title);
      }
    },
    [block.uuid, block.title, isEditing, setFocusedBlock, setEditContent]
  );

  // 双击进入编辑模式
  const handleDoubleClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      setEditingBlock(block.uuid);
      setEditContent(block.title);
    },
    [block.uuid, block.title, setEditingBlock, setEditContent]
  );

  // 保存编辑
  const commitEdit = useCallback(async () => {
    const newTitle = editContent.trim();
    if (newTitle && newTitle !== block.title) {
      pushUndo({
        type: "update",
        blockId: block.uuid,
        oldValue: block.title,
        newValue: newTitle,
        timestamp: Date.now(),
      });
      updateBlockTitle(block.uuid, newTitle);
      try {
        await ipc.updateBlock(block.uuid, newTitle);
      } catch (err) {
        console.error("Failed to save block:", err);
      }
    }
    setEditingBlock(null);
  }, [block.uuid, block.title, editContent, pushUndo, updateBlockTitle, setEditingBlock]);

  // 键盘处理
  const handleKeyDown = useCallback(
    async (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        e.preventDefault();
        if (isEditing) {
          await commitEdit();
        }
        // 创建新块
        const newBlock: Block = {
          uuid: crypto.randomUUID(),
          title: "",
          body: "",
          level: block.level,
          left: block.uuid,
          parent: block.parent,
          page: block.page,
          refs: [],
          pathRefs: [],
          tags: [],
          marker: null,
          priority: null,
          scheduled: null,
          deadline: null,
          collapsed: false,
          format: "markdown",
          container: false,
          properties: { map: {}, order: [], textValues: {} },
          preBlock: false,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        };
        addBlock(newBlock);
        setEditingBlock(newBlock.uuid);
        setEditContent("");
        try {
          await ipc.insertBlock("", block.page!, block.parent, block.uuid, block.level);
        } catch (err) {
          console.error("Failed to create block:", err);
        }
        return;
      }

      if (e.key === "Backspace" && isEditing && editContent === "") {
        e.preventDefault();
        if (block.parent) {
          removeBlock(block.uuid);
          setFocusedBlock(block.parent);
          try {
            await ipc.deleteBlock(block.uuid);
          } catch (err) {
            console.error("Failed to delete block:", err);
          }
        }
        return;
      }

      if (e.key === "Tab" && !isEditing) {
        e.preventDefault();
        if (e.shiftKey) {
          // 反缩进
          try {
            await ipc.outdentBlock(block.uuid);
          } catch (err) {
            console.error("Failed to outdent:", err);
          }
        } else {
          // 缩进
          try {
            await ipc.indentBlock(block.uuid);
          } catch (err) {
            console.error("Failed to indent:", err);
          }
        }
        return;
      }

      if (e.key === "Escape" && isEditing) {
        setEditingBlock(null);
        return;
      }

      if (e.key === "ArrowUp" && !isEditing) {
        e.preventDefault();
        setFocusedBlock(block.uuid);
        // 移到上一个同级块
        const prevEl = (
          document.getElementById(`block-${block.uuid}`)?.previousElementSibling
        );
        if (prevEl) {
          const prevId = prevEl.id.replace("block-", "");
          setFocusedBlock(prevId);
        }
        return;
      }

      if (e.key === "ArrowDown" && !isEditing) {
        e.preventDefault();
        const nextEl = (
          document.getElementById(`block-${block.uuid}`)?.nextElementSibling
        );
        if (nextEl) {
          const nextId = nextEl.id.replace("block-", "");
          setFocusedBlock(nextId);
        }
        return;
      }
    },
    [
      block, isEditing, editContent, commitEdit,
      addBlock, removeBlock, setEditingBlock, setEditContent, setFocusedBlock,
    ]
  );

  // 右键菜单
  const handleContextMenu = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      showContextMenu(e.clientX, e.clientY, block.uuid);
    },
    [block.uuid, showContextMenu]
  );

  const paddingLeft = 12 + depth * 24;

  return (
    <div
      id={`block-${block.uuid}`}
      className={`
        block-row flex items-start py-0.5 cursor-pointer select-none
        transition-colors duration-75
        ${isFocused ? "bg-[var(--ls-primary-bg)]" : "hover:bg-[var(--ls-bg-secondary)]"}
      `}
      style={{ paddingLeft: `${paddingLeft}px` }}
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      onContextMenu={handleContextMenu}
      onKeyDown={handleKeyDown}
      tabIndex={0}
    >
      {/* 展开/折叠手柄 */}
      <span className="block-bullet w-5 h-5 flex-shrink-0 flex items-center justify-center text-[var(--ls-text-secondary)] text-xs mr-1 mt-0.5">
        {hasChildren ? (
          <span className="cursor-pointer">
            {expanded ? "▾" : "▸"}
          </span>
        ) : (
          <span className="text-[var(--ls-secondary)] opacity-40">•</span>
        )}
      </span>

      {/* 任务标记 */}
      {block.marker && (
        <span className={`
          block-marker text-xs font-semibold mr-1.5 mt-0.5 px-1 rounded
          ${block.marker === "DONE" ? "text-green-600 bg-green-100" : ""}
          ${block.marker === "TODO" ? "text-orange-600 bg-orange-100" : ""}
          ${block.marker === "LATER" ? "text-blue-600 bg-blue-100" : ""}
        `}>
          {block.marker}
        </span>
      )}

      {/* 优先级 */}
      {block.priority && (
        <span className="block-priority text-xs font-bold text-[var(--ls-primary)] mr-1.5 mt-0.5">
          [#{block.priority}]
        </span>
      )}

      {/* 内容 */}
      <div className="block-content flex-1 min-w-0">
        {isEditing ? (
          <input
            ref={inputRef}
            type="text"
            value={editContent}
            onChange={(e) => setEditContent(e.target.value)}
            onBlur={commitEdit}
            onKeyDown={handleKeyDown}
            className="w-full bg-transparent outline-none text-sm"
            placeholder="输入内容..."
          />
        ) : (
          <div className="block-title text-sm whitespace-pre-wrap break-words">
            {block.title || (
              <span className="text-[var(--ls-text-secondary)] italic">
                空块
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
