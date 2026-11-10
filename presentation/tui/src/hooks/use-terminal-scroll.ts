import { useEffect, useRef } from "react";
import { useTerminalInput } from "./use-terminal-input.js";

export function useTerminalScroll(
  scroll: (rows: number) => void,
  active: boolean,
) {
  const input = useTerminalInput();
  const latest = useRef({ scroll, active });
  latest.current = { scroll, active };
  useEffect(
    () =>
      input.subscribe((event) => {
        if (
          event.type === "mouse" &&
          !event.released &&
          event.wheel &&
          latest.current.active
        )
          latest.current.scroll(event.wheel);
      }),
    [input],
  );

  useEffect(() => {
    if (active) return input.acquireMouse();
  }, [input, active]);
}
