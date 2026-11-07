import { Box, useApp, useInput, useWindowSize } from "ink";
import { useState } from "react";
import { commands } from "../commands.js";
import { TerminalText } from "../components/atoms/terminal-text.js";
import { MessageLine } from "../components/molecules/message-line.js";
import { PromptBox } from "../components/molecules/prompt-box.js";
import { StartupWordmark } from "../components/molecules/startup-wordmark.js";
import { WorkingIndicator } from "../components/molecules/working-indicator.js";
import { AgentMap } from "../components/organisms/agent-map.js";
import { ModelPicker } from "../components/organisms/model-picker.js";
import { ProviderConnectionStatus } from "../components/organisms/provider-connection-status.js";
import { ProviderPicker } from "../components/organisms/provider-picker.js";
import { Transcript } from "../components/organisms/transcript.js";
import { HarnessLayout } from "../components/templates/harness-layout.js";
import { useAgentConnection } from "../hooks/use-agent-connection.js";
import { useContextNavigation } from "../hooks/use-context-navigation.js";
import { useContextTitle } from "../hooks/use-context-title.js";
import { useConversation } from "../hooks/use-conversation.js";
import { useCreateProviderConnection } from "../hooks/use-create-provider-connection.js";
import { useGitBranch } from "../hooks/use-git-branch.js";
import { useModelPicker } from "../hooks/use-model-picker.js";
import { usePromptDraft } from "../hooks/use-prompt-draft.js";
import { useProviderPicker } from "../hooks/use-provider-picker.js";
import type { Agent } from "../services/acp.js";
import { usageLabel } from "../services/usage.js";
import type { Conversation } from "../types/conversation.js";
import type { ModelSelection } from "../types/model.js";

