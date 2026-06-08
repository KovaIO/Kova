pub mod aggregation;
pub mod collector;
pub mod models;
pub mod quit;

pub use aggregation::aggregate_process_metrics;
pub use collector::collect_processes;
pub use models::{FlatProcess, ProcessMeta, ProcessSnapshot};
pub use quit::{force_quit_process, quit_process};
