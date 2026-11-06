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
  picker,
  query,
  models,
  selected,
  selection,
  effortFor,
  visibleOptions,
}: {
  picker: ModelPickerState;
  query: string;
  models: Model[];
  selected: number;
  selection: ModelSelection | null;
  effortFor: (model: Model) => ModelEffort | null;
  visibleOptions: number;
}) {
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
          <TerminalText dimColor>Esc close</TerminalText>
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
      </Box>
      <Box marginTop={1}>
        <TerminalText dimColor wrap="truncate-end">
          ↑↓ model · ←→ effort · Enter select · Esc close
        </TerminalText>
      </Box>
    </Box>
  );
}
