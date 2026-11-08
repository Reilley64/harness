import { render } from "ink";
import { HarnessScreen } from "./screens/harness-screen.js";
import { connectAgent, errorMessage } from "./services/acp.js";
import { parseArguments } from "./services/conversations.js";
import { PresentationError } from "./services/errors.js";
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
    const wasRaw = process.stdin.isRaw;
    process.stdin.setRawMode(true);
    try {
      const app = render(
        <HarnessScreen
          agent={agent}
          initialConversation={conversation}
          initialSelection={initialSelection}
          initialModelError={initialModelError}
          initialSetup={initialSetup}
        />,
        {
          alternateScreen: true,
          maxFps: 60,
          kittyKeyboard: { mode: "enabled" },
          exitOnCtrlC: false,
        },
      );
      await app.waitUntilExit();
    } finally {
      process.stdin.setRawMode(wasRaw);
    }
  } finally {
    await agent.close();
  }
} catch (error) {
  process.stderr.write(`${errorMessage(error)}\n`);
  process.exitCode = 1;
}
