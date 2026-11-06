import { useEffect, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";

export function useAgentConnection(agent: Agent): string | null {
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    void agent.initialize().catch((error) => {
      if (active) setError(`Connection failed: ${errorMessage(error)}`);
    });
    void agent.closed.then(() => {
      if (active) setError(agent.failure());
    });
    return () => {
      active = false;
    };
  }, [agent]);

  return error;
}
