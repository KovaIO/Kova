export type LicenseTier = "free" | "pro";

export interface LicenseLimits {
  clipboard_history_unlimited: boolean;
}

export interface LicenseInfo {
  tier: LicenseTier;
  limits: LicenseLimits;
}
