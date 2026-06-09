pub mod client;
pub mod limits;
pub mod models;
pub mod service;
pub mod storage;

pub use limits::{
    sanitize_clipboard_history_limit, validate_clipboard_history_limit, validate_monitor_dim,
};
pub use models::{LicenseInfo, LicenseLimits, LicenseTier};
pub use service::LicenseService;
pub use storage::LicenseStorage;
