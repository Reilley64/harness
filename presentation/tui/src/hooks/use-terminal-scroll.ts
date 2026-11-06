import { useStdin, useStdout } from "ink";
import { useEffect, useRef } from "react";
import {
  disableMouseScroll,
  enableMouseScroll,
  mouseScroll,
} from "../services/terminal-scroll.js";

export function useTerminalScroll(
  scroll: (rows: number) => void,
  active: boolean,
) {
  const { stdin } = useStdin();
  const { stdout } = useStdout();
  const latest = useRef(scroll);
  latest.current = scroll;
  useEffect(() => {
    if (
      !active ||
      !("isTTY" in stdin) ||
      !stdin.isTTY ||
      !("isTTY" in stdout) ||
      !stdout.isTTY
    )
      return;
    let pending = "";
    const input = (chunk: Buffer | string) => {
      const parsed = mouseScroll(pending + chunk.toString());
      pending = parsed.remainder;
      if (parsed.rows) latest.current(parsed.rows);
    };
    stdin.on("data", input);
    stdout.write(enableMouseScroll);
    return () => {
      stdin.off("data", input);
      stdout.write(disableMouseScroll);
    };
  }, [active, stdin, stdout]);
}
