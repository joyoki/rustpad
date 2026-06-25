use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui;

/// A color theme for the editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorTheme {
    pub name: String,
    pub background: [u8; 4],
    pub foreground: [u8; 4],
    pub line_number_bg: [u8; 4],
    pub line_number_fg: [u8; 4],
    pub current_line_bg: [u8; 4],
    pub selection_bg: [u8; 4],
    pub cursor_color: [u8; 4],
    pub gutter_bg: [u8; 4],
    pub sidebar_bg: [u8; 4],
    pub sidebar_fg: [u8; 4],
    pub status_bar_bg: [u8; 4],
    pub status_bar_fg: [u8; 4],
    pub tab_bar_bg: [u8; 4],
    pub tab_active_bg: [u8; 4],
    pub tab_active_fg: [u8; 4],
    pub tab_inactive_bg: [u8; 4],
    pub tab_inactive_fg: [u8; 4],
    pub diff_insert_bg: [u8; 4],
    pub diff_delete_bg: [u8; 4],
    pub diff_replace_bg: [u8; 4],
    pub search_highlight_bg: [u8; 4],
    pub minimap_bg: [u8; 4],
    pub scroll_bar_bg: [u8; 4],
    pub scroll_bar_fg: [u8; 4],
}

impl EditorTheme {
    pub fn to_color32(rgba: [u8; 4]) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
    }

    pub fn background_color(&self) -> egui::Color32 {
        Self::to_color32(self.background)
    }

    pub fn foreground_color(&self) -> egui::Color32 {
        Self::to_color32(self.foreground)
    }

    pub fn selection_bg_color(&self) -> egui::Color32 {
        Self::to_color32(self.selection_bg)
    }

    pub fn current_line_bg_color(&self) -> egui::Color32 {
        Self::to_color32(self.current_line_bg)
    }

    pub fn cursor_color(&self) -> egui::Color32 {
        Self::to_color32(self.cursor_color)
    }

    pub fn diff_insert_color(&self) -> egui::Color32 {
        Self::to_color32(self.diff_insert_bg)
    }

    pub fn diff_delete_color(&self) -> egui::Color32 {
        Self::to_color32(self.diff_delete_bg)
    }

    pub fn diff_replace_color(&self) -> egui::Color32 {
        Self::to_color32(self.diff_replace_bg)
    }

    pub fn search_highlight_bg_color(&self) -> egui::Color32 {
        Self::to_color32(self.search_highlight_bg)
    }

    /// Brighter highlight for the active search match (Notepad++ style).
    pub fn search_current_highlight_bg_color(&self) -> egui::Color32 {
        let [r, g, b, _] = self.search_highlight_bg;
        Self::to_color32([r, g.saturating_sub(20), b, 240])
    }
}

/// Built-in dark theme (default).
pub fn dark_theme() -> EditorTheme {
    EditorTheme {
        name: "Dark".to_string(),
        background: [30, 30, 30, 255],
        foreground: [212, 212, 212, 255],
        line_number_bg: [30, 30, 30, 255],
        line_number_fg: [128, 128, 128, 255],
        current_line_bg: [40, 40, 40, 255],
        selection_bg: [38, 79, 120, 200],
        cursor_color: [212, 212, 212, 255],
        gutter_bg: [30, 30, 30, 255],
        sidebar_bg: [37, 37, 38, 255],
        sidebar_fg: [204, 204, 204, 255],
        status_bar_bg: [0, 122, 204, 255],
        status_bar_fg: [255, 255, 255, 255],
        tab_bar_bg: [45, 45, 48, 255],
        tab_active_bg: [30, 30, 30, 255],
        tab_active_fg: [255, 255, 255, 255],
        tab_inactive_bg: [45, 45, 48, 255],
        tab_inactive_fg: [160, 160, 160, 255],
        diff_insert_bg: [228, 255, 228, 255],
        diff_delete_bg: [255, 228, 228, 255],
        diff_replace_bg: [255, 251, 228, 255],
        search_highlight_bg: [255, 255, 0, 200],
        minimap_bg: [30, 30, 30, 255],
        scroll_bar_bg: [30, 30, 30, 255],
        scroll_bar_fg: [121, 121, 121, 255],
    }
}

/// Built-in light theme.
pub fn light_theme() -> EditorTheme {
    EditorTheme {
        name: "Light".to_string(),
        background: [255, 255, 255, 255],
        foreground: [30, 30, 30, 255],
        line_number_bg: [245, 245, 245, 255],
        line_number_fg: [150, 150, 150, 255],
        current_line_bg: [245, 245, 245, 255],
        selection_bg: [173, 214, 255, 200],
        cursor_color: [30, 30, 30, 255],
        gutter_bg: [245, 245, 245, 255],
        sidebar_bg: [240, 240, 240, 255],
        sidebar_fg: [50, 50, 50, 255],
        status_bar_bg: [0, 122, 204, 255],
        status_bar_fg: [255, 255, 255, 255],
        tab_bar_bg: [235, 235, 235, 255],
        tab_active_bg: [255, 255, 255, 255],
        tab_active_fg: [30, 30, 30, 255],
        tab_inactive_bg: [235, 235, 235, 255],
        tab_inactive_fg: [100, 100, 100, 255],
        diff_insert_bg: [228, 255, 228, 255],
        diff_delete_bg: [255, 228, 228, 255],
        diff_replace_bg: [255, 251, 200, 255],
        search_highlight_bg: [255, 255, 0, 200],
        minimap_bg: [245, 245, 245, 255],
        scroll_bar_bg: [245, 245, 245, 255],
        scroll_bar_fg: [180, 180, 180, 255],
    }
}

