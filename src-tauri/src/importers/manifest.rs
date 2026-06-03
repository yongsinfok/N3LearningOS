use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub content_type: String,
    pub checksum: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub version: String,
    pub content_date: Option<String>,
    pub files: Vec<ManifestEntry>,
}

impl Manifest {
    pub fn load(content_dir: &Path) -> Result<Self, String> {
        let manifest_path = content_dir.join("manifest.json");
        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Cannot read manifest: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Cannot parse manifest: {}", e))
    }
}
