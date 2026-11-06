import stringWidth from "string-width";

const segmenter = new Intl.Segmenter(undefined, { granularity: "grapheme" });
let cached: ReturnType<typeof geometry> | undefined;

function geometry(value: string, width: number) {
  const rows = [""];
  const positions: {
    cursor: number;
    row: number;
    column: number;
    index: number;
  }[] = [];
  let row = 0;
  let column = 0;
  let cursor = 0;
  const point = () => ({
    cursor,
    row: column === width ? row + 1 : row,
    column: column === width ? 0 : column,
    index: column === width ? 0 : (rows[row]?.length ?? 0),
  });
  positions.push(point());
  for (const { segment } of segmenter.segment(value)) {
    const text = segment
      .replace(/\t/g, "    ")
      .replace(/\p{Cc}/gu, (control) => (control === "\n" ? control : ""));
    if (text === "\n") {
      rows.push("");
      row++;
      column = 0;
    } else if (segment === "\t") {
      for (let cell = 0; cell < 4; cell++) {
        if (column === width) {
          rows.push("");
          row++;
          column = 0;
        }
        rows[row] += " ";
        column++;
      }
    } else if (text) {
      const size = stringWidth(text);
      if (column > 0 && column + size > width) {
        rows.push("");
        row++;
        column = 0;
        positions[positions.length - 1] = point();
      }
      rows[row] += size > width ? "�" : text;
      column += Math.min(width, size);
    }
    cursor += Array.from(segment).length;
    positions.push(point());
  }
  return {
    value,
    width,
    positions,
    rows: rows.map((content, index) => ({
      id: `prompt-row:${index}`,
      content,
    })),
  };
}

function layout(value: string, width: number) {
  const columns = Math.max(1, width);
  if (cached?.value !== value || cached.width !== columns)
    cached = geometry(value, columns);
  return cached;
}

function positionIndex(
  positions: ReturnType<typeof geometry>["positions"],
  cursor: number,
) {
  let low = 0;
  let high = positions.length - 1;
  while (low < high) {
    const middle = Math.ceil((low + high) / 2);
    if ((positions[middle]?.cursor ?? 0) <= cursor) low = middle;
    else high = middle - 1;
  }
  return low;
}

export function horizontalCursor(
  value: string,
  cursor: number,
  direction: number,
  width: number,
): number {
  const { positions } = layout(value, width);
  const index = positionIndex(positions, cursor);
  const current = positions[index];
  const next =
    direction > 0
      ? index + 1
      : direction < 0
        ? current?.cursor === cursor
          ? index - 1
          : index
        : current?.cursor === cursor
          ? index
          : index + 1;
  return (
    positions[Math.max(0, Math.min(positions.length - 1, next))]?.cursor ?? 0
  );
}

export function verticalCursor(
  value: string,
  cursor: number,
  direction: number,
  width: number,
): number {
  const { positions } = layout(value, width);
  const current = positions[positionIndex(positions, cursor)];
  if (!current) return cursor;
  let best = cursor;
  let rowDistance = Infinity;
  let distance = Infinity;
  for (const position of positions) {
    const rows = (position.row - current.row) * direction;
    const delta = Math.abs(position.column - current.column);
    if (
      rows > 0 &&
      (rows < rowDistance || (rows === rowDistance && delta < distance))
    ) {
      best = position.cursor;
      rowDistance = rows;
      distance = delta;
    }
  }
  return best;
}

export function promptLayout(value: string, cursor: number, width: number) {
  const { rows: content, positions } = layout(value, width);
  const position = positions[positionIndex(positions, cursor)];
  const row = position?.row ?? 0;
  const index = position?.index ?? 0;
  const rows =
    row < content.length
      ? content
      : [...content, { id: `prompt-row:${row}`, content: "" }];
  const line = rows[row]?.content ?? "";
  const character =
    segmenter.segment(line.slice(index))[Symbol.iterator]().next().value
      ?.segment ?? " ";
  return {
    rows,
    row,
    cursorId: `prompt-row:${row}`,
    before: line.slice(0, index),
    character,
    after: line.slice(index + (line.slice(index) ? character.length : 0)),
  };
}
