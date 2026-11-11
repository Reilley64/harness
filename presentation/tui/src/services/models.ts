import type {
  Model,
  ModelEffort,
  ModelPricing,
  ModelSelection,
} from "../types/model.js";
import { modelCapabilities, modelEfforts } from "../types/model.js";
import { PresentationError } from "./errors.js";

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function price(value: unknown): number | null {
  if (value === undefined || value === null) return null;
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0)
    throw new PresentationError(
      "Invalid model price returned by the ACP agent.",
    );
  return value;
}

function parsePricing(value: unknown): ModelPricing | null {
  if (value === undefined || value === null) return null;
  if (!record(value))
    throw new PresentationError(
      "Invalid model pricing returned by the ACP agent.",
    );
  return {
    input: price(value.input),
    cachedInput: price(value.cachedInput),
    output: price(value.output),
  };
}

export function modelKey(model: Model): string {
  return JSON.stringify([model.providerId, model.id]);
}

export function parseModels(value: unknown): Model[] {
  if (!record(value) || !Array.isArray(value.models))
    throw new PresentationError(
      "Invalid model catalog returned by the ACP agent.",
    );
  const keys = new Set<string>();
  return value.models.map((entry: unknown) => {
    if (
      !record(entry) ||
      typeof entry.id !== "string" ||
      !entry.id ||
      typeof entry.name !== "string" ||
      !entry.name ||
      typeof entry.providerId !== "string" ||
      !entry.providerId
    )
      throw new PresentationError(
        "Invalid model entry returned by the ACP agent.",
      );
    const capabilities = entry.capabilities ?? [];
    if (
      !Array.isArray(capabilities) ||
      capabilities.some(
        (capability: unknown) =>
          !modelCapabilities.some((known) => known === capability),
      )
    )
      throw new PresentationError(
        "Invalid model capabilities returned by the ACP agent.",
      );
    const efforts = entry.efforts ?? [];
    if (
      !Array.isArray(efforts) ||
      efforts.some(
        (effort: unknown) => !modelEfforts.some((known) => known === effort),
      )
    )
      throw new PresentationError(
        "Invalid model efforts returned by the ACP agent.",
      );
    if (
      entry.contextWindow != null &&
      (typeof entry.contextWindow !== "number" ||
        !Number.isSafeInteger(entry.contextWindow) ||
        entry.contextWindow <= 0 ||
        entry.contextWindow > 1_000_000_000)
    )
      throw new PresentationError("Invalid model context window.");
    if (entry.blocked != null && typeof entry.blocked !== "string")
      throw new PresentationError("Invalid model availability.");
    const model: Model = {
      blocked: entry.blocked == null ? null : String(entry.blocked),
      id: entry.id,
      name: entry.name,
      providerId: entry.providerId,
      capabilities: modelCapabilities.filter((capability) =>
        capabilities.includes(capability),
      ),
      efforts: modelEfforts.filter((effort) => efforts.includes(effort)),
      pricing: parsePricing(entry.pricing),
      contextWindow:
        entry.contextWindow == null ? null : Number(entry.contextWindow),
    };
    const key = modelKey(model);
    if (keys.has(key))
      throw new PresentationError("Duplicate model returned by the ACP agent.");
    keys.add(key);
    return model;
  });
}

export function parseModelSelection(value: unknown): ModelSelection | null {
  if (!record(value))
    throw new PresentationError(
      "Invalid saved model selection returned by the agent.",
    );
  if (value.selection === null) return null;
  if (!record(value.selection))
    throw new PresentationError(
      "Invalid saved model selection returned by the agent.",
    );
  const selected = value.selection;
  const model = parseModels({ models: [selected.model] })[0];
  const effort =
    selected.effort === null
      ? null
      : modelEfforts.find((effort) => effort === selected.effort);
  if (
    !model ||
    effort === undefined ||
    (effort && !model.efforts.includes(effort))
  )
    throw new PresentationError(
      "Invalid saved model effort returned by the agent.",
    );
  return { model, effort };
}

export function filterModels(models: Model[], query: string): Model[] {
  const words = query.toLowerCase().trim().split(/\s+/).filter(Boolean);
  return models.filter((model) => {
    const text = `${model.name} ${model.id} ${model.providerId}`.toLowerCase();
    return words.every((word) => text.includes(word));
  });
}

export function defaultEffort(model: Model): ModelEffort | null {
  return model.efforts.includes("medium")
    ? "medium"
    : (model.efforts[0] ?? null);
}
