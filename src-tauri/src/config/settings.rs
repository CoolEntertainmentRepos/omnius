use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

const DEFAULT_SERVER_URL: &str = "https://api.omnius.lol";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SettingsData {
    server_url: String,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            server_url: DEFAULT_SERVER_URL.to_string(),
        }
    }
}

pub struct Settings {
    data: RwLock<SettingsData>,
    config_path: Option<PathBuf>,
}

impl Settings {
    pub fn load(config_dir: Option<PathBuf>) -> Self {
        let config_path = config_dir.map(|dir| dir.join("config.json"));
        let data = if let Some(ref path) = config_path {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or_default()
                }
                Err(_) => SettingsData::default(),
            }
        } else {
            SettingsData::default()
        };

        Self {
            data: RwLock::new(data),
            config_path,
        }
    }

    pub fn server_url(&self) -> String {
        self.data.read().unwrap().server_url.clone()
    }

    pub fn set_server_url(&self, url: &str) {
        {
            let mut data = self.data.write().unwrap();
            data.server_url = url.trim_end_matches('/').to_string();
        }
        self.save();
    }

    fn save(&self) {
        if let Some(ref path) = self.config_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let data = self.data.read().unwrap();
            if let Ok(json) = serde_json::to_string_pretty(&*data) {
                let _ = std::fs::write(path, json);
            }
        }
    }
}
