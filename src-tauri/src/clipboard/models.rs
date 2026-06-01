use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClipboardItem {
    pub id: i64,
    pub content_type: ClipboardContentType,
    pub text_content: Option<String>,
    pub image_path: Option<String>,
    pub source_app: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub enum ClipboardContentType {
    Text,
    Image,
}
