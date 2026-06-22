use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// Persisted editor session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Open files in tab order.
    pub open_files: Vec<PathBuf>,
    /// Index of the active tab.
    pub active_tab: usize,
    /// Cursor positions per file (keyed by path).
    pub cursor_positions: HashMap<String, (usize, usize)>,
    /// Scroll positions per file.
    pub scroll_positions: HashMap<String, f32>,
    /// Window geometry.
    pub window_width: f32,
    pub window_height: f32,
    /// Whether the window was maximized.
    pub maximized: bool,
    /// Workspace root directory.
    pub workspace_root: Option<PathBuf>,
    /// Recent files (most recent first).
    pub recent_files: Vec<PathBuf>,
    /// Max recent files.
    max_recent: usize,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            open_files: Vec::new(),
            active_tab: 0,
            cursor_positions: HashMap::new(),
            scroll_positions: HashMap::new(),
            window_width: 1280.0,
            window_height: 720.0,
            maximized: false,
            workspace_root: None,
            recent_files: Vec::new(),
            max_recent: 20,
        }
    }
}

impl Session {
    /// Load session from the default path.
    pub fn load() -> Self {
        let path = Self::session_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save session to the default path.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::session_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    /// Add a file to the recent files list.
    pub fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > self.max_recent {
            self.recent_files.truncate(self.max_recent);
        }
    }

    /// Update cursor position for a file.
    pub fn update_cursor(&mut self, path: &PathBuf, line: usize, col: usize) {
        let key = path.to_string_lossy().to_string();
        self.cursor_positions.insert(key, (line, col));
    }

    /// Get cursor position for a file.
    pub fn get_cursor(&self, path: &PathBuf) -> Option<(usize, usize)> {
        let key = path.to_string_lossy().to_string();
        self.cursor_positions.get(&key).copied()
    }

    /// Update scroll position for a file.
    pub fn update_scroll(&mut self, path: &PathBuf, scroll: f32) {
        let key = path.to_string_lossy().to_string();
        self.scroll_positions.insert(key, scroll);
    }

    /// Get scroll position for a file.
    pub fn get_scroll(&self, path: &PathBuf) -> Option<f32> {
        let key = path.to_string_lossy().to_string();
        self.scroll_positions.get(&key).copied()
    }

    /// Set workspace root.
    pub fn set_workspace(&mut self, root: PathBuf) {
        self.workspace_root = Some(root);
    }

    /// Get workspace root.
    pub fn workspace_root(&self) -> Option<&PathBuf> {
        self.workspace_root.as_ref()
    }

    fn session_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustpad")
            .join("session.json")
    }
}

/// Auto-save manager.
pub struct AutoSaveManager {
    /// Whether auto-save is enabled.
    pub enabled: bool,
    /// Interval in seconds.
    pub interval_secs: u64,
    /// Last save time.
    last_save: Instant,
    /// Crash recovery temp dir.
    crash_recovery_dir: PathBuf,
}

impl AutoSaveManager {
    pub fn new(enabled: bool, interval_secs: u64) -> Self {
        let crash_recovery_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustpad")
            .join("crash_recovery");

        Self {
            enabled,
            interval_secs,
            last_save: Instant::now(),
            crash_recovery_dir,
        }
    }

    /// Check if it's time to auto-save.
    pub fn should_save(&self) -> bool {
        self.enabled && self.last_save.elapsed().as_secs() >= self.interval_secs
    }

    /// Mark that a save was performed.
    pub fn mark_saved(&mut self) {
        self.last_save = Instant::now();
    }

    /// Write crash recovery data for a file.
    pub fn write_crash_recovery(&self, path: &PathBuf, content: &str) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.crash_recovery_dir)?;
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string());
        let recovery_path = self
            .crash_recovery_dir
            .join(format!("{}.recovery", filename));
        std::fs::write(&recovery_path, content)?;
        Ok(())
    }

    /// Check if crash recovery data exists for a file.
    pub fn has_crash_recovery(&self, path: &PathBuf) -> bool {
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string());
        let recovery_path = self
            .crash_recovery_dir
            .join(format!("{}.recovery", filename));
        recovery_path.exists()
    }

    /// Read crash recovery data.
    pub fn read_crash_recovery(&self, path: &PathBuf) -> anyhow::Result<String> {
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string());
        let recovery_path = self
            .crash_recovery_dir
            .join(format!("{}.recovery", filename));
        Ok(std::fs::read_to_string(&recovery_path)?)
    }

    /// Clean up crash recovery data for a file.
    pub fn cleanup_crash_recovery(&self, path: &PathBuf) -> anyhow::Result<()> {
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string());
        let recovery_path = self
            .crash_recovery_dir
            .join(format!("{}.recovery", filename));
        if recovery_path.exists() {
            std::fs::remove_file(&recovery_path)?;
        }
        Ok(())
    }
}

impl Default for AutoSaveManager {
    fn default() -> Self {
        Self::new(true, 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_session() {
        let s = Session::default();
        assert!(s.open_files.is_empty());
        assert_eq!(s.active_tab, 0);
    }

    #[test]
    fn test_recent_files() {
        let mut s = Session::default();
        s.add_recent_file(PathBuf::from("a.txt"));
        s.add_recent_file(PathBuf::from("b.txt"));
        assert_eq!(s.recent_files.len(), 2);
        assert_eq!(s.recent_files[0], PathBuf::from("b.txt"));

        // Duplicate should move to front
        s.add_recent_file(PathBuf::from("a.txt"));
        assert_eq!(s.recent_files.len(), 2);
        assert_eq!(s.recent_files[0], PathBuf::from("a.txt"));
    }

    #[test]
    fn test_cursor_positions() {
        let mut s = Session::default();
        let path = PathBuf::from("test.txt");
        s.update_cursor(&path, 10, 5);
        assert_eq!(s.get_cursor(&path), Some((10, 5)));
    }

    #[test]
    fn test_auto_save_manager() {
        let manager = AutoSaveManager::new(true, 60);
        assert!(!manager.should_save()); // Just created, not enough time passed
    }
}
