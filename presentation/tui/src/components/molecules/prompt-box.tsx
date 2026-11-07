import { Box, Text, useInput, usePaste, useWindowSize } from "ink";
import { useMemo, useRef, useState } from "react";
import { commands } from "../../commands.js";
import { useRainbowColor } from "../../hooks/use-rainbow-color.js";
import {
  horizontalCursor,
  promptLayout,
  verticalCursor,
} from "../../services/prompt-layout.js";
import { Panel } from "../atoms/panel.js";
import { TerminalText } from "../atoms/terminal-text.js";

type Props = {
  value: string;
  focused: boolean;
  working?: boolean;
  onChange: (value: string) => void;
  onSubmit: (value: string) => void;
  onOpenMap: () => void;
};

export function PromptBox({
  value,
  focused,
  working = false,
  onChange,
  onSubmit,
  onOpenMap,
}: Props) {
  const { columns, rows } = useWindowSize();
  const borderColor = useRainbowColor(focused || working);
  const [editor, setEditor] = useState({
    value,
    cursor: Array.from(value).length,
  });
  const characters = useMemo(() => Array.from(value), [value]);
  const width = Math.max(1, columns - 2);
  const cursor = horizontalCursor(
    value,
    editor.value === value
      ? Math.min(editor.cursor, characters.length)
      : characters.length,
    0,
    width,
  );
  const completion =
    focused && value.length > 0
      ? commands.find(
          (command) => command.startsWith(value) && command !== value,
        )
      : undefined;
  const current = useRef({ value, cursor });
  current.current = { value, cursor };
  const change = (next: string, position: number) => {
    current.current = { value: next, cursor: position };
    setEditor(current.current);
    onChange(next);
  };
  const insert = (text: string) => {
    const { value, cursor } = current.current;
    const characters = Array.from(value);
    const normalized = text.replace(/\r\n?/g, "\n");
    change(
      characters.slice(0, cursor).join("") +
        normalized +
        characters.slice(cursor).join(""),
      cursor + Array.from(normalized).length,
    );
  };
  const move = (position: number) => {
    current.current = {
      value: current.current.value,
      cursor: Math.max(
        0,
        Math.min(Array.from(current.current.value).length, position),
      ),
    };
    setEditor(current.current);
  };

  usePaste(insert, { isActive: focused });
  useInput(
    (input, key) => {
      const { value, cursor } = current.current;
      const characters = Array.from(value);
      const start =
        cursor === 0 ? 0 : characters.lastIndexOf("\n", cursor - 1) + 1;
      const nextBreak = characters.indexOf("\n", cursor);
      const end = nextBreak < 0 ? characters.length : nextBreak;
      const adjacent = (direction: number) =>
        horizontalCursor(value, cursor, direction, width);
      const completion =
        value.length > 0
          ? commands.find(
              (command) => command.startsWith(value) && command !== value,
            )
          : undefined;
      if (
        key.eventType === "release" ||
        key.escape ||
        key.pageUp ||
        key.pageDown
      )
        return;
      if (
        key.leftArrow &&
        value === "" &&
        !key.ctrl &&
        !key.meta &&
        !key.shift &&
        !key.super &&
        !key.hyper
      ) {
        onOpenMap();
        return;
      }
      if (key.return) {
        if (key.shift || key.meta) insert("\n");
        else if (key.eventType !== "repeat") onSubmit(value);
        return;
      }
      if ((key.tab || key.rightArrow) && !key.shift && completion) {
        change(completion, Array.from(completion).length);
        return;
      }
      if (key.backspace) {
        if (cursor > 0) {
          const previous = adjacent(-1);
          change(
            [
              ...characters.slice(0, previous),
              ...characters.slice(cursor),
            ].join(""),
            previous,
          );
        }
        return;
      }
      if (key.delete) {
        change(
          [
            ...characters.slice(0, cursor),
            ...characters.slice(adjacent(1)),
          ].join(""),
          cursor,
        );
        return;
      }
      if (key.ctrl) {
        if (input === "j") insert("\n");
        else if (input === "a") move(start);
        else if (input === "e") move(end);
        else if (input === "u") change("", 0);
        else if (input === "x") {
          const after = end < characters.length ? end + 1 : end;
          change(
            characters.slice(0, start).join("") +
              characters.slice(after).join(""),
            start,
          );
        }
        return;
      }
      if (key.meta) return;
      if (key.leftArrow) move(adjacent(-1));
      else if (key.rightArrow) move(adjacent(1));
      else if (key.home) move(start);
      else if (key.end) move(end);
      else if ((key.upArrow || key.downArrow) && !key.shift) {
        move(verticalCursor(value, cursor, key.upArrow ? -1 : 1, width));
      } else if (key.tab && !key.shift) insert("  ");
      else if (input && !key.upArrow && !key.downArrow) insert(input);
    },
    { isActive: focused },
  );

  const layout = useMemo(
    () => promptLayout(value, cursor, width),
    [value, cursor, width],
  );
  const height = Math.min(8, Math.max(1, Math.floor((rows - 12) / 2)));
  const first = Math.max(0, layout.row - height + 1);
  return (
    <Panel
      tone={focused || working ? "accent" : "muted"}
      variant="horizontal"
      borderColor={borderColor}
    >
      <Box minHeight={1} flexDirection="column">
        {layout.rows.slice(first, first + height).map((line) => (
          <Box key={line.id} height={1} flexShrink={0}>
            <Text wrap="truncate-end">
              {focused && line.id === layout.cursorId ? (
                <>
                  <TerminalText>{layout.before}</TerminalText>
                  <Text inverse>
                    {completion && cursor === characters.length
                      ? completion.slice(value.length, value.length + 1)
                      : layout.character}
                  </Text>
                  <TerminalText>{layout.after}</TerminalText>
                </>
              ) : (
                <TerminalText>{line.content || " "}</TerminalText>
              )}
              {completion && line.id === layout.cursorId && (
                <TerminalText color="gray" dimColor>
                  {completion.slice(
                    value.length + (cursor === characters.length ? 1 : 0),
                  )}
                </TerminalText>
              )}
            </Text>
          </Box>
        ))}
      </Box>
    </Panel>
  );
}
