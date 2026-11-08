import { Box, useApp, useInput, useWindowSize } from "ink";
import { useState } from "react";
import { commands } from "../commands.js";
import { TerminalText } from "../components/atoms/terminal-text.js";
import { MessageLine } from "../components/molecules/message-line.js";
import { PromptBox } from "../components/molecules/prompt-box.js";
import { StartupWordmark } from "../components/molecules/startup-wordmark.js";
import { WorkingIndicator } from "../components/molecules/working-indicator.js";
import { AgentMap } from "../components/organisms/agent-map.js";
import { ApprovedWorkPanel } from "../components/organisms/approved-work-panel.js";
import { GuidancePanel } from "../components/organisms/guidance-panel.js";
import { McpPanel } from "../components/organisms/mcp-panel.js";
import { ModelPicker } from "../components/organisms/model-picker.js";
import { ProviderConnectionStatus } from "../components/organisms/provider-connection-status.js";
import { ProviderPicker } from "../components/organisms/provider-picker.js";
import { SetupPanel } from "../components/organisms/setup-panel.js";
import { Transcript } from "../components/organisms/transcript.js";
import { HarnessLayout } from "../components/templates/harness-layout.js";
import { useAgentConnection } from "../hooks/use-agent-connection.js";
import { useApprovedWork } from "../hooks/use-approved-work.js";
import { useContextNavigation } from "../hooks/use-context-navigation.js";
import { useContextTitle } from "../hooks/use-context-title.js";
import { useConversation } from "../hooks/use-conversation.js";
import { useCreateProviderConnection } from "../hooks/use-create-provider-connection.js";
import { useGitBranch } from "../hooks/use-git-branch.js";
import { useGuidance } from "../hooks/use-guidance.js";
import { useMcp } from "../hooks/use-mcp.js";
import { useModelPicker } from "../hooks/use-model-picker.js";
import { usePromptDraft } from "../hooks/use-prompt-draft.js";
import { useProviderPicker } from "../hooks/use-provider-picker.js";
import { useSetup } from "../hooks/use-setup.js";
import { useTranscriptViewport } from "../hooks/use-transcript-viewport.js";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import { usageLabel } from "../services/usage.js";
import type { Conversation } from "../types/conversation.js";
import type { ModelSelection } from "../types/model.js";
import type { Setup } from "../types/setup.js";

