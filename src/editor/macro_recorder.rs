use serde::{Deserialize, Serialize};
use std::time::Instant;

/// A recorded macro action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroAction {
    /// Insert text at cursor position.
    InsertText(String),
    /// Delete text at cursor position.
    DeleteText(usize), // count of chars
    /// Move cursor.
    MoveCursor(MoveDirection),
    /// Select text.
    SelectText(SelectAction),
    /// Newline.
    Newline,
    /// Undo.
    Undo,
    /// Redo.
    Redo,
}

/// Cursor movement direction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MoveDirection {
    Left,
    Right,
    Up,
    Down,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
}

/// Selection action.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SelectAction {
    All,
    Word,
    Line,
}

/// Macro recorder state.
#[derive(Debug)]
pub struct MacroRecorder {
    /// Whether currently recording.
    pub recording: bool,
    /// Current macro actions.
    actions: Vec<MacroAction>,
    /// Named macros (name -> actions).
    pub macros: Vec<NamedMacro>,
    /// Start time of recording.
    start_time: Option<Instant>,
}

/// A named macro.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedMacro {
    pub name: String,
    pub actions: Vec<MacroAction>,
}

impl MacroRecorder {
    pub fn new() -> Self {
        Self {
            recording: false,
            actions: Vec::new(),
            macros: Vec::new(),
            start_time: None,
        }
    }

    /// Start recording a new macro.
    pub fn start_recording(&mut self) {
        self.recording = true;
        self.actions.clear();
        self.start_time = Some(Instant::now());
    }

    /// Stop recording.
    pub fn stop_recording(&mut self) -> Vec<MacroAction> {
        self.recording = false;
        self.start_time = None;
        self.actions.clone()
    }

    /// Record an action.
    pub fn record_action(&mut self, action: MacroAction) {
        if self.recording {
            self.actions.push(action);
        }
    }

    /// Save the current macro with a name.
    pub fn save_macro(&mut self, name: String) {
        if !self.actions.is_empty() {
            let macro_entry = NamedMacro {
                name,
                actions: self.actions.clone(),
            };
            self.macros.push(macro_entry);
        }
    }

    /// Get a named macro by name.
    pub fn get_macro(&self, name: &str) -> Option<&NamedMacro> {
        self.macros.iter().find(|m| m.name == name)
    }

    /// Delete a named macro.
    pub fn delete_macro(&mut self, name: &str) -> bool {
        let len_before = self.macros.len();
        self.macros.retain(|m| m.name != name);
        self.macros.len() < len_before
    }

    /// Get all macro names.
    pub fn macro_names(&self) -> Vec<&str> {
        self.macros.iter().map(|m| m.name.as_str()).collect()
    }

    /// Check if currently recording.
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Get current recording actions (for display).
    pub fn current_actions(&self) -> &[MacroAction] {
        &self.actions
    }
}

impl Default for MacroRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_lifecycle() {
        let mut recorder = MacroRecorder::new();
        assert!(!recorder.is_recording());

        recorder.start_recording();
        assert!(recorder.is_recording());

        recorder.record_action(MacroAction::InsertText("hello".to_string()));
        recorder.record_action(MacroAction::Newline);
        recorder.record_action(MacroAction::InsertText("world".to_string()));

        let actions = recorder.stop_recording();
        assert!(!recorder.is_recording());
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn test_save_and_get_macro() {
        let mut recorder = MacroRecorder::new();
        recorder.start_recording();
        recorder.record_action(MacroAction::InsertText("test".to_string()));
        recorder.stop_recording();

        recorder.save_macro("test_macro".to_string());
        assert_eq!(recorder.macro_names().len(), 1);
        assert!(recorder.get_macro("test_macro").is_some());
        assert!(recorder.get_macro("nonexistent").is_none());
    }

    #[test]
    fn test_delete_macro() {
        let mut recorder = MacroRecorder::new();
        recorder.start_recording();
        recorder.record_action(MacroAction::InsertText("test".to_string()));
        recorder.stop_recording();

        recorder.save_macro("test_macro".to_string());
        assert!(recorder.delete_macro("test_macro"));
        assert!(recorder.get_macro("test_macro").is_none());
    }
}
