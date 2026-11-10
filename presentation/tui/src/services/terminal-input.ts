import { performance } from "node:perf_hooks";
import { Readable } from "node:stream";
import { StringDecoder } from "node:string_decoder";
import type { ReadStream, WriteStream } from "node:tty";
import type { InputEvent } from "ink";
import { createInputParser } from "ink";
import type { TerminalEvent } from "../types/terminal-input.js";
import { PresentationError } from "./errors.js";
import { disableMouseScroll, enableMouseScroll } from "./terminal-scroll.js";

type Frame = {
  event: InputEvent;
  startedAt: number;
  receivedAt: number;
};

export function createTerminalInput(source: ReadStream, output: WriteStream) {
  const originalRaw = source.isRaw;
  const parser = createInputParser();
  const decoder = new StringDecoder("utf8");
  const listeners = new Set<(event: TerminalEvent) => void>();
  const queue: Frame[] = [];

  let acknowledge: () => Promise<void>;
  let timeout: ReturnType<typeof setTimeout> | undefined;
  let pendingSince: number | null = null;
  let started = false;
  let closed = false;
  let pumping = false;
  let failed = false;
  let mouseUsers = 0;
  let rejectFailure: (error: unknown) => void = () => {};
  const failure = new Promise<never>((_, reject) => {
    rejectFailure = reject;
  });
  void failure.catch(() => {});

  class Input extends Readable {
    get isTTY() {
      return source.isTTY;
    }
    get isRaw() {
      return source.isRaw;
    }
    _read() {}
    setRawMode(value: boolean) {
      source.setRawMode(value);
      return this;
    }
    ref() {
      source.ref();
      return this;
    }
    unref() {
      source.unref();
      return this;
    }
  }
  const stream = new Input();
  const emit = (event: TerminalEvent) => {
    for (const listener of listeners) listener(event);
  };
  const fail = (error: unknown) => {
    if (closed || failed) return;
    failed = true;
    clearTimeout(timeout);
    source.off("data", consume);
    source.off("error", fail);
    queue.length = 0;
    rejectFailure(error);
  };
  const route = ({ event, startedAt, receivedAt }: Frame) => {
    if (typeof event !== "string") {
      stream.push(`\x1b[200~${event.paste}\x1b[201~`);
      return;
    }
    const timing = { startedAt, receivedAt };
    const mouse = /^\x1b\[<(\d+);(\d+);(\d+)([mM])$/.exec(event);
    if (mouse) {
      const button = Number(mouse[1]);
      emit({
        ...timing,
        type: "mouse",
        button,
        x: Number(mouse[2]) - 1,
        y: Number(mouse[3]) - 1,
        released: mouse[4] === "m",
        wheel:
          (button & 64) !== 0 && (button & 2) === 0
            ? (button & 1) === 0
              ? -1
              : 1
            : 0,
      });
      return;
    }
    if (event === "\x1b[I" || event === "\x1b[O") {
      emit({ ...timing, type: "focus", focused: event === "\x1b[I" });
      return;
    }
    if (
      /^\x1b(?:\]|P|_)/.test(event) ||
      /^\x1b\[[?=>][\d;:]*[cuynt]$/.test(event) ||
      /^\x1b\[\?\d+;\d+\$y$/.test(event) ||
      /^\x1b\[\d+;\d+R$/.test(event) ||
      /^\x1b\[\d+(?:;\d+)*t$/.test(event)
    ) {
      emit({ ...timing, type: "reply" });
      return;
    }
    const kitty = /^\x1b\[(57365|57366|57369)(?:;1(?::(1|2|3))?)?u$/.exec(
      event,
    );
    const action =
      ["\x1bOQ", "\x1b[12~"].includes(event) || kitty?.[1] === "57365"
        ? "toggle"
        : ["\x1bOR", "\x1b[13~"].includes(event) || kitty?.[1] === "57366"
          ? "griller"
          : event === "\x1b[17~" || kitty?.[1] === "57369"
            ? "children"
            : null;
    if (action) {
      if (!kitty?.[2] || kitty[2] === "1")
        emit({ ...timing, type: "shortcut", action });
      return;
    }
    stream.push(event === "\x1b" ? "\x1b[27u" : event);
  };
  const plain = (event: InputEvent): event is string =>
    typeof event === "string" &&
    event.length > 0 &&
    [...event].every(
      (character) =>
        (character.codePointAt(0) ?? 0) >= 32 && character !== "\x7f",
    );
  const pump = async () => {
    if (pumping || !started) return;
    pumping = true;
    try {
      while (queue.length && !closed && !failed) {
        const next = queue.shift();
        if (!next) break;
        if (plain(next.event)) {
          while (queue[0] && plain(queue[0].event)) {
            const following = queue.shift();
            if (following && typeof following.event === "string") {
              next.event += following.event;
              next.receivedAt = following.receivedAt;
            }
          }
        }
        route(next);
        await new Promise<void>((resolve) => setImmediate(resolve));
        if (!closed && !failed) await acknowledge();
      }
    } catch (error) {
      fail(error);
    } finally {
      pumping = false;
    }
  };
  const enqueue = (
    event: InputEvent,
    startedAt: number,
    receivedAt: number,
  ) => {
    if (closed || failed) return;
    if (
      typeof event === "string" &&
      event.startsWith("\x1b") &&
      event.length > 4096
    ) {
      fail(
        new PresentationError(
          "Terminal control report exceeds 4096 characters. No report tail was interpreted as editor input.",
        ),
      );
      return;
    }
    queue.push({ event, startedAt, receivedAt });
    void pump();
  };
  function consume(chunk: Buffer | string) {
    if (closed || failed) return;
    clearTimeout(timeout);
    const now = performance.now();
    const first = pendingSince ?? now;
    const text = typeof chunk === "string" ? chunk : decoder.write(chunk);
    const events = text ? parser.push(text) : [];
    for (let index = 0; index < events.length; index++)
      enqueue(events[index], index === 0 ? first : now, now);
    if (failed) return;
    if (parser.pendingControlLength() > 4096) {
      fail(
        new PresentationError(
          "Incomplete terminal control report exceeds 4096 characters. Input was not replayed as editor text.",
        ),
      );
      return;
    }
    pendingSince = parser.hasPendingEscape()
      ? events.length
        ? now
        : first
      : null;
    if (pendingSince !== null)
      timeout = setTimeout(() => {
        const event = parser.flushPendingEscape();
        const began = pendingSince ?? performance.now();
        pendingSince = null;
        if (event) enqueue(event, began, performance.now());
      }, 150);
  }

  return {
    stream,
    failure,
    start(commit: () => Promise<void>) {
      if (started || closed)
        throw new PresentationError(
          "Terminal input already owns its lifecycle.",
        );
      acknowledge = commit;
      started = true;
      source.on("data", consume);
      source.on("error", fail);
      void pump();
    },
    subscribe(listener: (event: TerminalEvent) => void) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    acquireMouse() {
      if (!closed && mouseUsers++ === 0) output.write(enableMouseScroll);
      let released = false;
      return () => {
        if (released) return;
        released = true;
        if (!closed && --mouseUsers === 0) output.write(disableMouseScroll);
      };
    },
    close() {
      if (closed) return;
      closed = true;
      clearTimeout(timeout);
      source.off("data", consume);
      source.off("error", fail);
      source.pause();
      parser.reset();
      queue.length = 0;
      listeners.clear();
      if (mouseUsers) output.write(disableMouseScroll);
      mouseUsers = 0;
      stream.destroy();
      source.setRawMode(!!originalRaw);
    },
  };
}

export type TerminalInput = ReturnType<typeof createTerminalInput>;
