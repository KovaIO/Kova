import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import type { Preferences } from "$types/preferences";
import { normalizeStoredShortcuts } from "$utils/keyboard-shortcuts";
import { applyAccentColor } from "$utils/accent-colors";

export const preferences = writable<Preferences | null>(null);

export function normalizePreferences(prefs: Preferences): Preferences {
  return {
    ...prefs,
    shortcuts: normalizeStoredShortcuts(prefs.shortcuts),
  };
}

export function setPreferences(prefs: Preferences) {
  const normalized = normalizePreferences(prefs);

  applyAccentColor(normalized.appearance.accent_color);

  preferences.set(normalized);
}

export async function loadPreferences() {
  const prefs = await invoke<Preferences>("get_preferences");
  setPreferences(prefs);
}

export async function refreshBrightness() {
  try {
    const realBrightness = await invoke<number>("get_brightness");
    preferences.update((current) =>
      current
        ? { ...current, general: { ...current.general, monitor_dim: realBrightness } }
        : current,
    );
  } catch {
    // brightness read failed, keep current value
  }
}
