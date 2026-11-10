import type { ReactNode } from "react";
import { createContext, useContext, useEffect, useRef, useState } from "react";
import { PresentationError } from "../services/errors.js";
import type { TerminalInput } from "../services/terminal-input.js";

const Context = createContext<TerminalInput | null>(null);

export function TerminalInputProvider({
  input,
  children,
}: {
  input: TerminalInput;
  children: ReactNode;
}) {
  const [version, setVersion] = useState(0);
  const requested = useRef(0);
  const pending = useRef<{ version: number; resolve: () => void }[]>([]);

  useEffect(() => {
    const ready = pending.current.filter((item) => item.version <= version);
    pending.current = pending.current.filter((item) => item.version > version);
    for (const item of ready) item.resolve();
  }, [version]);

  useEffect(() => {
    input.start(
      () =>
        new Promise<void>((resolve) => {
          const next = ++requested.current;
          pending.current.push({ version: next, resolve });
          setVersion(next);
        }),
    );
    return () => {
      input.close();
      for (const item of pending.current) item.resolve();
      pending.current = [];
    };
  }, [input]);

  return <Context.Provider value={input}>{children}</Context.Provider>;
}

export function useTerminalInput() {
  const input = useContext(Context);
  if (!input)
    throw new PresentationError("Terminal input owner is unavailable.");
  return input;
}
