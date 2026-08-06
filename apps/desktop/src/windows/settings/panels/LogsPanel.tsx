import { useCallback, useEffect, useRef, useState } from "react";
import { ScrollText } from "lucide-react";
import { Button, EmptyState, Panel, SettingGroup, Switch } from "@/components/ui";
import { cn } from "@/lib/cn";
import { timestamp } from "@/lib/format";
import { ipc, type LogLine } from "@/lib/ipc";


const POLL_MS = 1_000;
const LIMIT = 500;


export function LogsPanel() {
  const [lines, setLines] = useState<LogLine[]>([]);
  const [follow, setFollow] = useState(true);
  const bottomRef = useRef<HTMLDivElement>(null);

  const refresh = useCallback(async () => {
    setLines(await ipc.getLogs(LIMIT));
  }, []);

  useEffect(() => {
    void refresh();
    if (!follow) return;

    const timer = setInterval(() => void refresh(), POLL_MS);
    return () => clearInterval(timer);
  }, [refresh, follow]);

  useEffect(() => {
    if (follow) bottomRef.current?.scrollIntoView({ block: "end" });
  }, [lines, follow]);

  return (
    <Panel
      title="Logs"
      description="What ZyntaxAI has been doing. Useful when something did not behave as expected."
      actions={
        <>
          <Button
            variant="secondary"
            size="md"
            onClick={async () => {
              await ipc.clearLogs();
              await refresh();
            }}
          >
            Clear
          </Button>
        </>
      }
    >
      <SettingGroup>
        <div className="flex items-center justify-between px-4 py-2.5">
          <label htmlFor="follow-logs" className="text-sm text-fg">
            Follow new entries
          </label>
          <Switch
            id="follow-logs"
            checked={follow}
            onCheckedChange={setFollow}
            aria-label="Follow new entries"
          />
        </div>
      </SettingGroup>

      {lines.length === 0 ? (
        <EmptyState
          icon={<ScrollText />}
          title="Nothing logged yet"
          description="Entries appear here as ZyntaxAI runs. Set ZYNTAX_LOG=debug for more detail."
        />
      ) : (
        <div
          data-selectable
          className="scroll-area max-h-[26rem] rounded-lg border border-line-subtle bg-inset p-2 font-mono text-xs"
        >
          {lines.map((line, index) => (
            <div
              key={`${line.timestamp}-${index}`}
              className="flex gap-2.5 rounded-sm px-1.5 py-0.5 leading-relaxed hover:bg-hover/50"
            >
              <span data-numeric className="shrink-0 text-faint">
                {timestamp(line.timestamp)}
              </span>
              <span className={cn("w-10 shrink-0 font-medium", LEVEL_TONES[line.level] ?? "text-muted")}>
                {line.level}
              </span>
              <span className="min-w-0 break-words text-fg">{line.message}</span>
            </div>
          ))}
          <div ref={bottomRef} />
        </div>
      )}
    </Panel>
  );
}


const LEVEL_TONES: Record<string, string> = {
  ERROR: "text-danger",
  WARN: "text-warning",
  INFO: "text-muted",
  DEBUG: "text-faint",
  TRACE: "text-faint",
};