/// Built-in Monokai theme.
pub fn monokai_theme() -> EditorTheme {
    EditorTheme {
        name: "Monokai".to_string(),
        background: [39, 40, 34, 255],
        foreground: [248, 248, 242, 255],
        line_number_bg: [39, 40, 34, 255],
        line_number_fg: [100, 100, 100, 255],
        current_line_bg: [54, 56, 48, 255],
        selection_bg: [73, 72, 62, 200],
        cursor_color: [248, 248, 240, 255],
        gutter_bg: [39, 40, 34, 255],
        sidebar_bg: [34, 35, 30, 255],
        sidebar_fg: [248, 248, 242, 255],
        status_bar_bg: [166, 226, 46, 255],
        status_bar_fg: [39, 40, 34, 255],
        tab_bar_bg: [54, 56, 48, 255],
        tab_active_bg: [39, 40, 34, 255],
        tab_active_fg: [248, 248, 242, 255],
        tab_inactive_bg: [54, 56, 48, 255],
        tab_inactive_fg: [140, 140, 130, 255],
        diff_insert_bg: [166, 226, 46, 60],
        diff_delete_bg: [249, 38, 114, 60],
        diff_replace_bg: [253, 151, 31, 60],
        search_highlight_bg: [253, 151, 31, 200],
        minimap_bg: [39, 40, 34, 255],
        scroll_bar_bg: [39, 40, 34, 255],
        scroll_bar_fg: [73, 72, 62, 255],
    }
}

/// Built-in Solarized Dark theme.
pub fn solarized_dark_theme() -> EditorTheme {
    EditorTheme {
        name: "Solarized Dark".to_string(),
        background: [0, 43, 54, 255],
        foreground: [131, 148, 150, 255],
        line_number_bg: [0, 43, 54, 255],
        line_number_fg: [88, 110, 117, 255],
        current_line_bg: [7, 54, 66, 255],
        selection_bg: [7, 54, 66, 200],
        cursor_color: [203, 75, 22, 255],
        gutter_bg: [0, 43, 54, 255],
        sidebar_bg: [0, 38, 48, 255],
        sidebar_fg: [131, 148, 150, 255],
        status_bar_bg: [38, 139, 210, 255],
        status_bar_fg: [253, 246, 227, 255],
        tab_bar_bg: [7, 54, 66, 255],
        tab_active_bg: [0, 43, 54, 255],
        tab_active_fg: [131, 148, 150, 255],
        tab_inactive_bg: [7, 54, 66, 255],
        tab_inactive_fg: [88, 110, 117, 255],
        diff_insert_bg: [133, 153, 0, 60],
        diff_delete_bg: [220, 50, 47, 60],
        diff_replace_bg: [181, 137, 0, 60],
        search_highlight_bg: [181, 137, 0, 200],
        minimap_bg: [0, 43, 54, 255],
        scroll_bar_bg: [0, 43, 54, 255],
        scroll_bar_fg: [7, 54, 66, 255],
    }
}

/// Theme manager: built-in + custom themes loaded from disk.
pub struct ThemeManager {
    themes: HashMap<String, EditorTheme>,
    current_theme: String,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        themes.insert("Dark".to_string(), dark_theme());
        themes.insert("Light".to_string(), light_theme());
        themes.insert("Monokai".to_string(), monokai_theme());
        themes.insert("Solarized Dark".to_string(), solarized_dark_theme());

        let mut manager = Self {
            themes,
            current_theme: "Light".to_string(),
        };
        manager.load_custom_themes();
        manager
    }

    fn load_custom_themes(&mut self) {
        let themes_dir = Self::themes_dir();
        if !themes_dir.exists() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json") {
                    if let Ok(text) = std::fs::read_to_string(&path) {
                        if let Ok(theme) = serde_json::from_str::<EditorTheme>(&text) {
                            self.themes.insert(theme.name.clone(), theme);
                        }
                    }
                }
            }
        }
    }

    pub fn current_theme(&self) -> &EditorTheme {
        self.themes
            .get(&self.current_theme)
            .unwrap_or_else(|| self.themes.get("Dark").expect("Dark theme must exist"))
    }

    pub fn set_theme(&mut self, name: &str) {
        if self.themes.contains_key(name) {
            self.current_theme = name.to_string();
        }
    }

    pub fn theme_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.themes.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn current_theme_name(&self) -> &str {
        &self.current_theme
    }

    fn themes_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustpad")
            .join("themes")
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_themes() {
        let manager = ThemeManager::new();
        assert!(manager.themes.contains_key("Dark"));
        assert!(manager.themes.contains_key("Light"));
        assert!(manager.themes.contains_key("Monokai"));
        assert!(manager.themes.contains_key("Solarized Dark"));
    }

    #[test]
    fn test_set_theme() {
        let mut manager = ThemeManager::new();
        manager.set_theme("Monokai");
        assert_eq!(manager.current_theme_name(), "Monokai");
    }

    #[test]
    fn test_set_invalid_theme_ignored() {
        let mut manager = ThemeManager::new();
        let original = manager.current_theme_name().to_string();
        manager.set_theme("nonexistent_theme");
        assert_eq!(manager.current_theme_name(), original);
    }

    #[test]
    fn test_theme_names_sorted() {
        let manager = ThemeManager::new();
        let names = manager.theme_names();
        assert!(names.len() >= 4);
        for i in 1..names.len() {
            assert!(names[i - 1] <= names[i]);
        }
    }

    #[test]
    fn test_theme_serialization() {
        let theme = monokai_theme();
        let text = serde_json::to_string_pretty(&theme).unwrap();
        let loaded: EditorTheme = serde_json::from_str(&text).unwrap();
        assert_eq!(loaded.name, "Monokai");
        assert_eq!(loaded.background, theme.background);
    }
}
