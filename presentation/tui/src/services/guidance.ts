import type {
  Guidance,
  Instruction,
  InstructionSnapshot,
} from "../types/guidance.js";
import { PresentationError } from "./errors.js";

function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function text(value: unknown): string {
  if (typeof value !== "string")
    throw new PresentationError("Invalid instruction text.");
  return value;
}

function texts(value: unknown): string[] {
  if (!Array.isArray(value))
    throw new PresentationError("Invalid instruction list.");
  return value.map(text);
}

function instruction(value: unknown): Instruction {
  if (!record(value))
    throw new PresentationError("Invalid instruction provenance.");
  return {
    id: text(value.id),
    version: text(value.version),
    source: text(value.source),
    upstream: texts(value.upstream),
    requires: texts(value.requires),
    body: text(value.body),
  };
}

function snapshot(value: unknown): InstructionSnapshot | null {
  if (value === null) return null;
  if (!record(value) || !Array.isArray(value.instructions))
    throw new PresentationError("Invalid instruction snapshot.");
  return {
    id: text(value.id),
    phase: text(value.phase),
    scopes: texts(value.scopes),
    references: texts(value.references),
    instructions: value.instructions.map(instruction),
  };
}

export function parseGuidance(value: unknown): Guidance {
  if (
    !record(value) ||
    !Array.isArray(value.bundled) ||
    (value.revision !== null &&
      (typeof value.revision !== "number" ||
        !Number.isSafeInteger(value.revision) ||
        value.revision < 0))
  )
    throw new PresentationError("Invalid guidance response.");
  return {
    revision: value.revision,
    selected: snapshot(value.selected),
    current: snapshot(value.current),
    currentRun: value.currentRun === null ? null : text(value.currentRun),
    historicalUnknown: texts(value.historicalUnknown),
    available: snapshot(value.available),
    blocked: texts(value.blocked),
    overrides: texts(value.overrides),
    bundled: value.bundled.map(instruction),
  };
}
