pub mod metrics;
pub mod windows;

pub use metrics::{get_current_metrics, start_metrics_loop};
pub use windows::{open_monitor, open_preferences};
