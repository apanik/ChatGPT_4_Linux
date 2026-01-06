use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub accelerator: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            accelerator: "Alt+Space".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    pub position: Option<WindowPosition>,
    pub always_on_top: bool,
    pub opacity: f64,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 420.0,
            height: 520.0,
            position: None,
            always_on_top: true,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionSettings {
    pub window: WindowState,
    pub remember_position: bool,
}

impl Default for CompanionSettings {
    fn default() -> Self {
        Self {
            window: WindowState::default(),
            remember_position: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub open_links_in_browser: bool,
    pub notifications_enabled: bool,
    pub launch_at_login: bool,
    pub allow_voice: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            open_links_in_browser: true,
            notifications_enabled: true,
            launch_at_login: false,
            allow_voice: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey: HotkeyConfig,
    pub companion: CompanionSettings,
    pub preferences: Preferences,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: HotkeyConfig::default(),
            companion: CompanionSettings::default(),
            preferences: Preferences::default(),
        }
    }
}

pub struct SettingsManager {
    path: PathBuf,
    state: Mutex<AppSettings>,
}

impl SettingsManager {
    pub fn new(app: &AppHandle) -> Result<Self> {
        let path = app
            .path_resolver()
            .app_config_dir()
            .unwrap_or_else(|| app.path_resolver().app_cache_dir().unwrap())
            .join("settings.json");

        Self::from_path(path)
    }

    pub fn from_path(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("creating config directory")?;
        }

        let state = if path.exists() {
            let data = fs::read_to_string(&path).context("reading settings file")?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            AppSettings::default()
        };

        Ok(Self {
            path,
            state: Mutex::new(state),
        })
    }

    pub fn get(&self) -> AppSettings {
        self.state.lock().clone()
    }

    pub fn update<F>(&self, updater: F) -> Result<AppSettings>
    where
        F: FnOnce(&mut AppSettings),
    {
        let mut state = self.state.lock();
        updater(&mut state);
        self.persist(&state)?;
        Ok(state.clone())
    }

    pub fn persist(&self, state: &AppSettings) -> Result<()> {
        let serialized = serde_json::to_string_pretty(state).context("serializing settings")?;
        fs::write(&self.path, serialized).context("writing settings file")?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        {
            let mut state = self.state.lock();
            *state = AppSettings::default();
            self.persist(&state.clone())?;
        }
        if self.path.exists() {
            let _ = fs::remove_file(&self.path);
        }
        Ok(())
    }
}

pub fn reset_companion_position(settings: &SettingsManager) -> Result<AppSettings> {
    settings.update(|s| {
        s.companion.window.position = None;
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn defaults_persist_and_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let manager = SettingsManager::from_path(path.clone()).unwrap();

        let settings = manager.get();
        assert_eq!(settings.hotkey.accelerator, "Alt+Space");
        assert!(settings.preferences.open_links_in_browser);

        manager
            .update(|s| {
                s.hotkey.accelerator = "Ctrl+Shift+Space".into();
                s.preferences.notifications_enabled = false;
            })
            .unwrap();

        let manager2 = SettingsManager::from_path(path).unwrap();
        let reloaded = manager2.get();
        assert_eq!(reloaded.hotkey.accelerator, "Ctrl+Shift+Space");
        assert!(!reloaded.preferences.notifications_enabled);
    }
}
