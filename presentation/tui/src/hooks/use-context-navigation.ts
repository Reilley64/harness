import { useStdin } from "ink";
import { useEffect, useRef } from "react";

export function useContextNavigation(
  active: boolean,
  toggle: () => void,
  griller: () => void,
  children?: () => void,
) {
  const { stdin } = useStdin();
  const latest = useRef({ active, toggle, griller, children });
  latest.current = { active, toggle, griller, children };
  useEffect(() => {
    let pending = "";
    let pasting = false;
    const shortcuts = [
      "\x1bOQ",
      "\x1bOR",
      "\x1b[12~",
      "\x1b[13~",
      "\x1b[17~",
      "\x1b[200~",
    ];
    const pasteEnd = ["\x1b[201~"];
    const input = (chunk: Buffer | string) => {
      for (const character of chunk.toString()) {
        pending += character;
        const sequences = pasting ? pasteEnd : shortcuts;
        while (
          pending &&
          !sequences.some((sequence) => sequence.startsWith(pending))
        )
          pending = pending.slice(1);
        const sequence = sequences.find((sequence) => sequence === pending);
        if (!sequence) continue;
        pending = "";
        if (sequence === "\x1b[200~") pasting = true;
        else if (sequence === "\x1b[201~") pasting = false;
        else if (latest.current.active) {
          if (sequence === "\x1bOQ" || sequence === "\x1b[12~")
            latest.current.toggle();
          else if (sequence === "\x1b[17~") latest.current.children?.();
          else latest.current.griller();
        }
      }
    };
    stdin.on("data", input);
    return () => {
      stdin.off("data", input);
    };
  }, [stdin]);
}
