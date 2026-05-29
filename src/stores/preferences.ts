import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import type { Preferences } from "$types/preferences";

export const preferences = writable<Preferences | null>(null);

export async function loadPreferences() {
  const prefs = await invoke<Preferences>("get_preferences");

  preferences.set(prefs);
}
