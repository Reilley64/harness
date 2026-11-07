import { useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import type { Conversation } from "../types/conversation.js";
import type { Guidance, GuidanceQuery } from "../types/guidance.js";

export function useGuidance(agent: Agent, conversation: Conversation | null) {
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [guidance, setGuidance] = useState<Guidance | null>(null);
  const [error, setError] = useState<string | null>(null);
  const query = useRef<GuidanceQuery>({});
  const request = useRef(0);

  async function inspect(
    next: GuidanceQuery = query.current,
    id = conversation?.id,
  ) {
    const sequence = ++request.current;
    query.current = next;
    setOpen(true);
    setLoading(true);
    setGuidance(null);
    setError(null);
    try {
      const response = await agent.readGuidance(id, next);
      if (sequence === request.current) setGuidance(response);
    } catch (error) {
      if (sequence === request.current) setError(errorMessage(error));
    } finally {
      if (sequence === request.current) setLoading(false);
    }
  }

  return {
    open,
    loading,
    guidance,
    error,
    inspect,
    close: () => setOpen(false),
  };
}
