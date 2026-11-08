import { usePaste } from "ink";
import { useCallback, useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
import { openAuthorizationUrl } from "../services/authorization.js";
import { errorMessage } from "../services/errors.js";
import { parseMcpDefinition } from "../services/mcp.js";
import type {
  McpConfiguration,
  McpOperation,
  McpServer,
} from "../types/mcp.js";

export function useMcp(
  agent: Agent,
  workspace: string,
  columns: number,
  rows: number,
) {
  const [visible, setVisible] = useState(false);
  usePaste(() => undefined, { isActive: visible });
  const [configuration, setConfiguration] = useState<McpConfiguration | null>(
    null,
  );
  const [busy, setBusy] = useState(false);
  const [authorizationUrl, setAuthorizationUrl] = useState<string | null>(null);
  const [authorizationVisible, setAuthorizationVisible] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState(0);
  const [confirmation, setConfirmation] = useState<McpServer | null>(null);
  const [page, setPage] = useState(0);
  const [metadataVisible, setMetadataVisible] = useState(false);
  const diagnostic = error ?? configuration?.error ?? null;
  const detail =
    confirmation?.definition ??
    (metadataVisible ? configuration?.registry_metadata : null);
  const content =
    authorizationVisible && authorizationUrl
      ? authorizationUrl
      : !confirmation && !metadataVisible && diagnostic
        ? diagnostic
        : JSON.stringify(detail, null, 2);
  const ascii =
    content?.replace(
      /[^\x20-\x7e\n]/g,
      (character) =>
        `\\u${character.charCodeAt(0).toString(16).padStart(4, "0")}`,
    ) ?? "";
  const width = Math.max(8, columns - 4);
  const lines = ascii
    .split("\n")
    .flatMap((line) => line.match(new RegExp(`.{1,${width}}`, "g")) ?? [""]);
  const pageSize = Math.max(1, rows - 16);
  const pageCount = Math.max(1, Math.ceil(lines.length / pageSize));
  const detailLines = lines
    .slice(page * pageSize, (page + 1) * pageSize)
    .map((text, offset) => ({ id: `${page * pageSize + offset}`, text }));
  const [dimensions, setDimensions] = useState({ columns, rows });
  if (dimensions.columns !== columns || dimensions.rows !== rows) {
    setDimensions({ columns, rows });
    setPage(0);
  }

  const pending = useRef<AbortController | null>(null);

  const run = useCallback(
    async (operation: McpOperation) => {
      pending.current?.abort();
      const controller = new AbortController();
      pending.current = controller;
      setBusy(true);
      setError(null);
      setAuthorizationUrl(null);
      setAuthorizationVisible(false);
      setPage(0);

      try {
        const result = await agent.updateMcp(
          operation,
          workspace,
          controller.signal,
          (url) => {
            if (pending.current === controller && !controller.signal.aborted)
              setAuthorizationUrl(url);
          },
        );
        if (pending.current === controller && !controller.signal.aborted)
          setConfiguration(result);
      } catch (error) {
        if (!controller.signal.aborted) setError(errorMessage(error));
      } finally {
        if (pending.current === controller) {
          pending.current = null;
          setBusy(false);
          setAuthorizationUrl(null);
          setAuthorizationVisible(false);
        }
      }
    },
    [agent, workspace],
  );

  useEffect(() => {
    setConfiguration(null);
    setConfirmation(null);
    setVisible(false);
    void run({ action: "read" });
    return () => pending.current?.abort();
  }, [run]);

  function close() {
    if (confirmation || metadataVisible) {
      setConfirmation(null);
      setMetadataVisible(false);
      setPage(0);
      return;
    }

    pending.current?.abort();
    pending.current = null;
    setBusy(false);
    setAuthorizationUrl(null);
    setAuthorizationVisible(false);
    setVisible(false);
  }

  function open(command: string) {
    setVisible(true);
    setConfirmation(null);
    setMetadataVisible(false);
    setPage(0);
    setSelected(0);
    try {
      if (command.startsWith("save ")) {
        if (!configuration)
          throw new Error("Open /mcp first to read a configuration revision.");
        void run({
          action: "save",
          expected_revision: configuration.revision,
          definition: parseMcpDefinition(JSON.parse(command.slice(5))),
        });
      } else if (command.startsWith("remove ")) {
        if (!configuration)
          throw new Error("Open /mcp first to read a configuration revision.");
        void run({
          action: "remove",
          expected_revision: configuration.revision,
          name: command.slice(7).trim(),
        });
      } else if (command.startsWith("registry ")) {
        void run({ action: "registry", query: command.slice(9).trim() });
      } else if (command === "" || command === "reload") {
        void run({ action: command === "reload" ? "reload" : "read" });
      } else {
        setError(
          "Use /mcp, /mcp reload, /mcp save <nonsecret JSON>, /mcp remove <name>, or /mcp registry <search>. Edit local files for custom definitions; no registry publication required.",
        );
      }
    } catch (error) {
      setError(errorMessage(error));
    }
  }

  function input(
    input: string,
    key: {
      escape?: boolean;
      upArrow?: boolean;
      downArrow?: boolean;
      leftArrow?: boolean;
      rightArrow?: boolean;
      return?: boolean;
      ctrl?: boolean;
      meta?: boolean;
      eventType?: "press" | "repeat" | "release";
    },
  ) {
    if (key.eventType === "release") return;
    if (key.escape) {
      close();
      return;
    }
    if (columns < 50 || rows < 18 || key.ctrl || key.meta) return;
    if (
      key.eventType === "repeat" &&
      !key.leftArrow &&
      !key.rightArrow &&
      !key.upArrow &&
      !key.downArrow
    )
      return;
    if (busy && authorizationUrl) {
      if (input === "l") {
        setAuthorizationVisible((visible) => !visible);
        setPage(0);
      }
      if (authorizationVisible && key.rightArrow)
        setPage(Math.min(pageCount - 1, page + 1));
      if (authorizationVisible && key.leftArrow) setPage(Math.max(0, page - 1));
      if (input === "o") {
        const controller = pending.current;
        void openAuthorizationUrl(authorizationUrl).catch((error) => {
          if (
            controller &&
            pending.current === controller &&
            !controller.signal.aborted
          )
            setError(errorMessage(error));
        });
      }
      return;
    }
    if (busy) return;
    if (confirmation || metadataVisible) {
      if (key.rightArrow) setPage(Math.min(pageCount - 1, page + 1));
      if (key.leftArrow) setPage(Math.max(0, page - 1));
      if (confirmation && key.return && page === pageCount - 1) {
        void run({
          action: "connect",
          name: confirmation.definition.name,
          revision: confirmation.revision,
        });
        setConfirmation(null);
      }
      return;
    }

    if (diagnostic && key.rightArrow)
      setPage(Math.min(pageCount - 1, page + 1));
    if (diagnostic && key.leftArrow) setPage(Math.max(0, page - 1));
    const servers = configuration?.servers ?? [];
    if (key.upArrow || key.downArrow)
      setSelected(
        (selected + (key.upArrow ? -1 : 1) + Math.max(1, servers.length)) %
          Math.max(1, servers.length),
      );
    const server = servers[selected];
    if (input === "r") void run({ action: "reload" });
    if (input === "v" && configuration?.registry_metadata != null) {
      setMetadataVisible(true);
      setPage(0);
    }
    if (!server) return;
    if (input === "c" && server.definition.enabled !== false) {
      setConfirmation(server);
      setPage(0);
    } else if (input === "d")
      void run({ action: "disconnect", name: server.definition.name });
    else if (input === "t")
      void run({ action: "discover", name: server.definition.name });
  }

  return {
    visible,
    configuration,
    busy,
    authorizationUrl,
    authorizationVisible,
    diagnostic,
    error,
    selected,
    confirmation,
    metadataVisible,
    detailLines,
    page,
    pageCount,
    open,
    close,
    input,
  };
}
