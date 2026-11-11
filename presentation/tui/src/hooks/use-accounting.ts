import { useEffect, useRef, useState } from "react";
import { accountingLabel } from "../services/accounting.js";
import type { Agent } from "../services/acp.js";
import { errorMessage, PresentationError } from "../services/errors.js";
import type { Conversation } from "../types/conversation.js";

export function useAccounting(
  agent: Agent,
  conversation: Conversation | null | undefined,
  reconcile: (conversation: Conversation) => void,
) {
  const [notice, setNotice] = useState<string | null>(null);
  const current = useRef(conversation);
  current.current = conversation;
  const mounted = useRef(true);
  useEffect(
    () => () => {
      mounted.current = false;
    },
    [],
  );
  const serial = useRef(0);
  const selectedId = conversation?.id;
  useEffect(() => {
    if (current.current?.id !== selectedId) return;
    setNotice(null);
    serial.current += 1;
  }, [selectedId]);
  return {
    notice,
    clear: () => setNotice(null),
    read: async (runId: string | null, workId: string | null) => {
      const target = current.current;
      const request = ++serial.current;
      try {
        if (!target)
          throw new PresentationError("Open an existing conversation first.");
        const result = await agent.readUsage(
          workId ? null : target.id,
          runId,
          workId,
        );
        if (
          mounted.current &&
          request === serial.current &&
          current.current?.id === target.id
        )
          setNotice(accountingLabel(result));
      } catch (error) {
        if (
          mounted.current &&
          request === serial.current &&
          current.current?.id === target?.id
        )
          setNotice(errorMessage(error));
      }
    },
    update: async (additional: number | null) => {
      const target = current.current;
      try {
        if (!target)
          throw new PresentationError("Open the waiting conversation first.");
        const inspected = await agent.viewConversation(target.id);
        if (!mounted.current || current.current?.id !== target.id) return;
        const result = await agent.updateProviderRetry(
          target.id,
          inspected.revision,
          additional,
        );
        if (
          mounted.current &&
          current.current?.id === target.id &&
          result.revision >= current.current.revision
        ) {
          reconcile(result);
          setNotice(
            additional === null
              ? "Wait abandoned; its attempts and allowance remain retained."
              : result.resumeRequired
                ? "Retry authority saved. /resume explicitly continues; other gates still apply."
                : result.messages.at(-1)?.status === "in_progress"
                  ? "Retry authority saved; the waiting controller revalidates every gate before inference."
                  : "Retry authority saved; work remains stopped or gated. Inspect before continuing.",
          );
        }
      } catch (error) {
        if (mounted.current && current.current?.id === target?.id)
          setNotice(errorMessage(error));
      }
    },
  };
}
