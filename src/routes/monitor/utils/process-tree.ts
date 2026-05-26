import type { FlatProcess, ProcessNode } from "../types";

export function buildTree(processes: FlatProcess[]): ProcessNode[] {
  const map = new Map<number, ProcessNode>();

  for (const proc of processes) {
    map.set(proc.pid, {
      ...proc,
      children: [],
    });
  }

  const roots: ProcessNode[] = [];

  for (const proc of map.values()) {
    const parent =
      proc.parent_pid != null ? map.get(proc.parent_pid) : undefined;

    const isExplorerParent = parent?.name.toLowerCase() === "explorer.exe";

    if (parent && proc.parent_pid !== proc.pid && !isExplorerParent) {
      parent.children.push(proc);
    } else {
      roots.push(proc);
    }
  }

  return roots;
}

export function cloneTree(nodes: ProcessNode[]): ProcessNode[] {
  return nodes.map((node) => ({
    ...node,
    children: cloneTree(node.children),
  }));
}

export function matchesSearch(p: ProcessNode, searchLower: string): boolean {
  if (!searchLower) return true;

  if (p.name.toLowerCase().includes(searchLower)) {
    return true;
  }

  return p.children.some((c) => matchesSearch(c, searchLower));
}

export function sortTreeBy(
  nodes: ProcessNode[],
  metric: "cpu" | "ram" | "network",
) {
  nodes.sort((a, b) => {
    if (metric === "cpu") {
      return b.cpu_percent - a.cpu_percent;
    }

    if (metric === "ram") {
      return b.ram_bytes - a.ram_bytes;
    }

    return b.net_bps - a.net_bps;
  });

  for (const child of nodes) {
    sortTreeBy(child.children, metric);
  }
}
