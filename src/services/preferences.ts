import { get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import { preferences, normalizePreferences } from "$stores/preferences";
import type {
  ClipboardPreferences,
  GeneralPreferences,
  Preferences,
  Shortcut,
  WindowManagerPreferences,
} from "$types/preferences";
import { shortcutsForApi, normalizeStoredShortcuts } from "$utils/keyboard-shortcuts";

type UpdatableSection = keyof Pick<
  Preferences,
  "general" | "clipboard" | "window_manager"
>;

const UPDATE_COMMANDS: Record<UpdatableSection, string> = {
  general: "update_general_preferences",
  clipboard: "update_clipboard_preferences",
  window_manager: "update_window_manager_preferences",
};

async function updateSection<S extends UpdatableSection>(
  section: S,
  patch: Partial<Preferences[S]>,
): Promise<void> {
  const current = get(preferences);
  if (!current) {
    throw new Error("Preferences not loaded");
  }

  const prefs = { ...current[section], ...patch };

  await invoke(UPDATE_COMMANDS[section], { prefs });
}

export function updateGeneral(patch: Partial<GeneralPreferences>) {
  return updateSection("general", patch);
}

export function updateClipboard(patch: Partial<ClipboardPreferences>) {
  return updateSection("clipboard", patch);
}

export function updateWindowManager(patch: Partial<WindowManagerPreferences>) {
  return updateSection("window_manager", patch);
}

export function updateShortcuts(shortcuts: Shortcut[]) {
  const normalized = normalizeStoredShortcuts(shortcuts);

  preferences.update((current) =>
    current
      ? normalizePreferences({ ...current, shortcuts: normalized })
      : current,
  );

  return invoke("update_shortcuts", {
    shortcuts: shortcutsForApi(normalized),
  });
}
