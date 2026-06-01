// 将扁平的块数组构建为树结构

import type { Block, BlockNode } from "../types/graph";

/**
 * 把 blocks 数组转成嵌套的 BlockNode 树
 */
export function buildBlockTree(blocks: Block[]): BlockNode[] {
  const blockMap = new Map<string, BlockNode>();
  const roots: BlockNode[] = [];

  // 第一遍：创建所有节点
  for (const block of blocks) {
    blockMap.set(block.uuid, {
      block,
      children: [],
      expanded: !block.collapsed,
    });
  }

  // 第二遍：建立父子关系
  for (const block of blocks) {
    const node = blockMap.get(block.uuid)!;
    if (block.parent) {
      const parent = blockMap.get(block.parent);
      if (parent) {
        parent.children.push(node);
        // 按 left 排序
        parent.children.sort((a, b) => {
          const aLeft = a.block.left;
          const bLeft = b.block.left;
          if (aLeft === b.block.uuid) return -1;
          if (bLeft === a.block.uuid) return 1;
          return 0;
        });
      } else {
        roots.push(node);
      }
    } else {
      roots.push(node);
    }
  }

  return roots;
}

/**
 * 展平树为可视化列表（用于虚拟滚动时计算缩进）
 */
export interface FlatBlock {
  block: Block;
  depth: number;
  hasChildren: boolean;
  expanded: boolean;
  isLastChild: boolean;
}

export function flattenTree(
  nodes: BlockNode[],
  depth: number = 0,
  result: FlatBlock[] = []
): FlatBlock[] {
  for (let i = 0; i < nodes.length; i++) {
    const node = nodes[i];
    const hasChildren = node.children.length > 0;
    result.push({
      block: node.block,
      depth,
      hasChildren,
      expanded: node.expanded,
      isLastChild: i === nodes.length - 1,
    });

    if (node.expanded && hasChildren) {
      flattenTree(node.children, depth + 1, result);
    }
  }
  return result;
}

/**
 * 获取块的后代数量（用于删除时级的删除确认）
 */
export function getDescendantCount(node: BlockNode): number {
  let count = 0;
  for (const child of node.children) {
    count += 1 + getDescendantCount(child);
  }
  return count;
}
