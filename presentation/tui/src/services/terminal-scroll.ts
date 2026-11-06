export const enableMouseScroll = "\x1b[?1000h\x1b[?1006h";
export const disableMouseScroll = "\x1b[?1006l\x1b[?1000l";

export function mouseScroll(input: string): {
  rows: number;
  remainder: string;
} {
  let rows = 0,
    consumed = 0;
  for (const match of input.matchAll(/\x1b\[<(\d+);(\d+);(\d+)([mM])/g)) {
    consumed = match.index + match[0].length;
    const button = Number(match[1]);
    if (match[4] === "M" && (button & 64) !== 0 && (button & 2) === 0)
      rows += (button & 1) === 0 ? -1 : 1;
  }
  return { rows, remainder: input.slice(consumed).slice(-128) };
}
