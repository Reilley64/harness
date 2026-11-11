import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import {
  creationLayout,
  nodeHeight,
  nodeWidth,
} from "../services/creation-layout.js";
import type { CreationNode } from "../types/creation-map.js";

export function useCreationMap(agent: Agent, width: number, height: number) {
  const [nodes, setNodes] = useState<CreationNode[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [selected, setSelected] = useState<string | null>(null);
  const selectedRef = useRef(selected);
  const desiredFocus = useRef<string | undefined>(undefined);
  const [collapsed, setCollapsed] = useState(new Set<string>());
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [unseen, setUnseen] = useState(new Set<string>());
  const [follow, setFollow] = useState(false);
  const following = useRef(follow);
  following.current = follow;
  const [followTarget, setFollowTarget] = useState<string | null>(null);
  const initialized = useRef(false);
  const known = useRef(new Set<string>());

  useEffect(() => {
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout> | undefined;
    const refresh = async () => {
      try {
        const read: CreationNode[] = [];
        const ids = new Set<string>();
        const cursors = new Set<string>();
        let after: string | null = null;
        do {
          const page = await agent.readCreationPage(after, controller.signal);
          if (controller.signal.aborted) return;
          for (const node of page.agents) {
            if (ids.has(node.id))
              throw new Error("Duplicate creation-map identity across pages.");
            ids.add(node.id);
            read.push(node);
          }
          after = page.next;
          if (after && cursors.has(after))
            throw new Error("Creation-map pagination did not advance.");
          if (after) cursors.add(after);
        } while (after);
        if (controller.signal.aborted) return;
        const additions = read.filter((node) => !known.current.has(node.id));
        if (initialized.current && additions.length) {
          setUnseen(
            (prior) => new Set([...prior, ...additions.map((node) => node.id)]),
          );
          if (following.current) {
            const newest = additions.at(-1);
            const ancestors = new Set<string>();
            let parent = newest?.creatorId;
            const index = new Map(read.map((node) => [node.id, node]));
            while (parent && !ancestors.has(parent)) {
              ancestors.add(parent);
              parent = index.get(parent)?.creatorId;
            }
            setCollapsed(
              (prior) => new Set([...prior].filter((id) => !ancestors.has(id))),
            );
            setFollowTarget(newest?.id ?? null);
          }
        }
        initialized.current = true;
        for (const node of read) known.current.add(node.id);
        setNodes((prior) => {
          const updates = new Map(read.map((node) => [node.id, node]));
          return [
            ...prior.map((node) => {
              const next = updates.get(node.id);
              updates.delete(node.id);
              return next && next.revision >= node.revision ? next : node;
            }),
            ...[...updates.values()].sort(
              (a, b) => a.createdAt - b.createdAt || a.id.localeCompare(b.id),
            ),
          ];
        });
        setError(null);
      } catch (error) {
        if (!controller.signal.aborted) setError(errorMessage(error));
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false);
          timer = setTimeout(refresh, 1000);
        }
      }
    };
    void refresh();
    return () => {
      controller.abort();
      clearTimeout(timer);
    };
  }, [agent]);

  const layout = useMemo(
    () => creationLayout(nodes, collapsed),
    [nodes, collapsed],
  );
  const previous = useRef({ layout, pan, width, height });
  useLayoutEffect(() => {
    if (selectedRef.current === null && layout.visible.length) {
      const first =
        layout.visible.find((node) => node.id === desiredFocus.current) ??
        layout.visible[0];
      if (first) {
        selectedRef.current = first.id;
        setSelected(first.id);
        desiredFocus.current = undefined;
      }
    }
    const prior = previous.current;
    let next = pan;
    if (followTarget && layout.positions.has(followTarget)) {
      const p = layout.positions.get(followTarget);
      if (p)
        next = {
          x: Math.max(0, p.x - Math.floor((width - nodeWidth) / 2)),
          y: Math.max(0, p.y - Math.floor((height - nodeHeight) / 2)),
        };
      setFollowTarget(null);
    } else if (prior.layout.geometry !== layout.geometry) {
      const inView = prior.layout.visible.filter((node) => {
        const p = prior.layout.positions.get(node.id);
        return (
          p &&
          p.x + nodeWidth > prior.pan.x &&
          p.x < prior.pan.x + prior.width &&
          p.y + nodeHeight > prior.pan.y &&
          p.y < prior.pan.y + prior.height
        );
      });
      const anchor =
        inView.find(
          (node) =>
            node.id === selectedRef.current && layout.positions.has(node.id),
        ) ?? inView.find((node) => layout.positions.has(node.id));
      const before = anchor ? prior.layout.positions.get(anchor.id) : undefined;
      const after = anchor ? layout.positions.get(anchor.id) : undefined;
      if (before && after)
        next = {
          x: Math.max(0, prior.pan.x + after.x - before.x),
          y: Math.max(0, prior.pan.y + after.y - before.y),
        };
    }
    previous.current = { layout, pan: next, width, height };
    if (next.x !== pan.x || next.y !== pan.y) setPan(next);
  }, [layout, pan, width, height, followTarget]);

  function select(id: string) {
    selectedRef.current = id;
    setSelected(id);
  }
  function centre(id: string | null = selectedRef.current) {
    const p = id ? layout.positions.get(id) : undefined;
    if (p)
      setPan({
        x: Math.max(0, p.x - Math.floor((width - nodeWidth) / 2)),
        y: Math.max(0, p.y - Math.floor((height - nodeHeight) / 2)),
      });
  }
  function focus(id?: string) {
    desiredFocus.current = id;
    const target =
      nodes.find((node) => node.id === id) ??
      nodes.find((node) => node.id === selectedRef.current) ??
      layout.visible[0];
    if (target && target.id !== selectedRef.current) {
      select(target.id);
      const ancestors = new Set<string>();
      let parent = target.creatorId;
      while (parent && !ancestors.has(parent)) {
        ancestors.add(parent);
        parent = nodes.find((node) => node.id === parent)?.creatorId ?? null;
      }
      setCollapsed(
        (prior) => new Set([...prior].filter((id) => !ancestors.has(id))),
      );
      setFollowTarget(target.id);
    }
  }
  function moveSelection(direction: number) {
    const ids = layout.visible.map((node) => node.id);
    if (!ids.length) return;
    const index = ids.indexOf(selectedRef.current ?? "");
    const id = ids[(Math.max(-1, index) + direction + ids.length) % ids.length];
    if (id) {
      select(id);
      const p = layout.positions.get(id);
      if (
        p &&
        (p.x < pan.x ||
          p.x + nodeWidth > pan.x + width ||
          p.y < pan.y ||
          p.y + nodeHeight > pan.y + height)
      )
        centre(id);
    }
  }
  return {
    nodes,
    layout,
    loading,
    error,
    selected: nodes.find((node) => node.id === selected) ?? null,
    selectedId: () => selectedRef.current,
    select,
    focus,
    centre,
    moveSelection,
    collapsed,
    collapse: () => {
      const id = selectedRef.current;
      if (id)
        setCollapsed((prior) => {
          const next = new Set(prior);
          if (next.has(id)) next.delete(id);
          else next.add(id);
          return next;
        });
    },
    pan,
    move: (x: number, y: number) =>
      setPan((p) => ({
        x: Math.max(0, Math.min(Math.max(0, layout.width - 1), p.x + x)),
        y: Math.max(0, Math.min(Math.max(0, layout.height - 1), p.y + y)),
      })),
    unseen,
    follow,
    toggleFollow: () => setFollow((value) => !value),
    jumpNew: () => {
      const id = [...unseen].at(-1);
      if (!id) return;
      const path = new Set<string>();
      let parent = nodes.find((node) => node.id === id)?.creatorId;
      while (parent && !path.has(parent)) {
        path.add(parent);
        parent = nodes.find((node) => node.id === parent)?.creatorId;
      }
      setCollapsed(
        (prior) => new Set([...prior].filter((id) => !path.has(id))),
      );
      select(id);
      setFollowTarget(id);
      setUnseen(new Set());
    },
  };
}
