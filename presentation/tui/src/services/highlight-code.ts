import { common, createLowlight } from "lowlight";
import { syntaxColors } from "../theme.js";
import type { ToolCall } from "../types/tool.js";
import type { TranscriptSpan } from "../types/transcript-span.js";

const highlighter = createLowlight(common);
const languages: Record<string, string> = {
  c: "c",
  h: "c",
  cc: "cpp",
  cpp: "cpp",
  hpp: "cpp",
  cs: "csharp",
  css: "css",
  go: "go",
  html: "xml",
  htm: "xml",
  java: "java",
  js: "javascript",
  jsx: "javascript",
  cjs: "javascript",
  mjs: "javascript",
  json: "json",
  kt: "kotlin",
  lua: "lua",
  md: "markdown",
  php: "php",
  py: "python",
  rb: "ruby",
  rs: "rust",
  sh: "bash",
  bash: "bash",
  sql: "sql",
  swift: "swift",
  toml: "ini",
  ts: "typescript",
  tsx: "typescript",
  xml: "xml",
  yaml: "yaml",
  yml: "yaml",
};

const cache = new WeakMap<
  ToolCall,
  {
    content: string;
    language: string;
    rows: TranscriptSpan[][];
  }
>();

export function highlightCode(
  call: ToolCall,
  content: string,
  path: string | undefined,
): TranscriptSpan[][] | undefined {
  if (call.status !== "succeeded" || content.length > 65_536) return;
  const language =
    call.name === "edit"
      ? "diff"
      : ["read", "write"].includes(call.name)
        ? languages[path?.split(".").at(-1)?.toLowerCase() ?? ""]
        : undefined;
  if (!language || !highlighter.registered(language)) return;
  const previous = cache.get(call);
  if (previous?.content === content && previous.language === language)
    return previous.rows;
  try {
    const tree = highlighter.highlight(language, content);
    const rows: TranscriptSpan[][] = [[]];
    let offset = 0;
    const visit = (
      node: (typeof tree.children)[number],
      inherited?: string,
    ) => {
      if (node.type === "text") {
        const parts = node.value.split("\n");
        parts.forEach((part, index) => {
          if (index > 0) {
            rows.push([]);
            offset = 0;
          }
          if (part) {
            rows[rows.length - 1]?.push({
              offset,
              content: part,
              color: inherited,
            });
            offset += part.length;
          }
        });
      } else if (node.type === "element") {
        const classes = node.properties.className;
        const color = Array.isArray(classes)
          ? classes
              .map((name) => syntaxColors[String(name).replace(/^hljs-/, "")])
              .find(Boolean)
          : undefined;
        node.children.forEach((child) => {
          visit(child, color ?? inherited);
        });
      }
    };
    tree.children.forEach((node) => {
      visit(node);
    });
    cache.set(call, { content, language, rows });
    return rows;
  } catch {
    return;
  }
}
