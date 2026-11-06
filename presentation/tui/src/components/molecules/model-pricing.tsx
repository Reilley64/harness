import { Box } from "ink";
import type { Model } from "../../types/model.js";
import { TerminalText } from "../atoms/terminal-text.js";

function price(value: number | null | undefined): string {
  if (value === undefined || value === null) return "—";
  if (value > 0 && value < 0.000001) return "<$0.000001";
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 6,
  }).format(value);
}

export function ModelPricing({ model }: { model: Model | undefined }) {
  const pricing = model?.pricing;
  const available =
    pricing &&
    [pricing.input, pricing.cachedInput, pricing.output].some(
      (value) => value !== null,
    );
  return (
    <Box flexDirection="column">
      <TerminalText dimColor wrap="truncate-end">
        {available
          ? "Catalog USD / 1M tokens"
          : "Prices unavailable · USD / 1M tokens"}
      </TerminalText>
      <Box>
        {["Input", "Cached input", "Output"].map((label) => (
          <Box key={label} width="33%">
            <TerminalText dimColor>{label}</TerminalText>
          </Box>
        ))}
      </Box>
      <Box>
        {[
          { id: "input", value: pricing?.input },
          { id: "cachedInput", value: pricing?.cachedInput },
          { id: "output", value: pricing?.output },
        ].map(({ id, value }) => (
          <Box key={id} width="33%">
            <TerminalText wrap="truncate-end">{price(value)}</TerminalText>
          </Box>
        ))}
      </Box>
    </Box>
  );
}
