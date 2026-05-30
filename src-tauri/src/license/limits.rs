use crate::license::{LicenseLimits, LicenseTier};

const UNLIMITED_HISTORY_VALUE: i32 = 0;
const MAX_FINITE_HISTORY: i32 = 100;

pub fn limits_for_tier(tier: &LicenseTier) -> LicenseLimits {
    match tier {
        LicenseTier::Free => LicenseLimits {
            clipboard_history_unlimited: false,
            monitor_dimming: false,
            disk_clean: false,
            auto_layout: false,
            window_switcher: false,
        },
        LicenseTier::Pro => LicenseLimits {
            clipboard_history_unlimited: true,
            monitor_dimming: true,
            disk_clean: true,
            auto_layout: true,
            window_switcher: true,
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

pub fn validate_monitor_dim(monitor_dim: u8, tier: &LicenseTier) -> Result<u8, String> {
    if tier == &LicenseTier::Pro {
        return Ok(monitor_dim.clamp(0, 100));
    }

    Err("Monitor dimming requires a Pro license".into())
}

pub fn validate_window_manager_preferences(
    prefs: crate::preferences::WindowManagerPreferences,
    tier: &LicenseTier,
) -> Result<crate::preferences::WindowManagerPreferences, String> {
    if tier == &LicenseTier::Pro {
        return Ok(prefs);
    }

    if prefs.auto_layout || prefs.window_switcher {
        return Err("This feature requires a Pro license".into());
    }

    Ok(prefs)
}

pub fn sanitize_window_manager_preferences(
    prefs: crate::preferences::WindowManagerPreferences,
    tier: &LicenseTier,
) -> crate::preferences::WindowManagerPreferences {
    if tier == &LicenseTier::Pro {
        return prefs;
    }

    crate::preferences::WindowManagerPreferences {
        enabled: prefs.enabled,
        auto_layout: false,
        window_switcher: false,
    }
}
