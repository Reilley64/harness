import { useCallback, useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { errorMessage } from "../services/acp.js";
import { openAuthorizationUrl } from "../services/authorization.js";
import type { Provider } from "../types/provider.js";
import type { ProviderConnectionState } from "../types/provider-connection.js";

export function useCreateProviderConnection(agent: Agent) {
  const [state, setState] = useState<ProviderConnectionState | null>(null);
  const pending = useRef<{ agent: Agent; controller: AbortController } | null>(
    null,
  );

  useEffect(() => {
    return () => {
      if (pending.current?.agent === agent) pending.current.controller.abort();
    };
  }, [agent]);

  const cancel = useCallback(() => {
    pending.current?.controller.abort();
    pending.current = null;
    setState(null);
  }, []);

  const create = useCallback(
    (provider: Provider) => {
      pending.current?.controller.abort();
      const request = new AbortController();
      pending.current = { agent, controller: request };
      setState({ status: "starting", provider });
      void agent
        .createProviderConnection(
          provider.id,
          (challenge) => {
            if (!request.signal.aborted) {
              setState({
                status: "authorizing",
                provider,
                authorizationUrl: challenge.authorizationUrl,
              });
            }
          },
          request.signal,
        )
        .then(
          (connection) => {
            if (!request.signal.aborted)
              setState({ status: "connected", provider, connection });
          },
          (error) => {
            if (!request.signal.aborted)
              setState({
                status: "failed",
                provider,
                message: errorMessage(error),
              });
          },
        );
    },
    [agent],
  );

  function openBrowser() {
    if (state?.status !== "authorizing") return;
    const request = pending.current?.controller;
    void openAuthorizationUrl(state.authorizationUrl).catch(() => {
      if (pending.current?.controller !== request || request?.signal.aborted)
        return;
      setState((current) =>
        current?.status === "authorizing"
          ? {
              ...current,
              browserError: "Could not open a browser. Use the sign-in link.",
            }
          : current,
      );
    });
  }

  const busy = state?.status === "starting" || state?.status === "authorizing";
  return { state, busy, create, cancel, openBrowser };
}
