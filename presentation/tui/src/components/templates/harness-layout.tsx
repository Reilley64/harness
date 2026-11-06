import { Box } from "ink";
import type { ReactNode } from "react";
import { TerminalText } from "../atoms/terminal-text.js";

type Props = {
  children: ReactNode;
  prompt: ReactNode;
  activity?: ReactNode;
  columns: number;
  rows: number;
  selectedModel?: string;
  workspace?: string;
  gitBranch?: string | null;
  usage?: string;
  contentAlignment?: "start" | "end";
  showPrompt?: boolean;
};

export function HarnessLayout({
  children,
  prompt,
  activity,
  columns,
  rows,
  selectedModel,
  workspace,
  gitBranch,
  usage,
  contentAlignment = "start",
  showPrompt = true,
}: Props) {
  if (columns < 50 || rows < 18) {
    return <TerminalText>Please resize to at least 50 × 18.</TerminalText>;
  }

  return (
    <Box width={columns} height={rows} flexDirection="column">
      <Box
        flexGrow={1}
        minHeight={0}
        overflow="hidden"
        flexDirection="column"
        justifyContent={contentAlignment === "end" ? "flex-end" : "flex-start"}
        marginTop={1}
        paddingX={1}
      >
        {children}
      </Box>
      {showPrompt && (
        <>
          {activity && (
            <Box
              paddingX={1}
              paddingY={1}
              flexDirection="column"
              flexShrink={0}
            >
              {activity}
            </Box>
          )}
          {prompt}
          <Box
            paddingX={1}
            gap={1}
            justifyContent="space-between"
            flexShrink={0}
          >
            <Box flexGrow={1} flexShrink={1} minWidth={0} gap={1}>
              <Box flexShrink={1} minWidth={0}>
                <TerminalText dimColor wrap="truncate-start">
                  {workspace ?? ""}
                </TerminalText>
              </Box>
              {gitBranch && (
                <Box flexShrink={0} maxWidth={Math.floor(columns / 4)}>
                  <TerminalText color="gray" dimColor wrap="truncate-middle">
                    {`(${gitBranch})`}
                  </TerminalText>
                </Box>
              )}
            </Box>
            {usage && (
              <Box flexShrink={0} maxWidth={columns - 12}>
                <TerminalText dimColor wrap="truncate-end">
                  {usage}
                </TerminalText>
              </Box>
            )}
          </Box>
          <Box
            paddingX={1}
            justifyContent="space-between"
            gap={1}
            flexShrink={0}
          >
            <Box flexShrink={0}>
              <TerminalText dimColor>
                {selectedModel
                  ? columns >= 80
                    ? "Enter send · Alt+Enter newline · /"
                    : "/ commands"
                  : "Enter submit · / commands"}
              </TerminalText>
            </Box>
            {selectedModel && (
              <Box flexShrink={1} minWidth={0}>
                <TerminalText dimColor wrap="truncate-start">
                  {selectedModel}
                </TerminalText>
              </Box>
            )}
          </Box>
        </>
      )}
    </Box>
  );
}
