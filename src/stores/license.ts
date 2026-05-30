import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

import type { LicenseFeature, LicenseInfo } from "$types/license";

export const license = writable<LicenseInfo | null>(null);

export async function loadLicense() {
  const info = await invoke<LicenseInfo>("get_license");
  license.set(info);
}

export function isPro(info: LicenseInfo | null): boolean {
  return info?.tier === "pro";
}

export function canUse(
  feature: LicenseFeature,
  info: LicenseInfo | null,
): boolean {
  return info?.limits[feature] ?? false;
}
