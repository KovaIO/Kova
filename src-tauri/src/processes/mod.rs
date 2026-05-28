pub mod aggregation;
pub mod collector;
pub mod icons;
pub mod models;

pub use aggregation::aggregate_process_metrics;
pub use collector::{collect_processes, get_running_processes};
pub use icons::get_process_icon;
pub use models::{FlatProcess, ProcessMeta, RunningProcess};
