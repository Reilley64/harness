import { useCallback, useRef, useState } from "react";
import type { TranscriptViewport } from "../types/transcript-viewport.js";

const empty: TranscriptViewport = { anchor: null, expandedTools: false };

export function useTranscriptViewport(key = "initial") {
  const firstTarget = useRef<string | null>(key === "initial" ? null : key);
  if (firstTarget.current === null && key !== "initial")
    firstTarget.current = key;
  const storageKey = key === firstTarget.current ? "initial" : key;
  const [states, setStates] = useState(new Map<string, TranscriptViewport>());
  const change = useCallback(
    (update: (current: TranscriptViewport) => TranscriptViewport) => {
      setStates((current) => {
        const next = new Map(current);
        next.set(storageKey, update(current.get(storageKey) ?? empty));
        return next;
      });
    },
    [storageKey],
  );
  return { value: states.get(storageKey) ?? empty, change };
}
