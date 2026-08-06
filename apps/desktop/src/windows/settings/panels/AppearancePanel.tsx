import { useState } from "react";
import { Panel, Select, SettingGroup, SettingRow, Slider } from "@/components/ui";
import { useAppStore } from "@/store/useAppStore";
import type { Theme } from "@/lib/bindings/Theme";

const THEMES = [
  { value: "system", label: "Match my system" },
  { value: "dark", label: "Dark" },
  { value: "light", label: "Light" },
];


const OPACITY_MIN = 40;

export function AppearancePanel() {
  const settings = useAppStore((state) => state.settings);
  const update = useAppStore((state) => state.update);


  const [dragging, setDragging] = useState<number | null>(null);

  if (!settings) return null;
  const { appearance } = settings;
  const shown = dragging ?? appearance.opacity;

  return (
    <Panel title="Appearance" description="How ZyntaxAI looks on your desktop.">
      <SettingGroup title="Theme">
        <SettingRow
          label="Colour scheme"
          control={
            <Select
              value={appearance.theme}
              onValueChange={(theme) =>
                void update({ appearance: { ...appearance, theme: theme as Theme } })
              }
              options={THEMES}
              aria-label="Colour scheme"
              className="w-52"
            />
          }
        />
        <SettingRow
          label="Window opacity"
          description="Applies to this window and to the correction overlay. Below 100% your desktop shows through."
          control={
            <div className="flex w-52 items-center gap-3">
              <Slider
                value={shown}
                min={OPACITY_MIN}
                max={100}
                aria-label="Window opacity"


                onValueChange={(opacity) => {
                  setDragging(opacity);
                  document.documentElement.style.opacity = String(opacity / 100);
                }}
                onValueCommit={(opacity) => {
                  setDragging(null);
                  void update({ appearance: { ...appearance, opacity } });
                }}
              />
              <span data-numeric className="w-10 text-right text-xs text-muted">
                {shown}%
              </span>
            </div>
          }
        />
      </SettingGroup>
    </Panel>
  );
}
