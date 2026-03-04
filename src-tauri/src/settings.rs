use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub hooks: HookSettings,
    pub notification_duration: u32,
    pub play_sound: bool,
    pub auto_start: bool,
    pub server_port: u16,
    pub suppress_when_focused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSettings {
    pub permission_prompt: bool,
    pub idle_prompt: bool,
    pub stop: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hooks: HookSettings {
                permission_prompt: true,
                idle_prompt: true,
                stop: true,
            },
            notification_duration: 5,
            play_sound: false,
            auto_start: false,
            server_port: 31311,
            suppress_when_focused: true,
        }
    }
}

impl Settings {
    fn config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "mgblackwater", "claude-notify") {
            let config_dir = proj_dirs.config_dir();
            config_dir.join("settings.json")
        } else {
            PathBuf::from("settings.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    Ok(Settings::load())
}

#[tauri::command]
pub async fn update_settings(new_settings: Settings) -> Result<(), String> {
    tokio::task::spawn_blocking(move || new_settings.save())
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn reset_settings() -> Result<Settings, String> {
    let defaults = Settings::default();
    let to_save = defaults.clone();
    tokio::task::spawn_blocking(move || to_save.save())
        .await
        .map_err(|e| e.to_string())??;
    Ok(defaults)
}
