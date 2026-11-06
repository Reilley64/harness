import { Box } from "ink";
import type { ReactNode } from "react";
import { accentColor, palette } from "../../theme.js";

type Props = {
  children: ReactNode;
  tone?: "accent" | "muted";
  variant?: "outlined" | "horizontal";
  borderColor?: string;
};

export function Panel({
  children,
  tone = "accent",
  variant = "outlined",
  borderColor,
}: Props) {
  return (
    <Box
      borderStyle={variant === "horizontal" ? "single" : "round"}
      borderLeft={variant === "outlined"}
      borderRight={variant === "outlined"}
      borderColor={
        borderColor ?? (tone === "accent" ? accentColor : palette.borderMuted)
      }
      borderDimColor={variant === "horizontal"}
      flexDirection="column"
      flexShrink={0}
      paddingX={1}
    >
      {children}
    </Box>
  );
}
