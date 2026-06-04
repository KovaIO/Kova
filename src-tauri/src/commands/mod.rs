pub mod clipboard;
pub mod license;
pub mod metrics;
pub mod preferences;
pub mod processes;
pub mod windows;
pub mod workspaces;

pub use clipboard::{
    clear_clipboard_history, copy_clipboard_item, delete_clipboard_item, get_apps,
    get_clipboard_history, open_clipboard_url, paste_clipboard_item, paste_plain_clipboard_item,
    preview_clipboard_item, reveal_clipboard_item,
};
pub use license::*;
pub use metrics::{get_current_metrics, get_snapshot};
pub use preferences::*;
pub use processes::{force_quit_process_cmd, get_process_history, quit_process_cmd};
pub use windows::*;
pub use workspaces::{
    apply_workspace, delete_workspace_profile, get_workspace_profile, get_workspace_profiles,
    save_workspace_profile,
};
