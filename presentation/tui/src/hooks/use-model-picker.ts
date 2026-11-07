import { useCallback, useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import { defaultEffort, filterModels, modelKey } from "../services/models.js";
import type {
  Model,
  ModelEffort,
  ModelPickerState,
  ModelSelection,
} from "../types/model.js";

export function useModelPicker(
  agent: Agent,
  initialSelection: ModelSelection | null = null,
  initialError: string | null = null,
) {
  const [picker, setPicker] = useState<ModelPickerState | null>(null);
  const [query, setQuery] = useState("");
  const [policyDraft, setPolicyDraft] = useState<string | null>(null);
  const [policyError, setPolicyError] = useState<string | null>(null);
  const [policySaving, setPolicySaving] = useState(false);
  const [policyReading, setPolicyReading] = useState(false);
  const policyRequest = useRef(0);
  const policyLoading = useRef<AbortController | null>(null);
  const cancelPolicyRead = useCallback(() => {
    policyLoading.current?.abort();
    policyLoading.current = null;
    policyRequest.current++;
  }, []);
  const [error, setError] = useState<string | null>(initialError);
  const mounted = useRef(true);
  const [selected, setSelected] = useState(0);
  const [efforts, setEfforts] = useState<Record<string, ModelEffort>>({});
  const [selection, setSelection] = useState<ModelSelection | null>(
    initialSelection,
  );
  const loading = useRef<AbortController | null>(null);
  const committed = useRef<ModelSelection | null>(initialSelection);

  useEffect(() => {
    let active = true;
    mounted.current = true;
    void agent.closed.then(() => {
      if (!active) return;
      loading.current?.abort();
      cancelPolicyRead();
      setPolicyReading(false);
      setPolicyDraft(null);
      setPicker((current) =>
        current ? { status: "failed", message: agent.failure() } : null,
      );
    });
    return () => {
      active = false;
      mounted.current = false;
      loading.current?.abort();
      cancelPolicyRead();
    };
  }, [agent, cancelPolicyRead]);

  const open = useCallback(() => {
    loading.current?.abort();
    cancelPolicyRead();
    setPolicyReading(false);
    const request = new AbortController();
    loading.current = request;
    setQuery("");
    setSelected(0);
    setEfforts({});
    setPolicyDraft(null);
    setPicker({ status: "loading" });
    void agent.listModels(request.signal).then(
      (models) => {
        if (request.signal.aborted) return;
        const previous = committed.current;
        const index = previous
          ? models.findIndex(
              (model) => modelKey(model) === modelKey(previous.model),
            )
          : -1;
        setSelected(Math.max(0, index));
        if (previous?.effort)
          setEfforts({ [modelKey(previous.model)]: previous.effort });
        setPicker({ status: "ready", models });
      },
      (error) => {
        if (!request.signal.aborted)
          setPicker({ status: "failed", message: errorMessage(error) });
      },
    );
  }, [agent, cancelPolicyRead]);

  const close = useCallback(() => {
    loading.current?.abort();
    cancelPolicyRead();
    setPolicyReading(false);
    setPolicyDraft(null);
    setPicker(null);
  }, [cancelPolicyRead]);

  const models =
    picker?.status === "ready" ? filterModels(picker.models, query) : [];
  const highlighted = models[selected];

  function search(value: string) {
    setQuery(value.replace(/[\u0000-\u001f\u007f-\u009f]/g, ""));
    setSelected(0);
  }

  function effortFor(model: Model): ModelEffort | null {
    const effort = efforts[modelKey(model)];
    return effort && model.efforts.includes(effort)
      ? effort
      : defaultEffort(model);
  }

  function move(direction: -1 | 1) {
    setSelected((current) =>
      Math.max(0, Math.min(models.length - 1, current + direction)),
    );
  }

  function changeEffort(direction: -1 | 1) {
    if (!highlighted?.efforts.length) return;
    setEfforts((previous) => {
      const key = modelKey(highlighted);
      const saved = previous[key];
      const current =
        saved && highlighted.efforts.includes(saved)
          ? saved
          : defaultEffort(highlighted);
      const index = current ? highlighted.efforts.indexOf(current) : 0;
      const effort =
        highlighted.efforts[
          Math.max(
            0,
            Math.min(highlighted.efforts.length - 1, index + direction),
          )
        ];
      return { ...previous, [key]: effort };
    });
  }

  function select(): ModelSelection | undefined {
    if (!highlighted) return;
    if (highlighted.blocked) {
      setError(highlighted.blocked);
      return;
    }
    const next = { model: highlighted, effort: effortFor(highlighted) };
    committed.current = next;
    setSelection(next);
    setError(null);
    close();
    void agent.updateModelSelection(next).then(
      (saved) => {
        if (mounted.current && committed.current === next) {
          committed.current = saved;
          setSelection(saved);
        }
      },
      (error) => {
        if (mounted.current && committed.current === next)
          setError(
            `Could not remember the selected model: ${errorMessage(error)}`,
          );
      },
    );
    return next;
  }

  async function editPolicy() {
    cancelPolicyRead();
    const request = policyRequest.current;
    const controller = new AbortController();
    policyLoading.current = controller;
    setPolicyDraft("");
    setPolicyReading(true);
    setPolicyError(null);
    try {
      const patterns = await agent.modelAllowlist(undefined, controller.signal);
      if (mounted.current && request === policyRequest.current)
        setPolicyDraft(patterns.join(", "));
    } catch (error) {
      if (mounted.current && request === policyRequest.current)
        setPolicyError(errorMessage(error));
    } finally {
      if (mounted.current && request === policyRequest.current) {
        policyLoading.current = null;
        setPolicyReading(false);
      }
    }
  }

  async function savePolicy() {
    if (policyDraft === null || policySaving || policyReading) return;
    setPolicySaving(true);
    setPolicyError(null);
    try {
      await agent.modelAllowlist(policyDraft.split(/[\s,]+/).filter(Boolean));
      if (mounted.current) open();
    } catch (error) {
      if (mounted.current) setPolicyError(errorMessage(error));
    } finally {
      if (mounted.current) setPolicySaving(false);
    }
  }

  function cancelPolicy() {
    if (policySaving) return;
    cancelPolicyRead();
    setPolicyReading(false);
    setPolicyDraft(null);
    setPolicyError(null);
  }

  return {
    restore: (next: ModelSelection) => {
      committed.current = next;
      setSelection(next);
      setError(null);
    },
    policyDraft,
    policyError,
    policySaving,
    policyReading,
    editPolicy,
    savePolicy,
    cancelPolicy,
    changePolicy: (value: string) =>
      setPolicyDraft(value.replace(/[\u0000-\u001f\u007f-\u009f]/g, "")),
    picker,
    error,
    query,
    models,
    selected,
    highlighted,
    selection,
    open,
    close,
    search,
    move,
    effortFor,
    changeEffort,
    select,
  };
}
