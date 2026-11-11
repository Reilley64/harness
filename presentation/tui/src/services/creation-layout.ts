import type { CreationNode } from "../types/creation-map.js";

export const nodeWidth = 26;
export const nodeHeight = 4;
const generationGap = 5;
const rowGap = 2;

export type CreationPosition = { x: number; y: number };

export type CreationLayout = {
  visible: CreationNode[];
  positions: Map<string, CreationPosition>;
  width: number;
  height: number;
  geometry: string;
};

export function creationLayout(
  nodes: CreationNode[],
  collapsed: Set<string>,
): CreationLayout {
  const index = new Map(nodes.map((node) => [node.id, node]));
  const ids = new Set(index.keys());
  const children = new Map<string, CreationNode[]>();
  for (const node of nodes) {
    if (!node.creatorId || !ids.has(node.creatorId)) continue;
    const siblings = children.get(node.creatorId) ?? [];
    siblings.push(node);
    children.set(node.creatorId, siblings);
  }
  const roots = nodes.filter(
    (node) => !node.creatorId || !ids.has(node.creatorId),
  );
  const visible: CreationNode[] = [];
  const positions = new Map<string, CreationPosition>();
  const visited = new Set<string>();
  const heights = new Map<string, number>();
  const forest: CreationNode[] = [];
  for (const root of [...roots, ...nodes]) {
    if (visited.has(root.id)) continue;
    forest.push(root);
    const stack: { node: CreationNode; measured: boolean }[] = [
      { node: root, measured: false },
    ];
    while (stack.length) {
      const entry = stack.pop();
      if (!entry) break;
      const { node, measured } = entry;
      if (measured) {
        const kids = collapsed.has(node.id)
          ? []
          : (children.get(node.id) ?? []).filter((child) =>
              heights.has(child.id),
            );
        heights.set(
          node.id,
          Math.max(
            nodeHeight,
            kids.reduce(
              (sum, child) => sum + (heights.get(child.id) ?? nodeHeight),
              0,
            ) +
              Math.max(0, kids.length - 1) * rowGap,
          ),
        );
      } else {
        if (visited.has(node.id)) continue;
        visited.add(node.id);
        visible.push(node);
        stack.push({ node, measured: true });
        if (!collapsed.has(node.id)) {
          for (const child of [...(children.get(node.id) ?? [])].reverse())
            if (!visited.has(child.id))
              stack.push({ node: child, measured: false });
        }
      }
    }
  }
  const hidden = new Set<string>();
  for (const node of nodes) {
    const path = new Set<string>([node.id]);
    let parent = node.creatorId;
    while (parent && !path.has(parent)) {
      if (collapsed.has(parent)) {
        hidden.add(node.id);
        break;
      }
      path.add(parent);
      parent = index.get(parent)?.creatorId ?? null;
    }
  }
  let top = 0;
  for (const root of forest) {
    if (hidden.has(root.id) || positions.has(root.id)) continue;
    const stack = [{ node: root, depth: 0, top }];
    while (stack.length) {
      const entry = stack.pop();
      if (!entry || positions.has(entry.node.id) || hidden.has(entry.node.id))
        continue;
      const { node, depth, top: row } = entry;
      const height = heights.get(node.id) ?? nodeHeight;
      positions.set(node.id, {
        x: depth * (nodeWidth + generationGap),
        y: row + Math.floor((height - nodeHeight) / 2),
      });
      if (collapsed.has(node.id)) continue;
      let next = row;
      for (const child of children.get(node.id) ?? []) {
        stack.push({ node: child, depth: depth + 1, top: next });
        next += (heights.get(child.id) ?? nodeHeight) + rowGap;
      }
    }
    top += (heights.get(root.id) ?? nodeHeight) + rowGap;
  }
  const shown = visible.filter((node) => positions.has(node.id));
  const geometry = shown
    .map(
      (node) =>
        `${node.id}:${positions.get(node.id)?.x}:${positions.get(node.id)?.y}`,
    )
    .join("|");
  return {
    visible: shown,
    positions,
    width: [...positions.values()].reduce(
      (width, p) => Math.max(width, p.x + nodeWidth),
      nodeWidth,
    ),
    height: Math.max(nodeHeight, top),
    geometry,
  };
}

export function creationCanvas(
  layout: CreationLayout,
  pan: CreationPosition,
  width: number,
  height: number,
): string[] {
  const grid = Array.from({ length: height }, () =>
    Array<string>(width).fill(" "),
  );
  const put = (x: number, y: number, character: string) => {
    if (x >= pan.x && x < pan.x + width && y >= pan.y && y < pan.y + height)
      grid[y - pan.y][x - pan.x] = character;
  };
  const horizontal = (start: number, end: number, row: number) => {
    for (
      let col = Math.max(start, pan.x);
      col < Math.min(end, pan.x + width);
      col++
    )
      put(col, row, "─");
  };
  for (const node of layout.visible) {
    const from = node.creatorId
      ? layout.positions.get(node.creatorId)
      : undefined;
    const to = layout.positions.get(node.id);
    if (!from || !to || to.x <= from.x) continue;
    const x = from.x + nodeWidth + 2;
    const a = from.y + 2;
    const b = to.y + 2;
    horizontal(from.x + nodeWidth, x, a);
    for (
      let row = Math.max(Math.min(a, b), pan.y);
      row <= Math.min(Math.max(a, b), pan.y + height - 1);
      row++
    )
      put(x, row, "│");
    put(x, a, a === b ? "─" : "┤");
    horizontal(x + 1, to.x - 1, b);
    put(x, b, a === b ? "─" : a < b ? "└" : "┌");
    put(to.x - 1, b, ">");
  }
  return grid.map((row) => row.join(""));
}
