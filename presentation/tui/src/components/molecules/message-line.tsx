import { Box, Text } from "ink";
import { memo } from "react";
import { messageSections, palette } from "../../theme.js";
import type { TranscriptLine } from "../../types/transcript.js";
import { TerminalText } from "../atoms/terminal-text.js";

export const MessageLine = memo(function MessageLine({
  line,
}: {
  line: TranscriptLine;
}) {
  const section = messageSections[line.tone];
  return (
    <Box
      width="100%"
      height={1}
      flexShrink={0}
      paddingX={1}
      backgroundColor={section.backgroundColor}
    >
      <Text
        color={line.kind === "heading" ? palette.text : section.color}
        bold={line.kind === "heading" || line.kind === "status"}
        italic={line.tone === "pending"}
        wrap="truncate-end"
      >
        {line.spans?.length ? (
          line.spans.map((span) => (
            <TerminalText key={span.offset} color={span.color}>
              {span.content}
            </TerminalText>
          ))
        ) : (
          <TerminalText>{line.content || " "}</TerminalText>
        )}
      </Text>
    </Box>
  );
});
