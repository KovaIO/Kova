import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import type { Preferences } from "$types/preferences";
import { normalizeStoredShortcuts } from "$utils/keyboard-shortcuts";

export const preferences = writable<Preferences | null>(null);

export function normalizePreferences(prefs: Preferences): Preferences {
  return {
    ...prefs,
    shortcuts: normalizeStoredShortcuts(prefs.shortcuts),
  };
}

export function setPreferences(prefs: Preferences) {
  preferences.set(normalizePreferences(prefs));
}

export async function loadPreferences() {
  const prefs = await invoke<Preferences>("get_preferences");
  setPreferences(prefs);
}
