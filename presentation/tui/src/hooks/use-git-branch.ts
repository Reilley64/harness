import { useEffect, useState } from "react";
import { readGitBranch } from "../services/git.js";

export function useGitBranch(workspace: string): string | null {
  const [state, setState] = useState<{
    workspace: string;
    branch: string | null;
  } | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    let timer: ReturnType<typeof setTimeout> | undefined;
    const refresh = async () => {
      const branch = await readGitBranch(workspace, controller.signal);
      if (controller.signal.aborted) return;
      setState((current) =>
        current?.workspace === workspace && current.branch === branch
          ? current
          : { workspace, branch },
      );
      timer = setTimeout(() => void refresh(), 2000);
      timer.unref();
    };
    void refresh();
    return () => {
      controller.abort();
      clearTimeout(timer);
    };
  }, [workspace]);

  return state?.workspace === workspace ? state.branch : null;
}
