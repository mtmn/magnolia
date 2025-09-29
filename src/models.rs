use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visits: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub file_type: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opens: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileStats {
    pub file_type: String,
    pub action: String,
    pub opens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub directories: Vec<DirectoryEntry>,
    pub files: Vec<FileEntry>,
}
