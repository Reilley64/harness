import type { Setup } from "../types/setup.js";
import { PresentationError } from "./errors.js";
import { parseTool } from "./tools.js";

function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
function text(value: unknown): string {
  if (typeof value !== "string")
    throw new PresentationError("Invalid Setup text.");
  return value;
}
function texts(value: unknown): string[] {
  if (!Array.isArray(value)) throw new PresentationError("Invalid Setup list.");
  return value.map(text);
}
export function parseSetup(value: unknown): Setup {
  if (
    !record(value) ||
    !Array.isArray(value.proposals) ||
    typeof value.triage !== "boolean" ||
    (value.revision !== null &&
      (!Number.isSafeInteger(value.revision) || Number(value.revision) < 0))
  )
    throw new PresentationError("Invalid Setup response.");
  return {
    workspace: text(value.workspace),
    roots: texts(value.roots),
    missing: texts(value.missing),
    triage: value.triage,
    revision: value.revision === null ? null : Number(value.revision),
    proposals: value.proposals.map((proposal: unknown) => {
      if (
        !record(proposal) ||
        !record(proposal.draft) ||
        !Array.isArray(proposal.draft.roots)
      )
        throw new PresentationError("Invalid Setup proposal.");
      const draft = proposal.draft;
      return {
        ...parseTool(proposal),
        draft: {
          workspace: text(draft.workspace),
          path: text(draft.path),
          before: draft.before === null ? null : text(draft.before),
          after: text(draft.after),
          roots: (draft.roots as unknown[]).map(
            (root: unknown): [string, string | null] => {
              if (!Array.isArray(root) || root.length !== 2)
                throw new PresentationError("Invalid Setup root.");
              return [text(root[0]), root[1] === null ? null : text(root[1])];
            },
          ),
        },
      };
    }),
  };
}