export function HarnessScreen({
  agent,
  initialConversation,
  initialSelection = null,
  initialModelError = null,
  initialSetup,
}: {
  agent: Agent;
  initialConversation?: Conversation;
  initialSelection?: ModelSelection | null;
  initialModelError?: string | null;
  initialSetup: Setup;
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
  const chat = useConversation(agent, initialConversation, (next) => {
    models.restore({ model: next.model, effort: next.effort });
  });
  const guidance = useGuidance(agent, chat.conversation);
  const approved = useApprovedWork(agent, chat.conversation);
  const setup = useSetup(
    agent,
    chat.conversation,
    initialSetup,
    chat.reconcile,
  );
  const role =
    chat.conversation?.agent?.role ??
    (initialSetup.missing.length ? "setup" : "griller");
  const workspace =
    chat.conversation?.workspace ??
    agent.workspace ??
    process.env.INIT_CWD ??
    process.cwd();
  const mcp = useMcp(agent, workspace, columns, rows);
  const gitBranch = useGitBranch(workspace);
  const connection = useCreateProviderConnection(agent);
  const draft = usePromptDraft(chat.conversation?.id ?? "initial");
  const transcript = useTranscriptViewport(chat.conversation?.id ?? "initial");
  const [notice, setNotice] = useState<string | null>(null);
  const [map, setMap] = useState(false);
  const [stopConfirmation, setStopConfirmation] = useState(false);
  const modal =
    !!models.picker ||
    !!providers.picker ||
    connection.busy ||
    guidance.open ||
    approved.open ||
    mcp.visible ||
    setup.open;
  useContextNavigation(
    !modal && !stopConfirmation,
    () => {
      chat.cancelOpen();
      guidance.close();
      approved.close();
      setup.close();
      setMap((value) => !value);
    },
    () => {
      chat.cancelOpen();
      guidance.close();
      approved.close();
      if (chat.conversation?.agent?.role !== "griller") openConversation();
      else setMap(false);
    },
  );
  useContextTitle(
    map
      ? "harness - Agent map"
      : chat.conversation && !chat.conversation.agent
        ? "harness - Conversation"
        : role === "chief_of_staff"
          ? "harness - Chief of Staff"
          : role === "setup"
            ? "harness - Setup"
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
      !guidance.open &&
      !mcp.visible &&
      !approved.open &&
      !setup.open &&
      !connection.busy &&
      !models.picker &&
      !providers.picker
    ) {
      guidance.close();
      draft.recover();
      return;
    }
    if (mcp.visible) {
      if (!(key.return && key.eventType === "repeat")) mcp.input(input, key);
      return;
    }
    if (approved.open) {
      if (key.escape) approved.close();
      return;
    }
    if (setup.open) {
      if (key.escape) setup.close();
      return;
    }
    if (guidance.open) {
      if (key.escape) guidance.close();
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
      guidance.close();
      if (key.escape || key.return) setMap(false);
      else if (key.ctrl && input === "x") setStopConfirmation(true);
      return;
    }
    if (key.escape && !modal) {
      chat.cancelOpen();
      guidance.close();
      setMap(true);
      return;
    }
    if (connection.busy) {
      if (key.escape) connection.cancel();
      else if (input === "o") connection.openBrowser();
      return;
    }
    if (models.picker) {
      if (models.policyDraft !== null) {
        if (key.escape) models.cancelPolicy();
        else if (models.policySaving || models.policyReading) return;
        else if (key.return && key.eventType !== "repeat")
          void models.savePolicy();
        else if (key.ctrl && input === "u") models.changePolicy("");
        else if (key.backspace || key.delete)
          models.changePolicy([...models.policyDraft].slice(0, -1).join(""));
        else if (input && !key.ctrl && !key.meta && !key.tab && !key.return)
          models.changePolicy(models.policyDraft + input);
        return;
      }
      if (key.ctrl && input === "a") void models.editPolicy();
      else if (key.escape) models.close();
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

  function openConversation(
    id?: string,
    role: "setup" | "griller" = "griller",
  ) {
    guidance.close();
    approved.close();
    setup.close();
    setMap(false);
    void chat
      .open(id, role, models.selection)
      .catch((error) => setNotice(errorMessage(error)));
  }
  function submit(value: string) {
    chat.cancelOpen();
    const command = value.trim();
    if (!command.startsWith("/guidance")) guidance.close();
    if (command !== "/setup") setup.close();
    if (
      chat.busy &&
      command.startsWith("/") &&
      command !== "/quit" &&
      command !== "/exit" &&
      command !== "/stop" &&
      command !== "/map" &&
      command !== "/guidance" &&
      command !== "/setup"
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
    } else if (command.startsWith("/work ")) {
      draft.change("");
      setMap(false);
      void approved.inspect(command.slice(6));
    } else if (command.startsWith("/open ")) {
      draft.change("");
      openConversation(command.slice(6));
    } else if (command === "/griller" || command === "/setup chat") {
      draft.change("");
      openConversation(undefined, command === "/griller" ? "griller" : "setup");
    } else if (command === "/setup") {
      draft.change("");
      void setup.inspect();
    } else if (command === "/guidance") {
      draft.change("");
      void guidance.inspect(role === "setup" ? { phase: "setup" } : undefined);
    } else if (command.startsWith("/guidance phase ")) {
      draft.change("");
      void guidance.inspect({
        phase: command.slice("/guidance phase ".length),
      });
    } else if (command.startsWith("/guidance scope ")) {
      draft.change("");
      const scopes = [
        ...(guidance.guidance?.available?.scopes ?? []),
        command.slice("/guidance scope ".length),
      ];
      void guidance.inspect({
        phase: guidance.guidance?.available?.phase,
        references: guidance.guidance?.available?.references,
        scopes,
      });
    } else if (command.startsWith("/guidance reference ")) {
      draft.change("");
      const references = [
        ...(guidance.guidance?.available?.references ?? []),
        command.slice("/guidance reference ".length),
      ];
      void guidance.inspect({
        phase: guidance.guidance?.available?.phase,
        scopes: guidance.guidance?.available?.scopes,
        references,
      });
    } else if (command === "/guidance reset") {
      draft.change("");
      void guidance.inspect({
        phase: role === "setup" ? "setup" : "grilling",
        scopes: [],
        references: [],
      });
    } else if (command === "/guidance adopt") {
      draft.change("");
      const inspected = guidance.guidance;
      if (!inspected?.available) {
        setNotice(
          "Inspect /guidance and resolve REQUIRED blocks before explicit adoption.",
        );
      } else {
        void guidance
          .adopt(inspected, () =>
            chat.adoptGuidance(inspected, models.selection),
          )
          .catch((error) => setNotice(errorMessage(error)));
      }
    } else if (command === "/resume") {
      draft.change("");
      void chat.resume();
    } else if (command === "/resume inspect") {
      draft.change("");
      void chat.acknowledge();
    } else if (command === "/mcp" || command.startsWith("/mcp ")) {
      draft.change("");
      mcp.open(command.slice(4).trim());
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
      showPrompt={
        !models.picker &&
        !map &&
        !guidance.open &&
        !approved.open &&
        !setup.open
      }
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
          onChange={(value) => {
            guidance.close();
            draft.change(value);
          }}
          onSubmit={submit}
          onOpenMap={() => {
            chat.cancelOpen();
            guidance.close();
            approved.close();
            setup.close();
            setMap(true);
          }}
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
          {`Stop ${role === "chief_of_staff" ? "Chief of Staff" : role === "setup" ? "Setup" : "Griller"}'s active leaf run? No descendants. Completed effects remain; uncertain effects are not rolled back. Y/Enter Stop · Esc close`}
        </TerminalText>
      )}
      {mcp.visible && <McpPanel state={mcp} rows={rows} />}
      {approved.open && (
        <ApprovedWorkPanel
          key={approved.work?.revision ?? "loading"}
          work={approved.work}
          conversation={chat.conversation}
          loading={approved.loading}
          saving={approved.saving}
          error={approved.error}
          coordinator={approved.coordinator}
          columns={columns}
          rows={rows}
          approve={() => void approved.approve()}
          openCoordinator={() => {
            const id = approved.coordinator;
            if (id) openConversation(id);
          }}
        />
      )}
      {setup.open && (
        <SetupPanel
          view={setup.view}
          loading={setup.loading}
          saving={setup.saving}
          error={setup.error}
          columns={columns}
          rows={rows}
          refresh={() => void setup.inspect()}
          update={(id, confirm) => void setup.update(id, confirm)}
        />
      )}
      {guidance.open && (
        <GuidancePanel
          guidance={guidance.guidance}
          loading={guidance.loading}
          error={guidance.error}
          columns={columns}
          rows={rows}
          refresh={() => {
            void guidance.inspect();
          }}
        />
      )}
      {map &&
        !stopConfirmation &&
        !guidance.open &&
        !setup.open &&
        !approved.open && (
          <AgentMap
            conversation={chat.conversation}
            busy={chat.busy}
            open={() => {
              guidance.close();
              setMap(false);
            }}
          />
        )}
      <Box
        flexDirection="column"
        flexGrow={1}
        display={
          map ||
          stopConfirmation ||
          guidance.open ||
          approved.open ||
          mcp.visible ||
          setup.open
            ? "none"
            : "flex"
        }
      >
        {models.picker ? (
          <ModelPicker
            policyDraft={models.policyDraft}
            policyError={models.policyError}
            policySaving={models.policySaving}
            policyReading={models.policyReading}
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
            {role === "setup" && !chat.messages.length && (
              <TerminalText>
                Repository Setup · missing guidance detected. Choose /auth and
                /model, then discuss only missing outputs. /setup inspects
                durable proposals before any file confirmation.
              </TerminalText>
            )}
            {chat.messages.length === 0 && !connectionError && (
              <StartupWordmark />
            )}
            {chat.messages.length > 0 && (
              <Transcript
                viewportState={transcript.value}
                changeViewport={transcript.change}
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
            {(mcp.error || mcp.configuration?.stale) && (
              <MessageLine
                line={{
                  id: "mcp-configuration-error",
                  kind: "status",
                  tone: "error",
                  content: `${mcp.error ?? mcp.configuration?.error ?? "MCP is using its last validated snapshot."} Open /mcp to inspect and reload.`,
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
