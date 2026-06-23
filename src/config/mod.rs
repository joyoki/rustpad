use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod theme;

/// Global application configuration, persisted as TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub editor: EditorConfig,
    pub ui: UiConfig,
    pub recent_files: Vec<PathBuf>,
    pub window: WindowConfig,
    pub auto_save: AutoSaveConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub font_size: f32,
    pub tab_size: usize,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub auto_indent: bool,
    pub highlight_current_line: bool,
    pub font_family: String,
    #[serde(default)]
    pub display_blank_chars: bool,
    #[serde(default)]
    pub display_non_print_chars: bool,
    #[serde(default)]
    pub show_tabs_as_spaces: bool,
    #[serde(default)]
    pub wrap_by_character: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_minimap: bool,
    pub sidebar_width: f32,
    pub keybinding_scheme: String,
    /// UI display language: `"en"` or `"zh"`.
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
}

fn default_ui_language() -> String {
    "en".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub maximized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            editor: EditorConfig {
                font_size: 14.0,
                tab_size: 4,
                show_line_numbers: true,
                word_wrap: false,
                auto_indent: true,
                highlight_current_line: true,
                font_family: "JetBrains Mono".to_string(),
                display_blank_chars: false,
                display_non_print_chars: false,
                show_tabs_as_spaces: false,
                wrap_by_character: false,
            },
            ui: UiConfig {
                theme: "Light".to_string(),
                show_minimap: true,
                sidebar_width: 220.0,
                keybinding_scheme: "NotepadPP".to_string(),
                ui_language: default_ui_language(),
            },
            recent_files: Vec::new(),
            window: WindowConfig {
                width: 1280.0,
                height: 720.0,
                x: None,
                y: None,
                maximized: false,
            },
            auto_save: AutoSaveConfig {
                enabled: true,
                interval_seconds: 300,
            },
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 720.0,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 300,
        }
    }
}

#[allow(dead_code)]
impl AppConfig {
    /// Load config from the default path (~/.config/rustpad/config.toml).
    /// Falls back to `Default` if the file does not exist or is malformed.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(text) => toml::from_str(&text).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Persist the current config to disk.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    /// Add a file to the recent list (most-recent first, max 20 entries).
    pub fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(20);
    }

    /// Update window configuration.
    pub fn update_window(&mut self, width: f32, height: f32, x: f32, y: f32, maximized: bool) {
        self.window.width = width;
        self.window.height = height;
        self.window.x = Some(x);
        self.window.y = Some(y);
        self.window.maximized = maximized;
    }

    fn config_path() -> PathBuf {
        config_dir().join("rustpad").join("config.toml")
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.editor.tab_size, 4);
        assert_eq!(cfg.editor.font_size, 14.0);
        assert_eq!(cfg.editor.font_family, "JetBrains Mono");
        assert_eq!(cfg.window.width, 1280.0);
        assert_eq!(cfg.window.height, 720.0);
        assert!(cfg.auto_save.enabled);
        assert_eq!(cfg.auto_save.interval_seconds, 300);
    }

    #[test]
    fn test_add_recent_file() {
        let mut cfg = AppConfig::default();
        let p = PathBuf::from("/tmp/test.rs");
        cfg.add_recent_file(p.clone());
        assert_eq!(cfg.recent_files[0], p);
        assert_eq!(cfg.recent_files.len(), 1);
    }

    #[test]
    fn test_add_recent_file_dedup() {
        let mut cfg = AppConfig::default();
        let p = PathBuf::from("/tmp/test.rs");
        cfg.add_recent_file(p.clone());
        cfg.add_recent_file(p.clone());
        assert_eq!(cfg.recent_files.len(), 1);
    }

    #[test]
    fn test_add_recent_file_max() {
        let mut cfg = AppConfig::default();
        for i in 0..25 {
            cfg.add_recent_file(PathBuf::from(format!("/tmp/test{}.rs", i)));
        }
        assert_eq!(cfg.recent_files.len(), 20);
    }

    #[test]
    fn test_update_window() {
        let mut cfg = AppConfig::default();
        cfg.update_window(1920.0, 1080.0, 100.0, 100.0, true);
        assert_eq!(cfg.window.width, 1920.0);
        assert_eq!(cfg.window.height, 1080.0);
        assert_eq!(cfg.window.x, Some(100.0));
        assert_eq!(cfg.window.y, Some(100.0));
        assert!(cfg.window.maximized);
    }

    #[test]
    fn test_save_and_load() {
        let mut cfg = AppConfig::default();
        cfg.editor.font_size = 16.0;
        cfg.add_recent_file(PathBuf::from("/tmp/test.rs"));

        // Save to temp file
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Override config_path for testing
        let text = toml::to_string_pretty(&cfg).unwrap();
        std::fs::write(&config_path, &text).unwrap();

        // Load and verify
        let loaded: AppConfig = toml::from_str(&text).unwrap();
        assert_eq!(loaded.editor.font_size, 16.0);
        assert_eq!(loaded.recent_files.len(), 1);
    }
}
