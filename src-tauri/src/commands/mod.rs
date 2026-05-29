pub mod metrics;
pub mod processes;
pub mod windows;

pub use metrics::{get_current_metrics, get_snapshot};
pub use processes::{
    force_quit_process_cmd, get_process_history, get_running_procs, quit_process_cmd,
};
pub use windows::*;
