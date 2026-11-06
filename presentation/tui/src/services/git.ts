import type { ExecFileOptionsWithStringEncoding } from "node:child_process";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execute = promisify(execFile);

export async function readGitBranch(
  workspace: string,
  signal: AbortSignal,
): Promise<string | null> {
  if (signal.aborted) return null;
  const env: NodeJS.ProcessEnv = { ...process.env, GIT_OPTIONAL_LOCKS: "0" };
  for (const name of [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_NAMESPACE",
  ]) {
    delete env[name];
  }
  const options: ExecFileOptionsWithStringEncoding = {
    env,
    signal,
    encoding: "utf8",
    killSignal: "SIGKILL",
    timeout: 1500,
    maxBuffer: 4096,
    windowsHide: true,
  };
  const args = ["--no-optional-locks", "-C", workspace];
  try {
    const { stdout } = await execute(
      "git",
      [...args, "symbolic-ref", "--quiet", "--short", "HEAD"],
      options,
    );
    return stdout.trim() || null;
  } catch (error) {
    if (
      signal.aborted ||
      !error ||
      typeof error !== "object" ||
      !("code" in error) ||
      error.code !== 1
    )
      return null;
  }
  try {
    const { stdout } = await execute(
      "git",
      [...args, "rev-parse", "--short", "HEAD"],
      options,
    );
    const commit = stdout.trim();
    return commit ? `detached ${commit}` : null;
  } catch {
    return null;
  }
}
