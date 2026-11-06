import { useSyncExternalStore } from "react";
import { accentColor, palette, rainbowColor } from "../theme.js";

const startedAt = performance.now();
const listeners = new Set<() => void>();
let color = accentColor;
let timer: ReturnType<typeof setInterval> | undefined;

function subscribe(listener: () => void) {
  listeners.add(listener);
  if (timer === undefined) {
    timer = setInterval(() => {
      color = rainbowColor(performance.now() - startedAt);
      for (const notify of listeners) notify();
    }, 250);
    timer.unref();
  }
  return () => {
    listeners.delete(listener);
    if (listeners.size === 0 && timer !== undefined) {
      clearInterval(timer);
      timer = undefined;
    }
  };
}

const inactive = () => () => {};
const snapshot = () => color;
const serverSnapshot = () => accentColor;

export function useRainbowColor(active: boolean): string {
  const color = useSyncExternalStore(
    active ? subscribe : inactive,
    snapshot,
    serverSnapshot,
  );
  return active ? color : palette.borderMuted;
}
