use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    pub command_prefix: String,
    #[serde(default)]
    pub refresh_interval: u64,
    #[serde(default)]
    pub request_timeout: u64,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            command_prefix: "!".to_string(),
            refresh_interval: 5,
            request_timeout: 30,
        }
    }
}

impl Configuration {
    pub fn from_json_file(path: &Path) -> Result<Self, String> {
        let file_contents = match std::fs::read_to_string(path) {
            Ok(file_contents) => file_contents,
            Err(error) => {
                return match error.kind() {
                    std::io::ErrorKind::NotFound => Err(format!("file not found")),
                    _ => Err(format!("unable to read file: {error}")),
                }
            }
        };
        match serde_json::from_str(&file_contents) {
            Ok(configuration) => Ok(configuration),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn to_json_file(&self, path: &Path) -> Result<(), String> {
        match std::fs::write(
            path,
            serde_json::to_string_pretty(&self).unwrap().as_bytes(),
        ) {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("unable to save file: {error}")),
        }
    }
}
