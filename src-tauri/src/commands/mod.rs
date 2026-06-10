pub mod apps;
pub mod clipboard;
pub mod disk;
pub mod license;
pub mod metrics;
pub mod preferences;
pub mod processes;
pub mod windows;
pub mod workspaces;

pub use apps::get_apps;
pub use clipboard::{
    clear_clipboard_history, copy_clipboard_item, delete_clipboard_item, get_clipboard_history,
    open_clipboard_url, paste_clipboard_item, paste_plain_clipboard_item, preview_clipboard_item,
    reveal_clipboard_item,
};
pub use disk::{
    delete_all_disk_items, delete_disk_items, get_disk_item_detail, get_disk_scan_preview,
    get_disk_scan_result, get_disk_volume_info, start_disk_scan,
};
pub use license::{activate_license, get_license, get_portal_url};
pub use metrics::{get_current_metrics, get_snapshot};
pub use preferences::*;
pub use processes::{force_quit_process_cmd, get_process_history, quit_process_cmd};
pub use windows::*;
pub use workspaces::{
    apply_workspace, delete_workspace_profile, get_workspace_profile, get_workspace_profiles,
    save_workspace_profile,
};
