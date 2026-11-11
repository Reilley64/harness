import {
  memo,
  useCallback,
  useEffect,
  useReducer,
  useRef,
  useState,
} from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import { PresentationError } from "../services/errors.js";
import type { Conversation } from "../types/conversation.js";
import type { ModelSelection } from "../types/model.js";
import { useConversation } from "./use-conversation.js";

type Controller = ReturnType<typeof useConversation>;

type Session = { id: string; initial: Conversation };

type Store = {
  values: Map<string, Controller>;
  publish: (id: string, value: Controller) => void;
};

function useConversationRefresh(
  agent: Agent,
  chat: Controller,
  selected: boolean,
) {
  const latest = useRef(chat);
  latest.current = chat;
  const [error, setError] = useState<string | null>(null);
  const id = chat.conversation?.id;
  useEffect(() => {
    if (!id || !selected) return;
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout> | undefined;
    const refresh = async () => {
      try {
        if (!latest.current.busy) {
          const next = await agent.viewConversation(id, controller.signal);
          if (!controller.signal.aborted && !latest.current.busy) {
            latest.current.reconcile(next);
            setError(null);
          }
        }
      } catch (error) {
        if (!controller.signal.aborted) setError(errorMessage(error));
      } finally {
        if (!controller.signal.aborted) timer = setTimeout(refresh, 500);
      }
    };
    timer = setTimeout(refresh, 500);
    return () => {
      controller.abort();
      clearTimeout(timer);
    };
  }, [agent, id, selected]);
  return error;
}

const ConversationSession = memo(function ConversationSession({
  agent,
  session,
  selected,
  store,
}: {
  agent: Agent;
  session: Session;
  selected: boolean;
  store: Store;
}) {
  const chat = useConversation(agent, session.initial);
  const error = useConversationRefresh(agent, chat, selected);
  useEffect(() => {
    store.publish(session.id, { ...chat, error: chat.error ?? error });
  }, [chat, error, session.id, store]);
  return null;
});

export function useConversations(
  agent: Agent,
  initial: Conversation | undefined,
  onOpen: (next: Conversation) => void,
) {
  const root = useConversation(agent, initial);
  const [selected, setSelected] = useState<string | null>(null);
  const [opening, setOpening] = useState(false);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [, redraw] = useReducer((value: number) => value + 1, 0);
  const mounted = useRef(true);
  const navigation = useRef<AbortController | null>(null);
  const latest = useRef({ root, selected, sessions, onOpen });
  latest.current = { root, selected, sessions, onOpen };
  const store = useRef<Store>({
    values: new Map(),
    publish: (id, value) => {
      if (!mounted.current) return;
      store.current.values.set(id, value);
      redraw();
    },
  });
  const cancelOpen = useCallback(() => {
    navigation.current?.abort();
    navigation.current = null;
    if (mounted.current) setOpening(false);
  }, []);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      cancelOpen();
    };
  }, [cancelOpen]);
  const rootError = useConversationRefresh(agent, root, selected === null);

  async function open(
    id?: string,
    role: "setup" | "griller" = "griller",
    selection: ModelSelection | null = null,
  ) {
    cancelOpen();
    const controller = new AbortController();
    navigation.current = controller;
    setOpening(true);
    const owns = () =>
      mounted.current &&
      navigation.current === controller &&
      !controller.signal.aborted;
    try {
      let target = id;
      if (!target) {
        const known = [
          latest.current.root,
          ...store.current.values.values(),
        ].find((chat) => chat.conversation?.agent?.role === role)?.conversation;
        target = known?.id;
        if (!target) {
          let after: string | null = null;
          do {
            const page = await agent.readCreationPage(after, controller.signal);
            if (!owns()) return null;
            target = page.agents.find(
              (node) => node.role === role,
            )?.conversationId;
            after = page.next;
          } while (!target && after);
        }
      }
      const cached =
        target === latest.current.root.conversation?.id
          ? latest.current.root
          : target
            ? store.current.values.get(target)
            : undefined;
      const next =
        cached?.conversation ??
        (target
          ? await agent.viewConversation(target, controller.signal)
          : selection
            ? await agent.startAgent(selection, role)
            : null);
      if (!owns()) return null;
      if (!next)
        throw new PresentationError(
          "Choose a model before starting this role.",
        );
      if (next.id === latest.current.root.conversation?.id) {
        setSelected(null);
      } else {
        setSessions((prior) =>
          prior.some((session) => session.id === next.id)
            ? prior
            : [...prior, { id: next.id, initial: next }],
        );
        setSelected(next.id);
      }
      latest.current.onOpen(next);
      return next;
    } catch (error) {
      if (owns()) throw error;
      return null;
    } finally {
      if (navigation.current === controller) {
        navigation.current = null;
        if (mounted.current) setOpening(false);
      }
    }
  }

  const controller =
    selected === null ? root : store.current.values.get(selected);
  const pending = sessions.find((session) => session.id === selected)?.initial;
  const chat = controller ?? {
    ...root,
    conversation: pending ?? null,
    messages: pending?.messages ?? [],
    busy: false,
    work: null,
    error: null,
    cancel: () => {},
    send: async (draft: string) => ({ accepted: false, draft }),
  };
  return {
    ...chat,
    error: chat.error ?? (selected === null ? rootError : null),
    open,
    cancelOpen,
    loading: opening || !controller,
    sessions: sessions.map((session) => (
      <ConversationSession
        key={session.id}
        agent={agent}
        session={session}
        selected={session.id === selected}
        store={store.current}
      />
    )),
  };
}
