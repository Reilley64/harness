import { Box } from "ink";
import { Panel } from "../atoms/panel.js";
import { TerminalText } from "../atoms/terminal-text.js";

export function ModelSearch({ query }: { query: string }) {
  return (
    <Box marginX={-1} flexDirection="column">
      <Panel variant="horizontal">
        <TerminalText color={query ? undefined : "gray"} wrap="truncate-start">
          {query ? `${query}▏` : "Search models"}
        </TerminalText>
      </Panel>
    </Box>
  );
}
