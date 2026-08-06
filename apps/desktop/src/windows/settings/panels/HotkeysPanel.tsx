import { useCallback, useEffect, useRef, useState } from "react";
import { CircleAlert, Info, Keyboard } from "lucide-react";
import { Button, Callout, Panel, SettingGroup, SettingRow } from "@/components/ui";
import { ipc, toFixError } from "@/lib/ipc";
import { useAppStore } from "@/store/useAppStore";
import { cn } from "@/lib/cn";

export function HotkeysPanel() {
  const settings = useAppStore((state) => state.settings);
  const status = useAppStore((state) => state.hotkeyStatus);
  const capabilities = useAppStore((state) => state.capabilities);
  const update = useAppStore((state) => state.update);

  const [recording, setRecording] = useState(false);
  const [candidate, setCandidate] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  const stopRecording = useCallback(() => {
    setRecording(false);
    setCandidate(null);
    setProblem(null);
  }, []);

  const commit = useCallback(
    async (accelerator: string) => {
      try {
        await update({ hotkey: accelerator });
        stopRecording();
      } catch (error) {
        setProblem(toFixError(error).message);
      }
    },
    [update, stopRecording],
  );

  if (!settings || !capabilities) return null;

  const externalOnly = capabilities.hotkey === "externalCommand";

  return (
    <Panel
      title="Hotkeys"
      description="The key combination that corrects your selected text, anywhere on your desktop."
    >
      {externalOnly ? (
        <Callout tone="warning" icon={<CircleAlert />} title="Set this in your compositor" className="mb-7">
          This session cannot register global hotkeys from inside an application. Bind a key in
          your compositor&rsquo;s own settings to the command <code className="font-mono">zyntax fix</code>{" "}
          and it will drive the running app.
        </Callout>
      ) : status && !status.registered ? (
        <Callout
          tone="danger"
          icon={<CircleAlert />}
          title="This hotkey is not active"
          className="mb-7"
        >
          {status.problem ?? "It could not be registered."} Until you pick a free combination,
          pressing it will do nothing.
        </Callout>
      ) : null}

      <SettingGroup title="Correction hotkey">
        <SettingRow
          label="Shortcut"
          description={
            recording
              ? "Press the combination you want. Esc cancels."
              : "Include at least one modifier, so it cannot fire while you are typing."
          }
          control={
            <div className="flex items-center gap-2">
              <HotkeyRecorder
                recording={recording}
                display={candidate ?? status?.display ?? settings.hotkey}
                registered={status?.registered ?? false}
                onCandidate={setCandidate}
                onProblem={setProblem}
                onCommit={commit}
                onCancel={stopRecording}
              />
              <Button
                variant={recording ? "secondary" : "primary"}
                size="md"
                onClick={() => (recording ? stopRecording() : setRecording(true))}
                disabled={externalOnly}
              >
                {recording ? "Cancel" : "Change"}
              </Button>
            </div>
          }
        />
      </SettingGroup>

      {problem ? (
        <Callout tone="danger" icon={<CircleAlert />} className="mb-7">
          {problem}
        </Callout>
      ) : null}

      <SettingGroup title="Suggestions">
        <div className="px-4 py-3">
          <p className="mb-3 text-xs leading-relaxed text-muted">
            These are unclaimed on Windows, macOS and the common Linux desktops. ZyntaxAI no longer
            defaults to <span className="font-mono">Ctrl+Alt+T</span>, which most Linux desktops
            already use to open a terminal.
          </p>
          <div className="flex flex-wrap gap-2">
            {["Ctrl+Alt+G", "Ctrl+Shift+Space", "Alt+Shift+F", "Ctrl+Alt+Period"].map(
              (accelerator) => (
                <Button
                  key={accelerator}
                  variant="secondary"
                  size="sm"
                  disabled={externalOnly || settings.hotkey === accelerator}
                  onClick={() => void commit(accelerator)}
                >
                  {accelerator.replaceAll("+", " + ")}
                </Button>
              ),
            )}
          </div>
        </div>
      </SettingGroup>

      <Callout tone="neutral" icon={<Info />}>
        Pressing the hotkey while a correction is already running cancels the first one rather than
        queueing another.
      </Callout>
    </Panel>
  );
}


function HotkeyRecorder({
  recording,
  display,
  registered,
  onCandidate,
  onProblem,
  onCommit,
  onCancel,
}: {
  recording: boolean;
  display: string;
  registered: boolean;
  onCandidate: (value: string | null) => void;
  onProblem: (value: string | null) => void;
  onCommit: (accelerator: string) => void;
  onCancel: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (recording) ref.current?.focus();
  }, [recording]);

  useEffect(() => {
    if (!recording) return;

    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();

      if (event.key === "Escape") {
        onCancel();
        return;
      }

      const modifiers: string[] = [];
      if (event.ctrlKey) modifiers.push("Ctrl");
      if (event.altKey) modifiers.push("Alt");
      if (event.shiftKey) modifiers.push("Shift");
      if (event.metaKey) modifiers.push("Super");

      const key = normaliseKey(event);
      if (!key) {

        onCandidate(modifiers.length ? `${modifiers.join(" + ")} + …` : null);
        return;
      }
      if (!modifiers.length) {
        onProblem("Add a modifier — Ctrl, Alt, Shift or Super.");
        return;
      }

      const accelerator = [...modifiers, key].join("+");
      onCandidate(accelerator.replaceAll("+", " + "));
      void ipc
        .validateHotkey(accelerator)
        .then(() => onCommit(accelerator))
        .catch((error) => onProblem(toFixError(error).message));
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [recording, onCandidate, onProblem, onCommit, onCancel]);

  return (
    <div
      ref={ref}
      tabIndex={recording ? 0 : -1}
      className={cn(
        "flex h-control-md min-w-44 items-center justify-center rounded-md px-3",
        "border text-sm font-medium transition-colors duration-fast ease-out",
        recording
          ? "border-accent bg-accent-subtle text-fg"
          : registered
            ? "border-line bg-inset text-fg"
            : "border-danger/40 bg-inset text-muted",
      )}
    >
      {recording && !display.includes("…") ? (
        <span className="inline-flex items-center gap-2 text-muted">
          <Keyboard className="size-3.5" />
          Waiting…
        </span>
      ) : (
        display
      )}
    </div>
  );
}


function normaliseKey(event: KeyboardEvent): string | null {
  const { key, code } = event;

  if (["Control", "Alt", "Shift", "Meta"].includes(key)) return null;

  if (/^F\d{1,2}$/.test(key)) return key;
  if (key === " ") return "Space";

  const named: Record<string, string> = {
    Enter: "Enter",
    Tab: "Tab",
    Backspace: "Backspace",
    Delete: "Delete",
    Insert: "Insert",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    ",": "Comma",
    ".": "Period",
    "/": "Slash",
    ";": "Semicolon",
  };
  if (named[key]) return named[key];


  const letter = /^Key([A-Z])$/.exec(code);
  if (letter?.[1]) return letter[1];
  const digit = /^Digit(\d)$/.exec(code);
  if (digit?.[1]) return digit[1];

  return null;
}
