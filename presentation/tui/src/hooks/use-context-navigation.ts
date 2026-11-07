import { useStdin } from "ink";
import { useEffect, useRef } from "react";

export function useContextNavigation(
  active: boolean,
  toggle: () => void,
  griller: () => void,
) {
  const { stdin } = useStdin();
  const latest = useRef({ toggle, griller });
  latest.current = { toggle, griller };
  useEffect(() => {
    if (!active) return;
    let pending = "";
    const input = (chunk: Buffer | string) => {
      pending = (pending + chunk.toString()).slice(-64);
      for (const match of pending.matchAll(/\x1b(?:O([QR])|\[(1[23])~)/g)) {
        if (match[1] === "Q" || match[2] === "12") latest.current.toggle();
        else latest.current.griller();
      }
      pending = /\x1b(?:O?|\[(?:1[23]?)?)$/.exec(pending)?.[0] ?? "";
    };
    stdin.on("data", input);
    return () => {
      stdin.off("data", input);
    };
  }, [stdin, active]);
}
