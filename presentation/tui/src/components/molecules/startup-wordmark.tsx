import { Box } from "ink";
import { TerminalText } from "../atoms/terminal-text.js";

const lines = [
  " ███  ████  █████ █     █     █████ █   █  ███ ",
  "█     █   █   █   █     █       █   ██  █ █    ",
  "█  ██ ████    █   █     █       █   █ █ █ █  ██",
  "█   █ █  █    █   █     █       █   █  ██ █   █",
  " ███  █   █ █████ █████ █████ █████ █   █  ███ ",
];

export function StartupWordmark() {
  return (
    <Box
      flexGrow={1}
      flexDirection="column"
      alignItems="center"
      justifyContent="center"
    >
      {lines.map((line) => (
        <TerminalText key={line} color="gray" dimColor>
          {line}
        </TerminalText>
      ))}
    </Box>
  );
}
