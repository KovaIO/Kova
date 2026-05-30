pub mod limits;
pub mod models;
pub mod service;
pub mod storage;

pub use limits::{
    sanitize_clipboard_history_limit, sanitize_window_manager_preferences,
    validate_clipboard_history_limit, validate_monitor_dim, validate_window_manager_preferences,
};
pub use models::{LicenseInfo, LicenseLimits, LicenseTier};
pub use service::LicenseService;
pub use storage::LicenseStorage;