export function HarnessScreen({
  agent,
  initialConversation,
  initialSelection = null,
  initialModelError = null,
}: {
  agent: Agent;
  initialConversation?: Conversation;
  initialSelection?: ModelSelection | null;
  initialModelError?: string | null;
}) {
  const { exit } = useApp();
  const { columns, rows } = useWindowSize();
  const connectionError = useAgentConnection(agent);
  const providers = useProviderPicker(agent);
  const models = useModelPicker(
    agent,
    initialConversation
      ? { model: initialConversation.model, effort: initialConversation.effort }
      : initialSelection,
    initialModelError,
  );
  const chat = useConversation(agent, initialConversation);
  const workspace =
    chat.conversation?.workspace ??
    agent.workspace ??
    process.env.INIT_CWD ??
    process.cwd();
  const gitBranch = useGitBranch(workspace);
  const connection = useCreateProviderConnection(agent);
  const draft = usePromptDraft();
  const [notice, setNotice] = useState<string | null>(null);
  const [map, setMap] = useState(false);
  const [stopConfirmation, setStopConfirmation] = useState(false);
  const modal = !!models.picker || !!providers.picker || connection.busy;
  useContextNavigation(
    !modal && !stopConfirmation,
    () => setMap((value) => !value),
    () => setMap(false),
  );
  useContextTitle(
    map
      ? "harness - Agent map"
      : chat.conversation && !chat.conversation.agent
        ? "harness - Conversation"
        : "harness - Griller",
  );

  useInput((input, key) => {
    if (key.eventType === "release") return;
    if (key.ctrl && (input === "q" || input === "c")) {
      exit();
      return;
    }
    if (
      key.ctrl &&
      input === "r" &&
      !connection.busy &&
      !models.picker &&
      !providers.picker
    ) {
      draft.recover();
      return;
    }
    if (stopConfirmation) {
      if (key.escape || input === "n") setStopConfirmation(false);
      else if (input === "y" || key.return) {
        setStopConfirmation(false);
        chat.cancel();
      }
      return;
    }
    if (map && !modal) {
      if (key.escape || key.return) setMap(false);
      else if (key.ctrl && input === "x") setStopConfirmation(true);
      return;
    }
    if (key.escape && !modal) {
      setMap(true);
      return;
    }
    if (connection.busy) {
      if (key.escape) connection.cancel();
      else if (input === "o") connection.openBrowser();
      return;
    }
    if (models.picker) {
      if (key.escape) models.close();
      else if (key.ctrl && input === "r") models.open();
      else if (models.picker.status === "ready") {
        if (key.upArrow || key.downArrow) models.move(key.upArrow ? -1 : 1);
        else if (key.leftArrow || key.rightArrow)
          models.changeEffort(key.leftArrow ? -1 : 1);
        else if (key.return && key.eventType !== "repeat") {
          models.select();
          setNotice(null);
        } else if (key.ctrl && input === "u") models.search("");
        else if (key.backspace || key.delete)
          models.search([...models.query].slice(0, -1).join(""));
        else if (input && !key.ctrl && !key.meta && !key.tab && !key.return)
          models.search(models.query + input);
      }
      return;
    }
    if (!providers.picker) return;
    if (key.escape) {
      providers.close();
    } else if (input === "r" && providers.picker.status !== "loading") {
      providers.open();
    } else if (key.upArrow || key.downArrow) {
      providers.move(key.upArrow ? -1 : 1);
    } else if (key.return && key.eventType !== "repeat") {
      const provider = providers.select();
      if (provider) {
        setNotice(null);
        connection.create(provider);
      }
    }
  });

  function submit(value: string) {
    const command = value.trim();
    if (
      chat.busy &&
      command.startsWith("/") &&
      command !== "/quit" &&
      command !== "/exit" &&
      command !== "/stop" &&
      command !== "/map"
    ) {
      setNotice(
        "Use /stop to stop this Griller before using commands. Escape only navigates.",
      );
      return;
    }
    if (command) {
      connection.cancel();
    }
    if (command === "/quit" || command === "/exit") {
      exit();
    } else if (command === "/stop") {
      draft.change("");
      setStopConfirmation(true);
    } else if (command === "/map") {
      draft.change("");
      setMap(true);
    } else if (command === "/resume") {
      draft.change("");
      void chat.resume();
    } else if (command === "/resume inspect") {
      draft.change("");
      void chat.acknowledge();
    } else if (command === "/auth") {
      draft.change("");
      setNotice(null);
      providers.open();
    } else if (command === "/model") {
      draft.change("");
      setNotice(null);
      models.open();
    } else if (command.startsWith("/")) {
      setNotice(
        `Unknown command: ${command}. Available commands: ${commands.join(", ")}`,
      );
    } else if (command) {
      setNotice(null);
      draft.submit(value, () => chat.send(value, models.selection));
    }
  }

  return (
    <HarnessLayout
      columns={columns}
      rows={rows}
      workspace={workspace}
      gitBranch={gitBranch}
      usage={usageLabel(
        chat.usage,
        models.selection?.model ?? chat.conversation?.model,
      )}
      contentAlignment={models.picker ? "end" : "start"}
      showPrompt={!models.picker && !map}
      selectedModel={
        models.selection
          ? `${models.selection.model.providerId}/${models.selection.model.id}${models.selection.effort ? ` · ${models.selection.effort}` : ""}`
          : undefined
      }
      activity={
        chat.work ? (
          <WorkingIndicator work={chat.work} cancelling={chat.cancelling} />
        ) : null
      }
      prompt={
        <PromptBox
          working={chat.busy}
          value={draft.value}
          onChange={draft.change}
          onSubmit={submit}
          onOpenMap={() => setMap(true)}
          focused={!modal && !map && !stopConfirmation}
        />
      }
    >
      {connectionError && (
        <TerminalText color="red" wrap="truncate-end">
          {connectionError}
        </TerminalText>
      )}
      {stopConfirmation && (
        <TerminalText>
          Stop Griller's active leaf run? No descendants. Completed effects
          remain; uncertain effects are not rolled back. Y/Enter Stop · Esc
          close
        </TerminalText>
      )}
      {map && !stopConfirmation && (
        <AgentMap
          conversation={chat.conversation}
          busy={chat.busy}
          open={() => setMap(false)}
        />
      )}
      <Box
        flexDirection="column"
        flexGrow={1}
        display={map || stopConfirmation ? "none" : "flex"}
      >
        {models.picker ? (
          <ModelPicker
            picker={models.picker}
            query={models.query}
            models={models.models}
            selected={models.selected}
            selection={models.selection}
            effortFor={models.effortFor}
            visibleOptions={Math.min(8, rows - 15)}
          />
        ) : providers.picker ? (
          <ProviderPicker
            picker={providers.picker}
            visibleOptions={rows - 17}
          />
        ) : connection.state ? (
          <ProviderConnectionStatus state={connection.state} />
        ) : (
          <>
            {chat.messages.length === 0 && !connectionError && (
              <StartupWordmark />
            )}
            {chat.messages.length > 0 && (
              <Transcript
                messages={chat.messages}
                columns={columns}
                rows={rows}
                focused={!modal && !map && !stopConfirmation}
              />
            )}
            {chat.conversation?.resumeRequired && (
              <TerminalText>
                Interrupted · /resume explicitly continues · /resume inspect
                acknowledges without inference for a new inspection instruction.
              </TerminalText>
            )}
            {chat.error && (
              <MessageLine
                line={{
                  id: "conversation-error",
                  kind: "status",
                  tone: "error",
                  content: chat.error,
                }}
              />
            )}
            {models.error && (
              <MessageLine
                line={{
                  id: "model-selection-error",
                  kind: "status",
                  tone: "error",
                  content: models.error,
                }}
              />
            )}
            {notice && (
              <TerminalText wrap="truncate-end">{notice}</TerminalText>
            )}
            {draft.notice && <TerminalText>{draft.notice}</TerminalText>}
          </>
        )}
      </Box>
    </HarnessLayout>
  );
}
