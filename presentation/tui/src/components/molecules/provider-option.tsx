import type { Provider } from "../../types/provider.js";
import { TerminalText } from "../atoms/terminal-text.js";

type Props = {
  provider: Provider;
  selected: boolean;
};

export function ProviderOption({ provider, selected }: Props) {
  return (
    <TerminalText color={selected ? "cyan" : undefined}>
      {`${selected ? "› " : "  "}${provider.name} (${provider.id})`}
    </TerminalText>
  );
}
