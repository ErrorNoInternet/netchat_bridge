use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Secrets {
    pub homeserver_url: String,
    pub username: String,
    pub password: String,
}

impl Secrets {
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
            Ok(secrets) => Ok(secrets),
            Err(error) => Err(error.to_string()),
        }
    }
}
