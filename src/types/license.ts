export type LicenseTier = "free" | "pro";

export interface LicenseLimits {
  clipboard_history_unlimited: boolean;
  monitor_dimming: boolean;
  disk_clean: boolean;
  auto_layout: boolean;
  window_switcher: boolean;
  workspace_profiles: boolean;
}

export interface LicenseInfo {
  device_id: string;
  email: string | null;
  tier: LicenseTier;
  activated_at: number | null;
  last_verified_at: number | null;
  limits: LicenseLimits;
}

export type LicenseFeature = keyof LicenseLimits;
