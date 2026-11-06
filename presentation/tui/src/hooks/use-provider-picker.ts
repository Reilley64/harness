import { useCallback, useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import type { ProviderPickerState } from "../types/provider.js";

export function useProviderPicker(agent: Agent) {
  const [picker, setPicker] = useState<ProviderPickerState | null>(null);
  const loading = useRef<AbortController | null>(null);

  useEffect(() => {
    let active = true;
    void agent.closed.then(() => {
      if (!active) return;
      loading.current?.abort();
      setPicker((current) =>
        current ? { status: "failed", message: agent.failure() } : null,
      );
    });
    return () => {
      active = false;
      loading.current?.abort();
    };
  }, [agent]);

  const open = useCallback(() => {
    loading.current?.abort();
    const request = new AbortController();
    loading.current = request;
    setPicker({ status: "loading" });
    void agent.listProviders(request.signal).then(
      (providers) => {
        if (!request.signal.aborted)
          setPicker({ status: "ready", providers, selected: 0 });
      },
      (error) => {
        if (!request.signal.aborted)
          setPicker({ status: "failed", message: errorMessage(error) });
      },
    );
  }, [agent]);

  const close = useCallback(() => {
    loading.current?.abort();
    setPicker(null);
  }, []);

  const move = useCallback((direction: -1 | 1) => {
    setPicker((current) => {
      if (current?.status !== "ready") return current;
      return {
        ...current,
        selected: Math.max(
          0,
          Math.min(current.providers.length - 1, current.selected + direction),
        ),
      };
    });
  }, []);

  function select() {
    if (picker?.status !== "ready") return;
    const provider = picker.providers[picker.selected];
    if (provider) close();
    return provider;
  }

  return { picker, open, close, move, select };
}
