import type { TranscriptTone } from "./types/transcript.js";

export const palette = {
  accent: "#a798d7",
  text: "#dee0e1",
  muted: "#9da5a9",
  borderMuted: "#768186",
  success: "#68b78d",
  error: "#ea7f81",
  warning: "#cd9a22",
  blue: "#69add0",
  cyan: "#5db3ba",
  orange: "#de8d5a",
  userBackground: "#213b49",
  toolPendingBackground: "#34383a",
  toolSuccessBackground: "#254131",
  toolErrorBackground: "#5b282a",
};

export const accentColor = palette.accent;

export const foregroundColors: Record<string, string> = {
  gray: palette.muted,
  grey: palette.muted,
  white: palette.text,
  red: palette.error,
  green: palette.success,
  yellow: palette.warning,
  blue: palette.blue,
  cyan: palette.cyan,
  magenta: palette.accent,
};

export const rainbowPeriod = 180_000;
const rainbow = [
  palette.accent,
  palette.error,
  palette.orange,
  palette.warning,
  palette.success,
  palette.cyan,
  palette.blue,
];

export function rainbowColor(milliseconds: number): string {
  const position =
    ((((milliseconds % rainbowPeriod) + rainbowPeriod) % rainbowPeriod) /
      rainbowPeriod) *
    rainbow.length;
  const index = Math.floor(position);
  const from = rainbow[index] ?? accentColor;
  const to = rainbow[(index + 1) % rainbow.length] ?? accentColor;
  const fraction = position - index;
  return `#${[1, 3, 5]
    .map((offset) => {
      const start = Number.parseInt(from.slice(offset, offset + 2), 16);
      const end = Number.parseInt(to.slice(offset, offset + 2), 16);
      return Math.round(start + (end - start) * fraction)
        .toString(16)
        .padStart(2, "0");
    })
    .join("")}`;
}

export const syntaxColors: Record<string, string> = {
  keyword: palette.blue,
  literal: palette.success,
  built_in: palette.accent,
  string: palette.orange,
  regexp: palette.orange,
  number: palette.success,
  comment: palette.muted,
  quote: palette.muted,
  title: palette.warning,
  type: palette.accent,
  attr: palette.cyan,
  attribute: palette.cyan,
  tag: palette.blue,
  meta: palette.muted,
  addition: palette.success,
  deletion: palette.error,
};

export const messageSections: Record<
  TranscriptTone,
  {
    backgroundColor?: string;
    color?: string;
  }
> = {
  user: { backgroundColor: palette.userBackground, color: palette.text },
  assistant: { color: palette.text },
  pending: { color: palette.muted },
  warning: { backgroundColor: "#4e2f1b", color: palette.warning },
  error: { backgroundColor: palette.toolErrorBackground, color: palette.error },
  toolPending: {
    backgroundColor: palette.toolPendingBackground,
    color: palette.muted,
  },
  toolSuccess: {
    backgroundColor: palette.toolSuccessBackground,
    color: palette.muted,
  },
  toolError: {
    backgroundColor: palette.toolErrorBackground,
    color: palette.muted,
  },
  space: {},
};
