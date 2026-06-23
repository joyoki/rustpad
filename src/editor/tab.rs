use std::path::PathBuf;

use super::buffer::TextBuffer;
use super::context_actions::TabEditorExtras;
use super::cursor::{Cursor, Selection};
use super::{EncodingProfile, read_file_to_string};

/// Represents a single editor tab.
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub file_path: Option<PathBuf>,
    pub buffer: TextBuffer,
    pub cursor: Cursor,
    pub selection: Selection,
    pub scroll_offset: f32,
    pub encoding: EncodingProfile,
    pub column_selection: bool,
    pub modified: bool,
    /// Whether this tab is newly created (not yet saved).
    pub is_new: bool,
    /// Manually selected syntax/language name. When `None`, language is
    /// auto-detected from the file extension.
    pub syntax_override: Option<String>,
    /// Bookmarks, line marks, fold state for the context menu.
    pub editor_extras: TabEditorExtras,
}

impl Tab {
    /// Create a new empty tab with the given display title.
    pub fn new_with_title(id: usize, title: String) -> Self {
        Self {
            id,
            title,
            file_path: None,
            buffer: TextBuffer::new(),
            cursor: Cursor::default(),
            selection: Selection::default(),
            scroll_offset: 0.0,
            encoding: EncodingProfile::default(),
            column_selection: false,
            modified: false,
            is_new: true,
            syntax_override: None,
            editor_extras: TabEditorExtras::default(),
        }
    }

    /// Create a new empty tab (placeholder title; prefer `new_with_title`).
    pub fn new(id: usize) -> Self {
        Self::new_with_title(id, "untitled1.txt".to_string())
    }

    /// Open a file in this tab.
    pub fn open_file(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        let (text, encoding, line_ending) = read_file_to_string(path)?;
        self.buffer = TextBuffer::from_text(text, line_ending);
        self.file_path = Some(path.clone());
        self.title = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        self.encoding = encoding;
        self.cursor = Cursor::default();
        self.selection = Selection::default();
        self.scroll_offset = 0.0;
        self.modified = false;
        self.is_new = false;
        self.syntax_override = None;
        Ok(())
    }

    /// Save the current buffer to its file path.
    pub fn save(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.file_path {
            let bytes = self
                .encoding
                .encode_text(&self.buffer.text(), self.buffer.line_ending());
            std::fs::write(path, bytes)?;
            self.buffer.mark_clean();
            self.modified = false;
        }
        Ok(())
    }

    /// Save the buffer to a specific path.
    pub fn save_as(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = self
            .encoding
            .encode_text(&self.buffer.text(), self.buffer.line_ending());
        std::fs::write(path, bytes)?;
        self.file_path = Some(path.clone());
        self.title = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        self.buffer.mark_clean();
        self.modified = false;
        self.is_new = false;
        Ok(())
    }

    /// Re-decode the on-disk file using a specific encoding profile.
    pub fn reload_with_encoding(&mut self, profile: EncodingProfile) -> anyhow::Result<()> {
        let path = self
            .file_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No file path"))?
            .clone();
        let bytes = std::fs::read(&path)?;
        let text = profile.decode_bytes(&bytes);
        let line_ending = super::detect_line_ending(&text);
        self.buffer = TextBuffer::from_text(text, line_ending);
        self.encoding = profile;
        self.cursor = Cursor::default();
        self.selection = Selection::default();
        self.scroll_offset = 0.0;
        self.modified = false;
        self.buffer.mark_clean();
        Ok(())
    }

    /// Change the target save encoding without altering buffer text.
    pub fn convert_to_encoding(&mut self, profile: EncodingProfile) {
        if self.encoding != profile {
            self.encoding = profile;
            self.modified = true;
        }
    }

    /// Get the display title with a dirty indicator.
    pub fn display_title(&self) -> String {
        if self.buffer.is_dirty() || self.modified {
            format!("{} *", self.title)
        } else {
            self.title.clone()
        }
    }

    /// Get the line count of the buffer.
    pub fn line_count(&self) -> usize {
        self.buffer.line_count()
    }
}

/// Manages multiple editor tabs.
pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab: usize,
    next_id: usize,
}

