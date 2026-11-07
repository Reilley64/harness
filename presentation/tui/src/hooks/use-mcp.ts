import { useCallback, useEffect, useRef, useState } from "react";
import type { Agent } from "../services/acp.js";
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
  const [configuration, setConfiguration] = useState<McpConfiguration | null>(
    null,
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState(0);
  const [confirmation, setConfirmation] = useState<McpServer | null>(null);
  const [page, setPage] = useState(0);
  const [metadataVisible, setMetadataVisible] = useState(false);
  const detail =
    confirmation?.definition ??
    (metadataVisible ? configuration?.registry_metadata : null);
  const ascii =
    JSON.stringify(detail, null, 2)?.replace(
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
      try {
        const result = await agent.updateMcp(
          operation,
          workspace,
          controller.signal,
        );
        if (!controller.signal.aborted) setConfiguration(result);
      } catch (error) {
        if (!controller.signal.aborted) setError(errorMessage(error));
      } finally {
        if (pending.current === controller) {
          pending.current = null;
          setBusy(false);
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
    },
  ) {
    if (key.escape) {
      close();
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
