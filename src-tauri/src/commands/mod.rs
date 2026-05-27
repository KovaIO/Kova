pub mod metrics;
mod net_tracker;
mod process_icons;
mod processes;
pub mod windows;

pub use metrics::{get_current_metrics, start_metrics_loop};
pub use windows::{open_monitor, open_preferences};
pub use processes::get_running_processes;
