

import { create } from "zustand";
import {
  ipc,
  toFixError,
  type AppSettings,
  type Capabilities,
  type FixError,
  type HotkeyStatus,
  type Language,
  type Persona,
  type SecretBackend,
  type Stats,
  type UpdateInfo,
} from "@/lib/ipc";

interface AppStore {
  settings: AppSettings | null;
  capabilities: Capabilities | null;
  personas: Persona[];
  languages: Language[];
  hotkeyStatus: HotkeyStatus | null;
  secretBackend: SecretBackend | null;
  stats: Stats | null;
  version: string;

  availableUpdate: UpdateInfo | null;


  loading: boolean;

  saveError: FixError | null;

  hydrate: () => Promise<void>;

  refreshStats: () => Promise<void>;
  setAvailableUpdate: (update: UpdateInfo | null) => void;

  update: (patch: Partial<AppSettings>) => Promise<void>;
  clearSaveError: () => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  settings: null,
  capabilities: null,
  personas: [],
  languages: [],
  hotkeyStatus: null,
  secretBackend: null,
  stats: null,
  version: "",
  availableUpdate: null,
  loading: true,
  saveError: null,

  hydrate: async () => {


    const [
      settings,
      capabilities,
      personas,
      languages,
      hotkeyStatus,
      secretBackend,
      stats,
      version,
      availableUpdate,
    ] = await Promise.all([
      ipc.getSettings(),
      ipc.getCapabilities(),
      ipc.getPersonas(),
      ipc.getLanguages(),
      ipc.getHotkeyStatus(),
      ipc.getSecretBackend(),
      ipc.getStats(),
      ipc.appVersion(),
      ipc.pendingUpdate(),
    ]);

    set({
      settings,
      capabilities,
      personas,
      languages,
      hotkeyStatus,
      secretBackend,
      stats,
      version,
      availableUpdate,
      loading: false,
    });
  },

  refreshStats: async () => {
    set({ stats: await ipc.getStats() });
  },

  setAvailableUpdate: (availableUpdate) => set({ availableUpdate }),

  update: async (patch) => {
    const current = get().settings;
    if (!current) return;

    const next = { ...current, ...patch };


    set({ settings: next, saveError: null });

    try {
      const saved = await ipc.saveSettings(next);

      set({ settings: saved });


      const [personas, languages, hotkeyStatus] = await Promise.all([
        ipc.getPersonas(),
        ipc.getLanguages(),
        ipc.getHotkeyStatus(),
      ]);
      set({ personas, languages, hotkeyStatus });
    } catch (error) {
      const failure = toFixError(error);
      set({ settings: current, saveError: failure });
      throw failure;
    }
  },

  clearSaveError: () => set({ saveError: null }),
}));


export function useSettings(): AppSettings {
  const settings = useAppStore((state) => state.settings);
  if (!settings) {
    throw new Error("useSettings called before hydration completed");
  }
  return settings;
}
