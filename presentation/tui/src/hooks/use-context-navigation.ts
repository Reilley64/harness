import { useEffect, useRef } from "react";
import { useTerminalInput } from "./use-terminal-input.js";

export function useContextNavigation(
  active: boolean,
  toggle: () => void,
  griller: () => void,
  children?: () => void,
) {
  const input = useTerminalInput();
  const latest = useRef({ active, toggle, griller, children });
  latest.current = { active, toggle, griller, children };
  useEffect(
    () =>
      input.subscribe((event) => {
        if (event.type !== "shortcut" || !latest.current.active) return;
        if (event.action === "toggle") latest.current.toggle();
        else if (event.action === "griller") latest.current.griller();
        else latest.current.children?.();
      }),
    [input],
  );
}
