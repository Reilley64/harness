import { Box } from "ink";
import { modelKey } from "../../services/models.js";
import type {
  Model,
  ModelEffort,
  ModelPickerState,
  ModelSelection,
} from "../../types/model.js";
import { TerminalText } from "../atoms/terminal-text.js";
import { ModelOption } from "../molecules/model-option.js";
import { ModelPricing } from "../molecules/model-pricing.js";
import { ModelSearch } from "../molecules/model-search.js";

export function ModelPicker({
  policyDraft,
  policyError,
  policySaving,
  picker,
  query,
  models,
  selected,
  selection,
  effortFor,
  visibleOptions,
}: {
  policyDraft: string | null;
  policyError: string | null;
  policySaving: boolean;
  picker: ModelPickerState;
  query: string;
  models: Model[];
  selected: number;
  selection: ModelSelection | null;
  effortFor: (model: Model) => ModelEffort | null;
  visibleOptions: number;
}) {
  if (policyDraft !== null)
    return (
      <Box flexDirection="column">
        <TerminalText bold>Application-wide model allowlist</TerminalText>
        <TerminalText>
          Positive provider/model patterns, separated by commas. * matches any
          text.
        </TerminalText>
        <TerminalText>
          Broad overlapping patterns still permit a model. Remove broader
          coverage to narrow.
        </TerminalText>
        <TerminalText>
          Empty permits no inference. Authentication and worker capability are
          still required.
        </TerminalText>
        <TerminalText color="cyan" wrap="wrap">
          {policyDraft || "(empty)"}
        </TerminalText>
        {policyError && <TerminalText color="red">{policyError}</TerminalText>}
        <TerminalText dimColor>
          {policySaving
            ? "Loading / saving…"
            : "Enter save · Ctrl+U clear · Esc cancel (no change)"}
        </TerminalText>
      </Box>
    );
  const count = Math.max(1, visibleOptions);
  const start = Math.max(0, selected - count + 1);
  if (picker.status !== "ready")
    return (
      <Box flexDirection="column">
        <ModelSearch query={query} />
        <TerminalText
          color={picker.status === "failed" ? "red" : undefined}
          wrap="truncate-end"
        >
          {picker.status === "loading" ? "Loading models…" : picker.message}
        </TerminalText>
        <Box marginTop={1}>
          <TerminalText dimColor>Ctrl+A allowlist · Esc close</TerminalText>
        </Box>
      </Box>
    );
  return (
    <Box flexDirection="column">
      <ModelSearch query={query} />
      {models.length ? (
        models
          .slice(start, start + count)
          .map((model, index) => (
            <ModelOption
              key={modelKey(model)}
              model={model}
              selected={selected === start + index}
              active={
                !!selection && modelKey(selection.model) === modelKey(model)
              }
              effort={effortFor(model)}
            />
          ))
      ) : (
        <TerminalText>
          {picker.models.length
            ? "No matching models."
            : "No models are available."}
        </TerminalText>
      )}
      <Box marginTop={1} flexDirection="column">
        <ModelPricing model={models[selected]} />
        {models[selected]?.blocked && (
          <TerminalText color="yellow">
            {models[selected]?.blocked}
          </TerminalText>
        )}
      </Box>
      <Box marginTop={1}>
        <TerminalText dimColor wrap="truncate-end">
          ↑↓ model · ←→ effort · Enter select · Ctrl+A allowlist · Esc close
        </TerminalText>
      </Box>
    </Box>
  );
}
