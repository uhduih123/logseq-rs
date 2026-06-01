// 前后端共享的类型定义，精确映射 Rust model.rs

export interface BlockProperties {
  map: Record<string, string>;
  order: string[];
  textValues: Record<string, string>;
}

export type BlockFormat = "markdown" | "org";

export type BlockMarker =
  | "TODO"
  | "LATER"
  | "DONE"
  | "NOW"
  | "WAITING"
  | "CANCELED"
  | "IN-PROGRESS"
  | string;

export type Priority = "A" | "B" | "C";

export interface Block {
  uuid: string;
  title: string;
  body: string;
  level: number;
  left: string | null;
  parent: string | null;
  page: string | null;
  refs: string[];
  pathRefs: string[];
  tags: string[];
  marker: BlockMarker | null;
  priority: Priority | null;
  scheduled: string | null;  // ISO8601
  deadline: string | null;   // ISO8601
  collapsed: boolean;
  format: BlockFormat;
  container: boolean;
  properties: BlockProperties;
  preBlock: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface Page {
  id: string;
  name: string;
  title: string | null;
  isJournal: boolean;
  journalDay: number | null;
  properties: BlockProperties;
  namespace: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface GraphConfig {
  path: string;
  preferredFormat: BlockFormat;
  journalEnabled: boolean;
  journalTemplate: string;
  showPreBlock: boolean;
  dateFormat: string;
}

// 树节点（用于大纲渲染）
export interface BlockNode {
  block: Block;
  children: BlockNode[];
  expanded: boolean;
}
