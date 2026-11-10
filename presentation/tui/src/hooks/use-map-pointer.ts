import type { DOMElement } from "ink";
import { measureElement, useBoxMetrics } from "ink";
import type { RefObject } from "react";
import { useEffect, useLayoutEffect, useRef } from "react";
import type { CreationLayout } from "../services/creation-layout.js";
import { nodeHeight, nodeWidth } from "../services/creation-layout.js";
import { useTerminalInput } from "./use-terminal-input.js";

export function useMapPointer(
  target: RefObject<DOMElement | null>,
  layout: CreationLayout,
  pan: { x: number; y: number },
  select: (id: string) => void,
  open: (id: string) => void,
  scroll: (rows: number) => void,
) {
  const input = useTerminalInput();
  const bounds = useBoxMetrics(target);
  const validity = useRef({ geometry: "", since: performance.now() });
  const latest = useRef({
    bounds,
    layout,
    pan,
    select,
    open,
    scroll,
    validity: validity.current,
  });
  useLayoutEffect(() => {
    const measured = target.current ? measureElement(target.current) : null;
    const absolute = {
      ...bounds,
      left: measured?.x ?? 0,
      top: measured?.y ?? 0,
      width: measured?.width ?? 0,
      height: measured?.height ?? 0,
      hasMeasured: !!measured && measured.width > 0 && measured.height > 0,
    };
    const geometry = `${layout.geometry}/${pan.x}/${pan.y}/${absolute.hasMeasured}/${absolute.left}/${absolute.top}/${absolute.width}/${absolute.height}`;
    if (validity.current.geometry !== geometry)
      validity.current = { geometry, since: performance.now() };
    latest.current = {
      bounds: absolute,
      layout,
      pan,
      select,
      open,
      scroll,
      validity: validity.current,
    };
  }, [bounds, layout, pan, select, open, scroll, target]);
  useEffect(() => {
    let clicked: {
      id: string;
      time: number;
      geometry: string;
      released: boolean;
      second: boolean;
    } | null = null;
    const releaseMouse = input.acquireMouse();
    const unsubscribe = input.subscribe((event) => {
      if (event.type !== "mouse") return;
      const state = latest.current;
      if (!state.bounds.hasMeasured) {
        clicked = null;
        return;
      }
      const { x, y } = event;
      const inside =
        x >= state.bounds.left &&
        x < state.bounds.left + state.bounds.width &&
        y >= state.bounds.top &&
        y < state.bounds.top + state.bounds.height;
      if (event.wheel) {
        clicked = null;
        if (inside && !event.released) state.scroll(event.wheel * 3);
        return;
      }
      if (event.startedAt < state.validity.since) {
        clicked = null;
        return;
      }
      if (clicked?.geometry !== state.validity.geometry) clicked = null;
      const cx = x - state.bounds.left + state.pan.x;
      const cy = y - state.bounds.top + state.pan.y;
      const hit = inside
        ? state.layout.visible.find((node) => {
            const p = state.layout.positions.get(node.id);
            return (
              p &&
              cx >= p.x &&
              cx < p.x + nodeWidth &&
              cy >= p.y &&
              cy < p.y + nodeHeight
            );
          })
        : undefined;
      if (event.released) {
        if (
          clicked &&
          hit?.id === clicked.id &&
          (event.button === 0 || event.button === 3)
        ) {
          if (clicked.second && event.receivedAt - clicked.time <= 400) {
            clicked = null;
            state.open(hit.id);
          } else clicked.released = true;
        } else clicked = null;
        return;
      }
      if (event.button !== 0 || !hit) {
        clicked = null;
        return;
      }
      if (
        clicked?.id === hit.id &&
        clicked.released &&
        event.receivedAt - clicked.time <= 400
      ) {
        clicked.released = false;
        clicked.second = true;
      } else {
        clicked = {
          id: hit.id,
          time: event.receivedAt,
          geometry: state.validity.geometry,
          released: false,
          second: false,
        };
        state.select(hit.id);
      }
    });
    return () => {
      unsubscribe();
      releaseMouse();
    };
  }, [input]);
}
