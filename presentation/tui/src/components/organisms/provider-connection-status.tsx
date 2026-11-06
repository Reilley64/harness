import { useInput } from "ink";
import { useState } from "react";
import type { ProviderConnectionState } from "../../types/provider-connection.js";
import { Panel } from "../atoms/panel.js";
import { TerminalText } from "../atoms/terminal-text.js";

export function ProviderConnectionStatus({
  state,
}: {
  state: ProviderConnectionState;
}) {
  const [revealed, setRevealed] = useState<string | null>(null);
  const authorizationUrl =
    state.status === "authorizing" ? state.authorizationUrl : null;
  useInput(
    (input, key) => {
      if (
        input === "l" &&
        !key.ctrl &&
        !key.meta &&
        key.eventType !== "release"
      ) {
        setRevealed((current) =>
          current === authorizationUrl ? null : authorizationUrl,
        );
      }
    },
    { isActive: authorizationUrl !== null },
  );
  return (
    <Panel>
      <TerminalText bold>{`Connect ${state.provider.name}`}</TerminalText>
      {state.status === "starting" && (
        <TerminalText>Starting sign-in…</TerminalText>
      )}
      {state.status === "authorizing" && (
        <>
          <TerminalText>{`Continue with ${state.provider.name} in your browser.`}</TerminalText>
          {revealed === state.authorizationUrl ? (
            <TerminalText color="cyan" underline wrap="truncate-middle">
              {state.authorizationUrl}
            </TerminalText>
          ) : (
            <TerminalText dimColor>
              Sign-in link hidden: contains temporary identifiers.
            </TerminalText>
          )}
          {state.browserError && (
            <TerminalText color="red">{state.browserError}</TerminalText>
          )}
          <TerminalText dimColor>
            Waiting for authorization · o open browser · l toggle link · Esc
            cancel
          </TerminalText>
        </>
      )}
      {state.status === "starting" && (
        <TerminalText dimColor>Esc cancel</TerminalText>
      )}
      {state.status === "failed" && (
        <>
          <TerminalText color="red">{state.message}</TerminalText>
          <TerminalText dimColor>Run /auth to try again.</TerminalText>
        </>
      )}
      {state.status === "connected" && (
        <TerminalText color="green">
          {`Connected to ${state.provider.name}.`}
        </TerminalText>
      )}
    </Panel>
  );
}
