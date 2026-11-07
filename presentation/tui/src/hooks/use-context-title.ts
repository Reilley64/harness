import { useStdout } from "ink";
import { useEffect } from "react";

export function useContextTitle(title: string) {
  const { stdout } = useStdout();
  useEffect(() => {
    stdout.write("\x1b[22;0t");
    return () => {
      stdout.write("\x1b[23;0t");
    };
  }, [stdout]);
  useEffect(() => {
    stdout.write(`\x1b]0;${title}\x07`);
  }, [stdout, title]);
}
