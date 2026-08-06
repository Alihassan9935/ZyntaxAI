import { useState } from "react";
import { Lock, Pencil, Plus, Trash2, Users } from "lucide-react";
import {
  Button,
  Dialog,
  EmptyState,
  Input,
  Panel,
  SettingGroup,
  Textarea,
} from "@/components/ui";
import { cn } from "@/lib/cn";
import { useAppStore } from "@/store/useAppStore";
import type { Persona } from "@/lib/ipc";

export function PersonasPanel() {
  const settings = useAppStore((state) => state.settings);
  const personas = useAppStore((state) => state.personas);
  const update = useAppStore((state) => state.update);

  const [editing, setEditing] = useState<Persona | null>(null);
  const [creating, setCreating] = useState(false);
  const [deleting, setDeleting] = useState<Persona | null>(null);

  if (!settings) return null;

  const custom = personas.filter((persona) => !persona.builtin);
  const builtin = personas.filter((persona) => persona.builtin);

  const select = (id: string) => void update({ personaId: id });

  const save = (persona: Persona) => {
    const others = settings.customPersonas.filter((p) => p.id !== persona.id);
    void update({ customPersonas: [...others, persona] });
    setEditing(null);
    setCreating(false);
  };

  const remove = (persona: Persona) => {
    void update({
      customPersonas: settings.customPersonas.filter((p) => p.id !== persona.id),


      ...(settings.personaId === persona.id ? { personaId: "standard" } : {}),
    });
    setDeleting(null);
  };

  return (
    <Panel
      title="Personas"
      description="The writing style applied on top of grammar correction."
      actions={
        <Button variant="primary" size="md" onClick={() => setCreating(true)}>
          <Plus />
          New persona
        </Button>
      }
    >
      <SettingGroup title="Built in">
        {builtin.map((persona) => (
          <PersonaRow
            key={persona.id}
            persona={persona}
            active={persona.id === settings.personaId}
            onSelect={() => select(persona.id)}
          />
        ))}
      </SettingGroup>

      <SettingGroup title="Yours">
        {custom.length === 0 ? (
          <EmptyState
            icon={<Users />}
            title="No custom personas yet"
            description="Create one to give the model a voice of your own — a house style, a tone for a particular client, anything."
            action={
              <Button variant="secondary" size="md" onClick={() => setCreating(true)}>
                <Plus />
                New persona
              </Button>
            }
          />
        ) : (
          custom.map((persona) => (
            <PersonaRow
              key={persona.id}
              persona={persona}
              active={persona.id === settings.personaId}
              onSelect={() => select(persona.id)}
              onEdit={() => setEditing(persona)}
              onDelete={() => setDeleting(persona)}
            />
          ))
        )}
      </SettingGroup>

      <PersonaDialog
        open={creating || editing !== null}
        persona={editing}
        onClose={() => {
          setCreating(false);
          setEditing(null);
        }}
        onSave={save}
      />

      <Dialog
        open={deleting !== null}
        onOpenChange={(open) => !open && setDeleting(null)}
        title={`Delete “${deleting?.name}”?`}
        width="sm"
        footer={
          <>
            <Button variant="ghost" onClick={() => setDeleting(null)}>
              Cancel
            </Button>
            <Button variant="danger" onClick={() => deleting && remove(deleting)}>
              Delete
            </Button>
          </>
        }
      >
        <p className="text-sm text-muted">This persona will be removed. This cannot be undone.</p>
      </Dialog>
    </Panel>
  );
}

function PersonaRow({
  persona,
  active,
  onSelect,
  onEdit,
  onDelete,
}: {
  persona: Persona;
  active: boolean;
  onSelect: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
}) {
  return (
    <div
      className={cn(
        "group flex items-start gap-3 px-4 py-3 transition-colors duration-fast ease-out",
        active ? "bg-accent-subtle" : "hover:bg-hover/50",
      )}
    >
      <button
        onClick={onSelect}
        className="min-w-0 flex-1 text-left"
        aria-pressed={active}
        aria-label={`Use the ${persona.name} persona`}
      >
        <span className="flex items-center gap-2">
          <span className="text-sm font-medium text-fg">{persona.name}</span>
          {active ? <span className="text-2xs text-accent">Active</span> : null}
          {persona.builtin ? <Lock className="size-3 text-faint" aria-label="Built in" /> : null}
        </span>
        <p className="mt-1 line-clamp-2 text-xs leading-relaxed text-muted">
          {persona.instruction}
        </p>
      </button>

      {onEdit && onDelete ? (


        <div className="flex shrink-0 gap-1 opacity-0 transition-opacity duration-fast group-hover:opacity-100 focus-within:opacity-100">
          <Button variant="ghost" size="sm" icon onClick={onEdit} aria-label={`Edit ${persona.name}`}>
            <Pencil />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            icon
            onClick={onDelete}
            aria-label={`Delete ${persona.name}`}
          >
            <Trash2 />
          </Button>
        </div>
      ) : null}
    </div>
  );
}

function PersonaDialog({
  open,
  persona,
  onClose,
  onSave,
}: {
  open: boolean;
  persona: Persona | null;
  onClose: () => void;
  onSave: (persona: Persona) => void;
}) {
  const [name, setName] = useState("");
  const [instruction, setInstruction] = useState("");
  const [touched, setTouched] = useState(false);


  const key = `${open}:${persona?.id ?? "new"}`;
  const [lastKey, setLastKey] = useState(key);
  if (key !== lastKey) {
    setLastKey(key);
    setName(persona?.name ?? "");
    setInstruction(persona?.instruction ?? "");
    setTouched(false);
  }

  const valid = name.trim().length > 0 && instruction.trim().length > 0;

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => !next && onClose()}
      title={persona ? "Edit persona" : "New persona"}
      description="Describe the voice you want. This is added to the instructions the model receives."
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button
            variant="primary"
            disabled={!valid}
            onClick={() =>
              onSave({
                id: persona?.id ?? crypto.randomUUID(),
                name: name.trim(),
                instruction: instruction.trim(),
                builtin: false,
              })
            }
          >
            {persona ? "Save changes" : "Create persona"}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <label className="block">
          <span className="mb-1.5 block text-xs font-medium text-fg">Name</span>
          <Input
            value={name}
            onChange={(event) => setName(event.target.value)}
            onBlur={() => setTouched(true)}
            placeholder="Support reply"
            autoFocus
          />
        </label>

        <label className="block">
          <span className="mb-1.5 block text-xs font-medium text-fg">Instruction</span>
          <Textarea
            rows={5}
            value={instruction}
            onChange={(event) => setInstruction(event.target.value)}
            onBlur={() => setTouched(true)}
            placeholder="Correct the text and make it warm but efficient. Never promise a timeline the author did not give."
          />
          <span className="mt-1.5 block text-2xs leading-relaxed text-faint">
            Say what to change and what to leave alone — the second half is what keeps a persona
            from rewriting meaning.
          </span>
        </label>

        {touched && !valid ? (
          <p className="text-xs text-danger">Both a name and an instruction are required.</p>
        ) : null}
      </div>
    </Dialog>
  );
}
