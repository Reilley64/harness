import type { Instance } from "ink";
import { render } from "ink";
import { TerminalInputProvider } from "./hooks/use-terminal-input.js";
import { HarnessScreen } from "./screens/harness-screen.js";
import { connectAgent, errorMessage } from "./services/acp.js";
import { parseArguments } from "./services/conversations.js";
import { PresentationError } from "./services/errors.js";
import { createTerminalInput } from "./services/terminal-input.js";
import type { ModelSelection } from "./types/model.js";

try {
  const resumeId = parseArguments(process.argv.slice(2));
  if (!process.stdin.isTTY || !process.stdout.isTTY)
    throw new PresentationError(
      "The Ink interface requires an interactive terminal.",
    );
  const agent = connectAgent();
  try {
    const conversation = resumeId
      ? await agent.loadConversation(resumeId)
      : await agent.readAgent();
    const initialSetup = await agent.readSetup(conversation?.id);
    let initialSelection: ModelSelection | null = null;
    let initialModelError: string | null = null;
    if (!conversation) {
      try {
        initialSelection = await agent.readModelSelection();
      } catch (error) {
        initialModelError = `Could not restore the last model: ${errorMessage(error)}`;
      }
    }
    const input = createTerminalInput(process.stdin, process.stdout);
    input.stream.setRawMode(true);
    let app: Instance | undefined;
    try {
      app = render(
        <TerminalInputProvider input={input}>
          <HarnessScreen
            agent={agent}
            initialConversation={conversation}
            initialSelection={initialSelection}
            initialModelError={initialModelError}
            initialSetup={initialSetup}
          />
        </TerminalInputProvider>,
        {
          stdin: input.stream,
          alternateScreen: true,
          maxFps: 60,
          kittyKeyboard: { mode: "enabled" },
          exitOnCtrlC: false,
        },
      );
      await Promise.race([app.waitUntilExit(), input.failure]);
    } finally {
      app?.unmount();
      input.close();
    }
  } finally {
    await agent.close();
  }
} catch (error) {
  process.stderr.write(`${errorMessage(error)}\n`);
  process.exitCode = 1;
}
