pub mod icons;
pub mod installed;

pub use icons::{get_app_icon, get_process_icon};
pub use installed::get_installed_apps;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
    pub exe_path: Option<String>,
    pub icon: Option<String>,
}
