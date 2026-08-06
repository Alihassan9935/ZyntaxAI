import { useState } from "react";
import { cn } from "@/lib/cn";
import { count } from "@/lib/format";
import type { DailyUsage } from "@/lib/bindings/DailyUsage";


export function UsageChart({ data }: { data: DailyUsage[] }) {
  const [hovered, setHovered] = useState<number | null>(null);

  const peak = Math.max(...data.map((d) => d.tokens), 0);

  const ceiling = niceCeiling(peak);
  const active = hovered === null ? null : data[hovered];

  if (data.length === 0) return null;

  return (
    <figure className="m-0">
      <figcaption className="mb-3 flex items-baseline justify-between gap-4">
        <span className="text-xs font-medium tracking-wide text-muted uppercase">
          Tokens per day
        </span>
        <span data-numeric className="text-2xs text-faint">
          last {data.length} days
        </span>
      </figcaption>

      <div className="relative rounded-lg border border-line-subtle bg-surface px-3 pt-3 pb-2">


        <div className="relative h-40 pl-10">
          {[ceiling, ceiling / 2, 0].map((value, index) => (
            <div
              key={value}
              className="absolute inset-x-0 flex items-center gap-2"
              style={{ top: `${index * 50}%`, left: 0 }}
            >
              <span
                data-numeric
                className="w-9 shrink-0 text-right text-2xs leading-none text-faint"
              >
                {count(value)}
              </span>

              <span className="h-px flex-1 bg-line-subtle" />
            </div>
          ))}

          <div className="absolute inset-0 left-10 flex items-end gap-[2px]">
            {data.map((day, index) => {
              const height = ceiling === 0 ? 0 : (day.tokens / ceiling) * 100;
              return (
                <button
                  key={day.day}
                  type="button"
                  onMouseEnter={() => setHovered(index)}
                  onMouseLeave={() => setHovered(null)}
                  onFocus={() => setHovered(index)}
                  onBlur={() => setHovered(null)}
                  aria-label={`${formatDay(day.day)}: ${day.tokens} tokens, ${day.fixes} corrections`}


                  className="group relative flex h-full max-w-6 flex-1 items-end"
                >


                  {day.tokens > 0 ? (
                    <span
                      className={cn(
                        "w-full rounded-t-[4px] transition-colors duration-fast ease-out",
                        hovered === index ? "bg-accent-hover" : "bg-accent",
                      )}
                      style={{ height: `max(2px, ${height}%)` }}
                    />
                  ) : null}
                </button>
              );
            })}
          </div>
        </div>

        <div className="mt-2 flex justify-between pl-10 text-2xs text-faint">
          <span>{formatDay(data[0]!.day)}</span>
          <span>{formatDay(data[data.length - 1]!.day)}</span>
        </div>


        <div
          className={cn(
            "pointer-events-none absolute top-2 right-3 rounded-md border px-2.5 py-1.5",
            "border-line bg-raised text-2xs shadow-popover",
            "transition-opacity duration-fast ease-out",
            active ? "opacity-100" : "opacity-0",
          )}
        >
          {active ? (
            <>
              <span className="block text-fg">{formatDay(active.day)}</span>
              <span data-numeric className="block text-muted">
                {count(active.tokens)} tokens &middot; {active.fixes}{" "}
                {active.fixes === 1 ? "correction" : "corrections"}
              </span>
            </>
          ) : (
            <span>&nbsp;</span>
          )}
        </div>
      </div>
    </figure>
  );
}


function niceCeiling(peak: number): number {
  if (peak <= 0) return 0;
  const magnitude = 10 ** Math.floor(Math.log10(peak));
  return Math.ceil(peak / magnitude) * magnitude;
}

function formatDay(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toLocaleDateString(undefined, {
    day: "numeric",
    month: "short",
  });
}
