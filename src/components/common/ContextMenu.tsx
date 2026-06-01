// 右键菜单

import { useUIStore } from "../../stores/ui";
import { useGraphStore } from "../../stores/graph";
import { useEditorStore } from "../../stores/editor";
import * as ipc from "../../ipc/commands";

export function ContextMenu() {
  const contextMenu = useUIStore((s) => s.contextMenu);
  const hideContextMenu = useUIStore((s) => s.hideContextMenu);
  const blocks = useGraphStore((s) => s.blocks);
  const removeBlock = useGraphStore((s) => s.removeBlock);
  const setEditingBlock = useEditorStore((s) => s.setEditingBlock);
  const setEditContent = useEditorStore((s) => s.setEditContent);

  if (!contextMenu?.open || !contextMenu.blockId) return null;

  const block = blocks[contextMenu.blockId];
  if (!block) return null;

  const menuItems = [
    {
      label: "编辑",
      action: () => {
        setEditingBlock(block.uuid);
        setEditContent(block.title);
      },
    },
    {
      label: "在下方插入",
      action: async () => {
        // TODO: 实现插入逻辑
      },
    },
    {
      label: "缩进",
      action: async () => {
        await ipc.indentBlock(block.uuid).catch(console.error);
      },
    },
    {
      label: "反缩进",
      action: async () => {
        await ipc.outdentBlock(block.uuid).catch(console.error);
      },
    },
    { label: "分隔线", action: () => {} }, // separator
    {
      label: "复制块引用",
      action: () => {
        navigator.clipboard.writeText(`((${block.uuid}))`);
      },
    },
    {
      label: "删除",
      danger: true,
      action: async () => {
        removeBlock(block.uuid);
        await ipc.deleteBlock(block.uuid).catch(console.error);
      },
    },
  ];

  return (
    <>
      {/* 点击空白关闭 */}
      <div
        className="fixed inset-0 z-50"
        onClick={hideContextMenu}
      />

      {/* 菜单 */}
      <div
        className="fixed z-50 bg-[var(--ls-bg)] border border-[var(--ls-border)] rounded-md shadow-lg py-1 min-w-[160px]"
        style={{
          left: Math.min(contextMenu.x, window.innerWidth - 180),
          top: Math.min(contextMenu.y, window.innerHeight - 300),
        }}
      >
        {menuItems.map((item, idx) => {
          if (item.label === "分隔线") {
            return (
              <div
                key={`sep-${idx}`}
                className="my-1 border-t border-[var(--ls-border)]"
              />
            );
          }
          return (
            <button
              key={idx}
              className={`w-full text-left px-3 py-1.5 text-sm hover:bg-[var(--ls-bg-secondary)]
                ${item.danger ? "text-red-500 hover:bg-red-50" : "text-[var(--ls-text)]"}
              `}
              onClick={() => {
                item.action();
                hideContextMenu();
              }}
            >
              {item.label}
            </button>
          );
        })}
      </div>
    </>
  );
}
