use crate::license::{LicenseLimits, LicenseTier};

const UNLIMITED_HISTORY_VALUE: i32 = 0;
const MAX_FINITE_HISTORY: i32 = 100;

pub fn limits_for_tier(tier: &LicenseTier) -> LicenseLimits {
    match tier {
        LicenseTier::Free => LicenseLimits {
            clipboard_history_unlimited: false,
        },
        LicenseTier::Pro => LicenseLimits {
            clipboard_history_unlimited: true,
        },
    }
}

pub fn validate_clipboard_history_limit(limit: i32, tier: &LicenseTier) -> Result<i32, String> {
    if limit == UNLIMITED_HISTORY_VALUE {
        if tier == &LicenseTier::Pro {
            return Ok(limit);
        }

        return Err("Unlimited clipboard history requires a Pro license".into());
    }

    if (1..=MAX_FINITE_HISTORY).contains(&limit) {
        return Ok(limit);
    }

    Err(format!(
        "History limit must be between 1 and {MAX_FINITE_HISTORY}, or unlimited with Pro"
    ))
}

pub fn sanitize_clipboard_history_limit(limit: i32, tier: &LicenseTier) -> i32 {
    match validate_clipboard_history_limit(limit, tier) {
        Ok(valid) => valid,
        Err(_) => 25,
    }
}
