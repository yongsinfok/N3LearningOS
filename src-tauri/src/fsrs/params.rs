use super::types::FSRSParameters;
use std::fs;
use std::path::PathBuf;

pub fn load_params(params_path: &PathBuf) -> FSRSParameters {
    if params_path.exists() {
        match fs::read_to_string(params_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => FSRSParameters::default(),
        }
    } else {
        FSRSParameters::default()
    }
}

pub fn save_params(params_path: &PathBuf, params: &FSRSParameters) -> Result<(), String> {
    if let Some(parent) = params_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(params).map_err(|e| e.to_string())?;
    fs::write(params_path, json).map_err(|e| e.to_string())?;
    Ok(())
}
