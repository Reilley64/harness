import { useEffect, useReducer, useRef } from "react";

type RejectedDraft = { id: number; value: string };
type State = {
  value: string;
  submission: number;
  rejected: RejectedDraft[];
  restored: boolean;
};
type Action =
  | { type: "edit"; value: string }
  | { type: "submit"; id: number }
  | { type: "reject"; draft: RejectedDraft }
  | { type: "recover" };

function reduce(state: State, action: Action): State {
  if (action.type === "edit")
    return { ...state, value: action.value, restored: false };
  if (action.type === "submit")
    return { ...state, value: "", submission: action.id, restored: false };
  if (action.type === "recover") {
    const [draft, ...rejected] = state.rejected;
    return state.value || !draft
      ? state
      : { ...state, value: draft.value, rejected, restored: true };
  }
  if (
    state.submission === action.draft.id &&
    state.value === "" &&
    state.rejected.length === 0
  )
    return { ...state, value: action.draft.value, restored: true };
  return {
    ...state,
    rejected: [...state.rejected, action.draft].sort((a, b) => a.id - b.id),
  };
}

const empty: State = {
  value: "",
  submission: 0,
  rejected: [],
  restored: false,
};
function reduceDrafts(
  states: Map<string, State>,
  event: { key: string; action: Action },
) {
  const next = new Map(states);
  next.set(event.key, reduce(states.get(event.key) ?? empty, event.action));
  return next;
}
export function usePromptDraft(key = "initial") {
  const firstTarget = useRef<string | null>(key === "initial" ? null : key);
  if (firstTarget.current === null && key !== "initial")
    firstTarget.current = key;
  const storageKey = key === firstTarget.current ? "initial" : key;
  const [states, update] = useReducer(reduceDrafts, new Map<string, State>());
  const state = states.get(storageKey) ?? empty;
  const dispatch = (action: Action) => update({ key: storageKey, action });
  const sequence = useRef(0);
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  function submit(value: string, send: () => Promise<{ accepted: boolean }>) {
    const id = ++sequence.current;
    dispatch({ type: "submit", id });
    void (async () => {
      let accepted = false;
      try {
        accepted = (await send()).accepted;
      } catch {
        accepted = false;
      }
      if (mounted.current && !accepted)
        dispatch({ type: "reject", draft: { id, value } });
    })();
  }

  return {
    value: state.value,
    change: (value: string) => dispatch({ type: "edit", value }),
    submit,
    recover: () => dispatch({ type: "recover" }),
    notice: state.rejected.length
      ? `${state.rejected.length} rejected draft(s) saved. ${state.value ? "Clear editor, then press" : "Press"} Ctrl+R to recover oldest; Enter sends.`
      : state.restored
        ? "Rejected submission restored to editor; Enter sends."
        : null,
  };
}
