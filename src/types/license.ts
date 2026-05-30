export type LicenseTier = "free" | "pro";

export interface LicenseLimits {
  clipboard_history_unlimited: boolean;
  monitor_dimming: boolean;
  disk_clean: boolean;
  auto_layout: boolean;
  window_switcher: boolean;
}

export interface LicenseInfo {
  tier: LicenseTier;
  limits: LicenseLimits;
}

export type LicenseFeature = keyof LicenseLimits;
