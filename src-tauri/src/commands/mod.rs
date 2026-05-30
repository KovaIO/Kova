pub mod clipboard;
pub mod license;
pub mod metrics;
pub mod preferences;
pub mod processes;
pub mod windows;

pub use clipboard::get_apps;
pub use license::*;
pub use metrics::{get_current_metrics, get_snapshot};
pub use preferences::*;
pub use processes::{force_quit_process_cmd, get_process_history, quit_process_cmd};
pub use windows::*;
