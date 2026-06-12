use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceApp {
    pub name: String,
    pub path: String,
    pub exe_path: Option<String>,
    pub icon: Option<String>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceProfile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub gap: u32,
    pub apps: Vec<WorkspaceApp>,
}
