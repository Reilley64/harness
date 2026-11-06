import { Readable } from "node:stream";

export function readableStream(source: Readable): ReadableStream<Uint8Array> {
  const reader = Readable.toWeb(source).getReader();
  return new ReadableStream<Uint8Array>({
    async pull(controller) {
      try {
        const result = await reader.read();
        if (result.done) {
          reader.releaseLock();
          controller.close();
        } else controller.enqueue(result.value);
      } catch (error) {
        reader.releaseLock();
        controller.error(error);
      }
    },
    cancel(reason) {
      return reader.cancel(reason).finally(() => reader.releaseLock());
    },
  });
}
