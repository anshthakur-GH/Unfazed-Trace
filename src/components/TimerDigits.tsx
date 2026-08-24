import { useEffect, useState } from "react";
import { elapsedSeconds, formatClock } from "../lib/format";

interface TimerDigitsProps {
  totalSeconds: number;
  runningStartedAt: string | null;
  className?: string;
}

/**
 * The only component that ticks. It owns its own 1s interval and re-renders only itself —
 * the task list, header, and everything else stay untouched every second (Architecture §10:
 * "update digits, not the tree"). The interval exists only while a session is actually running.
 */
export function TimerDigits({ totalSeconds, runningStartedAt, className }: TimerDigitsProps) {
  const [nowMs, setNowMs] = useState(() => Date.now());

  useEffect(() => {
    if (!runningStartedAt) return;
    const id = window.setInterval(() => setNowMs(Date.now()), 1000);
    return () => window.clearInterval(id);
  }, [runningStartedAt]);

  const elapsed = elapsedSeconds(totalSeconds, runningStartedAt, nowMs);
  return <span className={`tabular ${className ?? ""}`}>{formatClock(elapsed)}</span>;
}
