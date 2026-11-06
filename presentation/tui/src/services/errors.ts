import { RequestError } from "@agentclientprotocol/sdk";

export class PresentationError extends Error {}

const contexts = new Map([
  [-32001, "Could not load providers"],
  [-32002, "Could not create provider connection"],
  [-32003, "Could not load models"],
  [-32004, "Conversation request failed"],
  [-32005, "Could not save or restore the model selection"],
]);

export function errorMessage(error: unknown): string {
  if (error instanceof PresentationError) return error.message;
  if (error instanceof RequestError) {
    const context = contexts.get(error.code);
    if (context) {
      if (typeof error.data !== "string") return context;
      return error.code === -32004 ? error.data : `${context}: ${error.data}`;
    }
    if (error.code === -32602)
      return "The request was invalid. Check your inputs and try again.";
    if (error.code === -32800) return "The request was cancelled.";
  }
  return "The operation failed. Restart Harness or check your configuration.";
}
