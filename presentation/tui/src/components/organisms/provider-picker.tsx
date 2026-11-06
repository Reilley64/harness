import { Box } from "ink";
import type { ProviderPickerState } from "../../types/provider.js";
import { Panel } from "../atoms/panel.js";
import { TerminalText } from "../atoms/terminal-text.js";
import { ProviderOption } from "../molecules/provider-option.js";

type Props = {
  picker: ProviderPickerState;
  visibleOptions: number;
};

export function ProviderPicker({ picker, visibleOptions }: Props) {
  const count = Math.max(1, visibleOptions);
  const start =
    picker.status === "ready" ? Math.max(0, picker.selected - count + 1) : 0;

  return (
    <Panel>
      <TerminalText bold>Choose provider</TerminalText>
      <TerminalText dimColor>Select a provider to sign in.</TerminalText>
      <Box marginY={1} flexDirection="column">
        {picker.status === "loading" && (
          <TerminalText>Loading providers…</TerminalText>
        )}
        {picker.status === "failed" && (
          <TerminalText color="red" wrap="truncate-end">
            {picker.message}
          </TerminalText>
        )}
        {picker.status === "ready" && picker.providers.length === 0 && (
          <TerminalText>No providers are available.</TerminalText>
        )}
        {picker.status === "ready" &&
          picker.providers
            .slice(start, start + count)
            .map((provider, index) => (
              <ProviderOption
                key={provider.id}
                provider={provider}
                selected={picker.selected === start + index}
              />
            ))}
      </Box>
      <TerminalText dimColor>
        ↑/↓ select · Enter connect · Esc close · r retry
      </TerminalText>
    </Panel>
  );
}
