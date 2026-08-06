import { CircleAlert } from "lucide-react";
import { Callout, Panel, Select, SettingGroup, SettingRow, Switch } from "@/components/ui";
import { useAppStore } from "@/store/useAppStore";
import type { InputSource } from "@/lib/bindings/InputSource";
import type { OutputMode } from "@/lib/bindings/OutputMode";
import type { Speed } from "@/lib/bindings/Speed";

const SPEEDS = [
  { value: "fast", label: "Fast", hint: "fewest changes" },
  { value: "normal", label: "Normal", hint: "recommended" },
  { value: "detailed", label: "Detailed", hint: "most thorough" },
];

const INPUT_SOURCES = [
  { value: "selection", label: "Selected text" },
  { value: "clipboard", label: "Clipboard contents" },
];

export function BehaviorPanel() {
  const settings = useAppStore((state) => state.settings);
  const capabilities = useAppStore((state) => state.capabilities);
  const update = useAppStore((state) => state.update);

  if (!settings || !capabilities) return null;

  const { behavior } = settings;
  const canInject = capabilities.injection !== "none";

  const setBehavior = (patch: Partial<typeof behavior>) =>
    void update({ behavior: { ...behavior, ...patch } });


  const outputModes = [
    { value: "review", label: "Show me first", hint: "default" },
    { value: "replace", label: "Replace the selection", disabled: !canInject },
    { value: "clipboard", label: "Copy to clipboard" },
    { value: "append", label: "Append after it", disabled: !canInject },
    { value: "prepend", label: "Insert before it", disabled: !canInject },
  ];

  return (
    <Panel title="Behavior" description="What happens when you press the hotkey.">
      {!canInject ? (
        <Callout
          tone="warning"
          icon={<CircleAlert />}
          title="Replacing text is unavailable in this session"
          className="mb-7"
        >
          {capabilities.notes.find((note) => note.title.includes("type into"))?.remedy ??
            "Corrections will be put on your clipboard instead."}
        </Callout>
      ) : null}

      <SettingGroup title="Correction">
        <SettingRow
          label="Processing depth"
          description="How much latitude the model has to rewrite, rather than only fix errors."
          control={
            <Select
              value={settings.speed}
              onValueChange={(value) => void update({ speed: value as Speed })}
              options={SPEEDS}
              aria-label="Processing depth"
              className="w-52"
            />
          }
        />
        <SettingRow
          label="Where the text comes from"
          description="Selected text is read directly where possible, with no keystrokes sent."
          control={
            <Select
              value={behavior.inputSource}
              onValueChange={(value) => setBehavior({ inputSource: value as InputSource })}
              options={INPUT_SOURCES}
              aria-label="Input source"
              className="w-52"
            />
          }
        />
        <SettingRow
          label="What happens to the result"
          description="Showing it first lets you see the change before it touches your document."
          control={
            <Select
              value={behavior.outputMode}
              onValueChange={(value) => setBehavior({ outputMode: value as OutputMode })}
              options={outputModes}
              aria-label="Output mode"
              className="w-52"
            />
          }
        />
        <SettingRow
          label="Also copy the correction"
          description="Puts the corrected text on your clipboard as well as applying it."
          control={
            <Switch
              checked={behavior.autoCopyFixed}
              onCheckedChange={(autoCopyFixed) => setBehavior({ autoCopyFixed })}
              aria-label="Also copy the correction"
            />
          }
        />
      </SettingGroup>

      <SettingGroup title="Feedback">
        <SettingRow
          label="Show notifications"
          description={
            behavior.outputMode === "review"
              ? "Only applies when a correction is applied without the overlay — the overlay is already the feedback. Failures are always shown."
              : "A desktop notification when a correction finishes. Failures are always shown."
          }
          control={
            <Switch
              checked={behavior.showNotifications}
              onCheckedChange={(showNotifications) => setBehavior({ showNotifications })}
              aria-label="Show notifications"
            />
          }
        />
        <SettingRow
          label="Play a sound"
          description={
            behavior.showNotifications
              ? "Uses your desktop's own notification sound."
              : "Needs notifications turned on — the sound is part of the notification."
          }
          control={
            <Switch
              checked={behavior.playSound}
              onCheckedChange={(playSound) => setBehavior({ playSound })}
              disabled={!behavior.showNotifications}
              aria-label="Play a sound"
            />
          }
        />
      </SettingGroup>

      <SettingGroup title="Window">
        <SettingRow
          label="Keep running in the tray"
          description="Closing this window leaves ZyntaxAI listening for the hotkey. Turn this off to quit on close."
          control={
            <Switch
              checked={behavior.minimizeToTray}
              onCheckedChange={(minimizeToTray) => setBehavior({ minimizeToTray })}
              aria-label="Keep running in the tray"
            />
          }
        />
      </SettingGroup>

      <SettingGroup
        title="History"
        description="Only counts and token totals are stored. Your text is never written to disk."
      >
        <SettingRow
          label="Record corrections"
          description="Powers the fix count and the Usage panel."
          control={
            <Switch
              checked={behavior.keepHistory}
              onCheckedChange={(keepHistory) => setBehavior({ keepHistory })}
              aria-label="Record corrections"
            />
          }
        />
      </SettingGroup>
    </Panel>
  );
}
