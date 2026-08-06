import { useEffect, useState } from "react";
import { BarChart3, Info } from "lucide-react";
import {
  Callout,
  EmptyState,
  Input,
  Panel,
  SettingGroup,
  SettingRow,
  Skeleton,
} from "@/components/ui";
import { cost, count, relativeTime } from "@/lib/format";
import { ipc, type UsageReport } from "@/lib/ipc";
import { useAppStore } from "@/store/useAppStore";
import type { UsagePeriod } from "@/lib/bindings/UsagePeriod";
import type { DailyUsage } from "@/lib/bindings/DailyUsage";
import { UsageChart } from "@/components/ui/UsageChart";

export function UsagePanel() {
  const settings = useAppStore((state) => state.settings);
  const update = useAppStore((state) => state.update);
  const [report, setReport] = useState<UsageReport | null>(null);
  const [daily, setDaily] = useState<DailyUsage[]>([]);

  useEffect(() => {
    void ipc.getUsage().then(setReport);
    void ipc.getDailyUsage(30).then(setDaily);
  }, []);

  if (!settings) return null;

  if (!report) {
    return (
      <Panel title="Usage & costs" description="Token consumption and estimated spend.">
        <div className="grid grid-cols-4 gap-3">
          {[0, 1, 2, 3].map((index) => (
            <Skeleton key={index} className="h-20" />
          ))}
        </div>
      </Panel>
    );
  }

  const empty = report.allTime.summary.fixes === 0;

  return (
    <Panel
      title="Usage & costs"
      description="Token consumption and estimated spend, computed at each model's own price."
    >
      {empty ? (
        <EmptyState
          icon={<BarChart3 />}
          title="No corrections recorded yet"
          description="Once you start correcting text, token counts and estimated costs appear here."
        />
      ) : (
        <>
          <div className="mb-7 grid grid-cols-4 gap-3">
            <PeriodTile label="Today" period={report.today} />
            <PeriodTile label="7 days" period={report.week} />
            <PeriodTile label="30 days" period={report.month} />
            <PeriodTile
              label="All time"
              period={report.allTime}
              footnote={report.since ? `since ${relativeTime(report.since)}` : undefined}
            />
          </div>

          <div className="mb-7">
            <UsageChart data={daily} />
          </div>

          <SettingGroup title="By model">
            <div className="px-4 py-1">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-line-subtle text-left">
                    <th className="py-2 font-medium text-muted">Model</th>
                    <th className="py-2 text-right font-medium text-muted">Fixes</th>
                    <th className="py-2 text-right font-medium text-muted">Tokens</th>
                    <th className="py-2 text-right font-medium text-muted">Cost</th>
                  </tr>
                </thead>
                <tbody>
                  {report.byModel.map((row) => (
                    <tr
                      key={`${row.provider}/${row.model}`}
                      className="border-b border-line-subtle last:border-0"
                    >
                      <td className="py-2.5">
                        <span className="block truncate text-fg">{row.model}</span>
                        <span className="text-2xs text-faint">{row.provider}</span>
                      </td>
                      <td data-numeric className="py-2.5 text-right text-muted">
                        {count(row.summary.fixes)}
                      </td>
                      <td data-numeric className="py-2.5 text-right text-muted">
                        {count(row.summary.inputTokens + row.summary.outputTokens)}
                      </td>
                      <td data-numeric className="py-2.5 text-right text-fg">
                        {row.priced ? (
                          cost(row.cost)
                        ) : (
                          <span className="text-faint">not priced</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </SettingGroup>

          <SettingGroup
            title="Pricing"
            description="Costs are only estimates, and only for models you have priced. Rates change, so these are yours to set rather than ours to guess."
          >
            {report.byModel.map((row) => (
              <PricingRow
                key={`${row.provider}/${row.model}`}
                providerSlug={row.provider}
                model={row.model}
                pricing={settings.pricing[`${row.provider}/${row.model}`]}
                onChange={(input, output) =>
                  void update({
                    pricing: {
                      ...settings.pricing,
                      [`${row.provider}/${row.model}`]: {
                        inputPerMillion: input,
                        outputPerMillion: output,
                      },
                    },
                  }).then(() => ipc.getUsage().then(setReport))
                }
              />
            ))}
          </SettingGroup>
        </>
      )}

      <Callout tone="neutral" icon={<Info />}>
        Only counts are recorded. The text you correct is never written to disk.
      </Callout>
    </Panel>
  );
}

function PeriodTile({
  label,
  period,
  footnote,
}: {
  label: string;
  period: UsagePeriod;
  footnote?: string | undefined;
}) {
  const tokens = period.summary.inputTokens + period.summary.outputTokens;

  return (
    <div className="rounded-lg border border-line-subtle bg-surface px-3.5 py-3">
      <p className="text-2xs font-medium tracking-wide text-muted uppercase">{label}</p>
      <p data-numeric className="mt-1.5 text-lg leading-none font-medium text-fg">
        {count(period.summary.fixes)}
      </p>
      <p data-numeric className="mt-1 text-2xs text-faint">
        {count(tokens)} tokens
      </p>
      <p data-numeric className="mt-0.5 text-2xs text-muted">


        {period.partialPricing && period.summary.fixes > 0 ? "partly priced" : cost(period.cost)}
      </p>
      {footnote ? <p className="mt-1 text-2xs text-faint">{footnote}</p> : null}
    </div>
  );
}

function PricingRow({
  providerSlug,
  model,
  pricing,
  onChange,
}: {
  providerSlug: string;
  model: string;
  pricing: { inputPerMillion: number; outputPerMillion: number } | undefined;
  onChange: (input: number, output: number) => void;
}) {
  const [input, setInput] = useState(String(pricing?.inputPerMillion ?? ""));
  const [output, setOutput] = useState(String(pricing?.outputPerMillion ?? ""));

  const commit = () => onChange(Number(input) || 0, Number(output) || 0);

  return (
    <SettingRow
      label={model}
      description={`${providerSlug} — price per million tokens, in your billing currency.`}
      control={
        <div className="flex items-center gap-2">
          <PriceInput label="in" value={input} onChange={setInput} onCommit={commit} />
          <PriceInput label="out" value={output} onChange={setOutput} onCommit={commit} />
        </div>
      }
    />
  );
}

function PriceInput({
  label,
  value,
  onChange,
  onCommit,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  onCommit: () => void;
}) {
  return (
    <label className="flex items-center gap-1.5">
      <span className="text-2xs text-faint">{label}</span>
      <Input
        type="number"
        step="0.01"
        min="0"
        value={value}
        onChange={(event) => onChange(event.target.value)}


        onBlur={onCommit}
        onKeyDown={(event) => event.key === "Enter" && onCommit()}
        className="w-20 text-right"
        aria-label={`${label} price per million tokens`}
      />
    </label>
  );
}
