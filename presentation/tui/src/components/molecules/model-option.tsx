import { Box } from "ink";
import type { Model, ModelEffort } from "../../types/model.js";
import { TerminalText } from "../atoms/terminal-text.js";

export function ModelOption({
  model,
  effort,
  selected,
  active,
}: {
  model: Model;
  effort: ModelEffort | null;
  selected: boolean;
  active: boolean;
}) {
  const index = effort ? model.efforts.indexOf(effort) : -1;
  const bars = model.efforts
    .map((_, position) => (position <= index ? "■" : "□"))
    .join("")
    .padEnd(4);
  return (
    <Box>
      <Box flexGrow={1} flexShrink={1} minWidth={0}>
        <TerminalText color={selected ? "cyan" : undefined} wrap="truncate-end">
          {`${selected ? "›" : " "} ${model.name}${model.blocked ? " [blocked]" : ""}${active ? " ✓" : ""}`}
        </TerminalText>
      </Box>
      <Box width={22} flexShrink={0} justifyContent="flex-end">
        <TerminalText
          color={selected ? "cyan" : undefined}
          dimColor={!selected}
          wrap="truncate-end"
        >
          {effort
            ? `${selected ? "← " : "  "}${bars} ${effort.replace("_", " ").padEnd(10)}${selected ? " →" : "  "}`
            : "No effort"}
        </TerminalText>
      </Box>
    </Box>
  );
}
