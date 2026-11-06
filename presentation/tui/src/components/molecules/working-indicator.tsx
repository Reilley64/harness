import { useEffect, useState } from "react";
import { useRainbowColor } from "../../hooks/use-rainbow-color.js";
import { TerminalText } from "../atoms/terminal-text.js";

type Props = {
  work: { startedAt: number; duration: number | null } | null;
  cancelling?: boolean;
};

const frames = ["·", "✢", "✳", "✶", "✻", "✽", "✻", "✶", "✳", "✢"];

function durationLabel(milliseconds: number): string {
  const seconds = Math.floor(Math.max(0, milliseconds) / 1000);
  if (seconds < 1) return `${(milliseconds / 1000).toFixed(1)}s`;
  if (seconds < 60) return `${seconds}s`;
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}

export function WorkingIndicator({ work, cancelling = false }: Props) {
  const [frame, setFrame] = useState(0);
  const startedAt = work?.duration === null ? work.startedAt : null;
  const color = useRainbowColor(startedAt !== null);
  useEffect(() => {
    if (startedAt === null) return;
    setFrame(0);
    const timer = setInterval(
      () => setFrame((frame) => (frame + 1) % frames.length),
      100,
    );
    return () => clearInterval(timer);
  }, [startedAt]);

  if (!work) return null;
  if (work.duration !== null) {
    return (
      <TerminalText color="gray" wrap="truncate-end">
        {`✻ Worked for ${durationLabel(work.duration)}`}
      </TerminalText>
    );
  }
  const elapsed = Math.max(0, performance.now() - work.startedAt);
  return (
    <TerminalText color={color} dimColor wrap="truncate-end">
      {`${frames[frame]} ${cancelling ? "Cancelling…" : "Thinking…"}${elapsed >= 1000 ? ` ${durationLabel(elapsed)}` : ""}`}
    </TerminalText>
  );
}
