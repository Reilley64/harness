import type { DOMElement } from "ink";
import { useBoxMetrics, useStdin, useStdout } from "ink";
import type { RefObject } from "react";
import { useEffect, useRef } from "react";
import {
  disableMouseScroll,
  enableMouseScroll,
} from "../services/terminal-scroll.js";

export function useMapPointer(
  target: RefObject<DOMElement | null>,
  identity: string | undefined,
  open: () => void,
) {
  const { stdin } = useStdin();
  const { stdout } = useStdout();
  const bounds = useBoxMetrics(target);
  const latest = useRef(open);
  latest.current = open;
  useEffect(() => {
    if (!identity || !bounds.hasMeasured) return;
    let pending = "";
    let clicked = 0;
    const input = (chunk: Buffer | string) => {
      pending = (pending + chunk.toString()).slice(-1024);
      let consumed = 0;
      for (const match of pending.matchAll(/\x1b\[<(\d+);(\d+);(\d+)([mM])/g)) {
        consumed = match.index + match[0].length;
        if (match[1] !== "0" || match[4] !== "M") continue;
        const x = Number(match[2]) - 1,
          y = Number(match[3]) - 1;
        const hit =
          x >= bounds.left &&
          x < bounds.left + bounds.width &&
          y >= bounds.top &&
          y < bounds.top + bounds.height;
        const now = performance.now();
        if (hit && clicked && now - clicked <= 400) {
          clicked = 0;
          latest.current();
        } else clicked = hit ? now : 0;
      }
      pending = pending.slice(consumed).slice(-128);
    };
    stdin.on("data", input);
    stdout.write(enableMouseScroll);
    return () => {
      stdin.off("data", input);
      stdout.write(disableMouseScroll);
    };
  }, [
    stdin,
    stdout,
    identity,
    bounds.hasMeasured,
    bounds.left,
    bounds.top,
    bounds.width,
    bounds.height,
  ]);
}