#[allow(dead_code)]
impl TabManager {
    pub fn new() -> Self {
        let mut manager = Self {
            tabs: Vec::new(),
            active_tab: 0,
            next_id: 0,
        };
        // Start with one empty tab
        let id = manager.next_id;
        manager.next_id += 1;
        manager.tabs.push(Tab::new_with_title(id, "untitled1.txt".to_string()));
        manager
    }

    /// Create a tab manager restored from a previous session.
    pub fn from_session(paths: &[PathBuf]) -> Self {
        let mut manager = Self {
            tabs: Vec::new(),
            active_tab: 0,
            next_id: 0,
        };
        for path in paths {
            if path.exists() {
                let _ = manager.open_file(path);
            }
        }
        if manager.tabs.is_empty() {
            manager.new_tab();
        }
        manager
    }

    /// Pick the next unused `untitledN.txt` name among open new tabs.
    fn next_untitled_title(&self) -> String {
        let mut max_n = 0usize;
        for tab in &self.tabs {
            if tab.file_path.is_some() {
                continue;
            }
            let title = tab.title.as_str();
            if let Some(rest) = title.strip_prefix("untitled") {
                if let Some(num_str) = rest.strip_suffix(".txt") {
                    if let Ok(n) = num_str.parse::<usize>() {
                        max_n = max_n.max(n);
                    }
                }
            }
        }
        format!("untitled{}.txt", max_n + 1)
    }

    /// Create a new empty tab and make it active.
    pub fn new_tab(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let title = self.next_untitled_title();
        let tab = Tab::new_with_title(id, title);
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        id
    }

    /// Open a file in a new tab.
    pub fn open_file(&mut self, path: &PathBuf) -> anyhow::Result<usize> {
        // Check if file is already open
        for (i, tab) in self.tabs.iter().enumerate() {
            if tab.file_path.as_ref() == Some(path) {
                self.active_tab = i;
                return Ok(tab.id);
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        let mut tab = Tab::new(id);
        tab.open_file(path)?;
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        Ok(id)
    }

    /// Close a tab by index. Returns false if it's the last tab.
    pub fn close_tab(&mut self, index: usize) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        } else if self.active_tab > index {
            self.active_tab -= 1;
        }
        true
    }

    /// Get a reference to the active tab.
    pub fn active(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    /// Get a mutable reference to the active tab.
    pub fn active_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    /// Set the active tab by index.
    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
    }

    /// Get the active tab index.
    pub fn active_index(&self) -> usize {
        self.active_tab
    }

    /// Get all tabs.
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Get a mutable reference to all tabs.
    pub fn tabs_mut(&mut self) -> &mut [Tab] {
        &mut self.tabs
    }

    /// Get the number of tabs.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Check if any tab has unsaved changes.
    pub fn has_unsaved_changes(&self) -> bool {
        self.tabs.iter().any(|t| t.buffer.is_dirty() || t.modified)
    }

    /// Get indices of tabs with unsaved changes.
    pub fn unsaved_tab_indices(&self) -> Vec<usize> {
        self.tabs
            .iter()
            .enumerate()
            .filter(|(_, t)| t.buffer.is_dirty() || t.modified)
            .map(|(i, _)| i)
            .collect()
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tab() {
        let tm = TabManager::new();
        assert_eq!(tm.tab_count(), 1);
        assert_eq!(tm.active().title, "untitled1.txt");
    }

    #[test]
    fn test_untitled_increment() {
        let mut tm = TabManager::new();
        tm.new_tab();
        tm.new_tab();
        assert_eq!(tm.tabs()[0].title, "untitled1.txt");
        assert_eq!(tm.tabs()[1].title, "untitled2.txt");
        assert_eq!(tm.tabs()[2].title, "untitled3.txt");
    }

    #[test]
    fn test_close_last_tab_fails() {
        let mut tm = TabManager::new();
        assert!(!tm.close_tab(0));
    }

    #[test]
    fn test_multiple_tabs() {
        let mut tm = TabManager::new();
        tm.new_tab();
        tm.new_tab();
        assert_eq!(tm.tab_count(), 3);
        tm.set_active(1);
        assert_eq!(tm.active_index(), 1);
    }
}
