// Tauri IPC 桥接层
// 封装所有 Rust 后端调用

import { invoke } from "@tauri-apps/api/core";
import type { Block, Page, GraphConfig } from "../types/graph";

// --- Graph 操作 ---

export async function openGraph(path: string): Promise<GraphConfig> {
  return invoke("open_graph", { path });
}

export async function listPages(): Promise<Page[]> {
  return invoke("list_pages");
}

export async function getPageBlocks(pageId: string): Promise<Block[]> {
  return invoke("get_page_blocks", { pageId });
}

// --- 块操作 ---

export async function insertBlock(
  title: string,
  pageId: string,
  parentId: string | null,
  leftId: string | null,
  level: number
): Promise<Block> {
  return invoke("insert_block", { title, pageId, parentId, leftId, level });
}

export async function updateBlock(id: string, title: string): Promise<void> {
  return invoke("update_block", { id, title });
}

export async function deleteBlock(id: string): Promise<void> {
  return invoke("delete_block", { id });
}

export async function moveBlock(
  id: string,
  newParentId: string | null,
  newLeftId: string | null
): Promise<void> {
  return invoke("move_block", { id, newParentId, newLeftId });
}

export async function indentBlock(id: string): Promise<void> {
  return invoke("indent_block", { id });
}

export async function outdentBlock(id: string): Promise<void> {
  return invoke("outdent_block", { id });
}

// --- 搜索 ---

export async function searchBlocks(query: string): Promise<Block[]> {
  return invoke("search_blocks", { query });
}

// --- 页面操作 ---

export async function createPage(name: string): Promise<Page> {
  return invoke("create_page", { name });
}

export async function deletePage(id: string): Promise<void> {
  return invoke("delete_page", { id });
}

export async function getBacklinks(blockId: string): Promise<Block[]> {
  return invoke("get_backlinks", { blockId });
}
