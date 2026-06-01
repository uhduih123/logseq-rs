// 搜索面板

import { useState, useEffect, useCallback } from "react";
import { useUIStore } from "../stores/ui";
import { useGraphStore } from "../stores/graph";
import * as ipc from "../ipc/commands";
import type { Block } from "../types/graph";

export function SearchPanel() {
  const searchOpen = useUIStore((s) => s.searchOpen);
  const searchQuery = useUIStore((s) => s.searchQuery);
  const setSearchQuery = useUIStore((s) => s.setSearchQuery);
  const closeSearch = useUIStore((s) => s.closeSearch);
  const loadPageBlocks = useGraphStore((s) => s.loadPageBlocks);

  const [results, setResults] = useState<Block[]>([]);
  const [selectedIdx, setSelectedIdx] = useState(0);

  useEffect(() => {
    if (searchQuery.length > 1) {
      ipc.searchBlocks(searchQuery).then(setResults).catch(() => setResults([]));
      setSelectedIdx(0);
    } else {
      setResults([]);
    }
  }, [searchQuery]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Escape") {
        closeSearch();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIdx((i) => Math.min(i + 1, results.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIdx((i) => Math.max(i - 1, 0));
      } else if (e.key === "Enter" && results[selectedIdx]) {
        const block = results[selectedIdx];
        if (block.page) {
          loadPageBlocks(block.page);
        }
        closeSearch();
      }
    },
    [results, selectedIdx, closeSearch, loadPageBlocks]
  );

  if (!searchOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-20 bg-black/30">
      <div className="w-full max-w-lg bg-[var(--ls-bg)] rounded-lg shadow-lg border border-[var(--ls-border)] overflow-hidden">
        {/* Search input */}
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="搜索页面或块..."
          className="w-full px-4 py-3 text-sm bg-transparent outline-none border-b border-[var(--ls-border)]"
          autoFocus
        />

        {/* Results */}
        <div className="max-h-64 overflow-y-auto">
          {results.length === 0 && searchQuery.length > 1 && (
            <div className="p-4 text-sm text-[var(--ls-text-secondary)] text-center">
              无结果
            </div>
          )}
          {results.map((block, idx) => (
            <div
              key={block.uuid}
              className={`
                px-4 py-2 text-sm cursor-pointer border-b border-[var(--ls-border)] last:border-0
                ${idx === selectedIdx
                  ? "bg-[var(--ls-primary-bg)]"
                  : "hover:bg-[var(--ls-bg-secondary)]"
                }
              `}
              onClick={() => {
                if (block.page) loadPageBlocks(block.page);
                closeSearch();
              }}
            >
              <div className="truncate">{block.title || "(空块)"}</div>
              {block.marker && (
                <span className="text-xs text-[var(--ls-text-secondary)]">{block.marker}</span>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
