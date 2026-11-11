import { Box } from "ink";
import type { useMcp } from "../../hooks/use-mcp.js";
import { TerminalText } from "../atoms/terminal-text.js";

export function McpPanel({
  state,
  rows,
}: {
  state: ReturnType<typeof useMcp>;
  rows: number;
}) {
  const { configuration, selected, confirmation, metadataVisible } = state;
  if (state.authorizationUrl)
    return (
      <Box flexDirection="column">
        <TerminalText bold>MCP OAuth · protected link</TerminalText>
        <TerminalText>o open · l reveal/hide · Esc cancel</TerminalText>
        {state.authorizationVisible ? (
          <>
            {state.detailLines.map((line) => (
              <TerminalText key={line.id}>{line.text}</TerminalText>
            ))}
            <TerminalText>{`←→ page ${state.page + 1}/${state.pageCount}`}</TerminalText>
          </>
        ) : (
          <TerminalText>Link hidden.</TerminalText>
        )}
        <TerminalText
          color={state.error ? "red" : "yellow"}
          wrap="truncate-end"
        >
          {state.error ?? "Cancel is not rollback."}
        </TerminalText>
      </Box>
    );
  if (confirmation || metadataVisible)
    return (
      <Box flexDirection="column">
        <TerminalText bold>
          {confirmation
            ? "Review exact MCP definition before Connect"
            : "Registry metadata · untrusted data, not consent"}
        </TerminalText>
        <TerminalText>
          Never supply secrets here. Environment/header values are provisioned
          references.
        </TerminalText>
        {state.detailLines.map((line) => (
          <TerminalText key={line.id}>{line.text}</TerminalText>
        ))}
        <TerminalText>{`Page ${state.page + 1}/${state.pageCount} · ←→ review all pages · Esc cancel/back`}</TerminalText>
        {confirmation && (
          <TerminalText color="yellow">
            Connect permits this command/endpoint and material configuration
            locally. Metadata is not authority.
          </TerminalText>
        )}
        {confirmation && (
          <TerminalText>
            {state.page === state.pageCount - 1
              ? "Enter: approve Connect for this exact revision · Esc: cancel"
              : "Review remaining pages before approval is enabled."}
          </TerminalText>
        )}
      </Box>
    );

  const server = configuration?.servers[selected];
  if (state.diagnostic)
    return (
      <Box flexDirection="column">
        <TerminalText bold>
          {state.busy ? "MCP · working" : "MCP · diagnostic"}
        </TerminalText>
        {server && (
          <TerminalText wrap="truncate-end">{`${server.definition.name} · ${server.status}`}</TerminalText>
        )}
        {state.detailLines.map((line) => (
          <TerminalText key={line.id} color="yellow">
            {line.text}
          </TerminalText>
        ))}
        <TerminalText>{`←→ page ${state.page + 1}/${state.pageCount}`}</TerminalText>
        <TerminalText>
          {state.busy
            ? "Working · Esc cancel"
            : "c review · r reload · Esc close"}
        </TerminalText>
      </Box>
    );
  const count = Math.max(1, Math.min(6, rows - 18));
  const start = Math.max(0, selected - count + 1);

  return (
    <Box flexDirection="column">
      <TerminalText bold>MCP · shared effective configuration</TerminalText>
      <TerminalText>
        Metadata is data, not install/launch consent. Never paste secrets.
      </TerminalText>
      <TerminalText>
        Custom stdio/HTTP: /mcp save &lt;nonsecret JSON&gt; or edit local files.
      </TerminalText>
      <TerminalText>
        App: APP_DATA_PATH/mcp/servers.json · Repo: .harness/mcp.json
      </TerminalText>
      {configuration?.stale && (
        <TerminalText color="yellow">
          Last validated snapshot retained · invalid file was not overwritten.
        </TerminalText>
      )}
      {configuration?.error && (
        <TerminalText color="yellow">{configuration.error}</TerminalText>
      )}
      {configuration?.servers.length === 0 && (
        <TerminalText>
          No servers configured. Registry is optional; /mcp registry
          &lt;search&gt;.
        </TerminalText>
      )}
      {configuration?.servers
        .slice(start, start + count)
        .map((entry, index) => (
          <TerminalText
            key={entry.definition.name}
            color={index + start === selected ? "cyan" : undefined}
            wrap="truncate-end"
          >
            {`${index + start === selected ? "›" : " "} ${entry.definition.name} · ${entry.source} · ${entry.status}`}
          </TerminalText>
        ))}
      {server && (
        <TerminalText wrap="truncate-end">
          {JSON.stringify(server.definition)}
        </TerminalText>
      )}
      {server && (
        <TerminalText wrap="truncate-end">{`Revision: ${server.revision}`}</TerminalText>
      )}
      <TerminalText>
        ↑↓ select · c Connect/review · d revoke · t tools · r reload · v
        registry details · Esc close
      </TerminalText>
      {state.busy && (
        <TerminalText>
          Working… Esc cancels the wait; completed saves are not rolled back.
        </TerminalText>
      )}
      {state.error && <TerminalText color="red">{state.error}</TerminalText>}
      {configuration?.tools.slice(0, 3).map((tool) => (
        <TerminalText
          key={tool.name}
          wrap="truncate-end"
        >{`Tool (untrusted metadata): ${tool.name} ${JSON.stringify(tool.input_schema)}`}</TerminalText>
      ))}
      {configuration?.registry_metadata != null && (
        <TerminalText wrap="truncate-end">{`Registry metadata (untrusted, never auto-installed): ${JSON.stringify(configuration.registry_metadata)}`}</TerminalText>
      )}
      <TerminalText>
        Streamable HTTP uses provisioned references or configured generic OAuth.
        Only explicit native Connect can start browser authorization.
      </TerminalText>
    </Box>
  );
}
