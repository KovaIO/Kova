use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
    pub exe_path: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardContentType {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: i64,
    pub content_type: ClipboardContentType,
    pub text_content: Option<String>,
    pub image_path: Option<String>,
    pub source_app: Option<String>,
    pub source_app_icon: Option<String>,
    pub source_app_path: Option<String>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub image_size: Option<u64>,
    pub image_filename: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct SourceApp {
    pub name: String,
    pub path: String,
}
