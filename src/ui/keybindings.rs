use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use eframe::egui;

/// Keybinding scheme compatibility.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyScheme {
    #[default]
    NotepadPP,
    VSCode,
}

/// A single keybinding entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: String,
}

impl KeyBinding {
    pub fn new(ctrl: bool, shift: bool, alt: bool, key: &str) -> Self {
        Self {
            ctrl,
            shift,
            alt,
            key: key.to_string(),
        }
    }

    /// Check if this binding matches the current egui input state.
    pub fn matches(&self, input: &egui::InputState) -> bool {
        let modifiers = &input.modifiers;
        let ctrl_pressed = modifiers.command || modifiers.ctrl;

        if self.ctrl != ctrl_pressed {
            return false;
        }
        if self.shift != modifiers.shift {
            return false;
        }
        if self.alt != modifiers.alt {
            return false;
        }

        let egui_key = match self.key.as_str() {
            "A" => egui::Key::A,
            "B" => egui::Key::B,
            "C" => egui::Key::C,
            "D" => egui::Key::D,
            "E" => egui::Key::E,
            "F" => egui::Key::F,
            "G" => egui::Key::G,
            "H" => egui::Key::H,
            "I" => egui::Key::I,
            "J" => egui::Key::J,
            "K" => egui::Key::K,
            "L" => egui::Key::L,
            "M" => egui::Key::M,
            "N" => egui::Key::N,
            "O" => egui::Key::O,
            "P" => egui::Key::P,
            "Q" => egui::Key::Q,
            "R" => egui::Key::R,
            "S" => egui::Key::S,
            "T" => egui::Key::T,
            "U" => egui::Key::U,
            "V" => egui::Key::V,
            "W" => egui::Key::W,
            "X" => egui::Key::X,
            "Y" => egui::Key::Y,
            "Z" => egui::Key::Z,
            "Tab" => egui::Key::Tab,
            "Enter" => egui::Key::Enter,
            "Escape" => egui::Key::Escape,
            "F1" => egui::Key::F1,
            "F2" => egui::Key::F2,
            "F3" => egui::Key::F3,
            "F4" => egui::Key::F4,
            "F5" => egui::Key::F5,
            "F6" => egui::Key::F6,
            "F7" => egui::Key::F7,
            "F8" => egui::Key::F8,
            "F9" => egui::Key::F9,
            "F10" => egui::Key::F10,
            "F11" => egui::Key::F11,
            "F12" => egui::Key::F12,
            "Home" => egui::Key::Home,
            "End" => egui::Key::End,
            "PageUp" => egui::Key::PageUp,
            "PageDown" => egui::Key::PageDown,
            _ => return false,
        };

        input.key_pressed(egui_key)
    }

    /// Human-readable display string.
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.alt {
            parts.push("Alt");
        }
        parts.push(&self.key);
        parts.join("+")
    }
}

/// Commands that can be bound to keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Command {
    NewTab,
    OpenFile,
    Save,
    SaveAs,
    SaveAll,
    CloseTab,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Find,
    Replace,
    GotoLine,
    FindInFiles,
    ToggleSidebar,
    ToggleMinimap,
    ToggleDiffView,
    Palette,
    NextTab,
    PrevTab,
    MacroRecord,
    Preferences,
    Exit,
}

/// Manages all keybindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub scheme: KeyScheme,
    pub bindings: HashMap<Command, Vec<KeyBinding>>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::notepad_pp()
    }
}

impl KeyBindings {
    /// Notepad++ compatible keybindings.
    pub fn notepad_pp() -> Self {
        let mut bindings = HashMap::new();

        bindings.insert(Command::NewTab, vec![KeyBinding::new(true, false, false, "N")]);
        bindings.insert(Command::OpenFile, vec![KeyBinding::new(true, false, false, "O")]);
        bindings.insert(Command::Save, vec![KeyBinding::new(true, false, false, "S")]);
        bindings.insert(Command::SaveAs, vec![KeyBinding::new(true, true, false, "S")]);
        bindings.insert(Command::SaveAll, vec![KeyBinding::new(true, true, false, "A")]);
        bindings.insert(Command::CloseTab, vec![KeyBinding::new(true, false, false, "W")]);
        bindings.insert(Command::Undo, vec![KeyBinding::new(true, false, false, "Z")]);
        bindings.insert(Command::Redo, vec![KeyBinding::new(true, false, false, "Y")]);
        bindings.insert(Command::Cut, vec![KeyBinding::new(true, false, false, "X")]);
        bindings.insert(Command::Copy, vec![KeyBinding::new(true, false, false, "C")]);
        bindings.insert(Command::Paste, vec![KeyBinding::new(true, false, false, "V")]);
        bindings.insert(Command::SelectAll, vec![KeyBinding::new(true, false, false, "A")]);
        bindings.insert(Command::Find, vec![KeyBinding::new(true, false, false, "F")]);
        bindings.insert(Command::Replace, vec![KeyBinding::new(true, false, false, "H")]);
        bindings.insert(Command::GotoLine, vec![KeyBinding::new(true, false, false, "G")]);
        bindings.insert(Command::FindInFiles, vec![KeyBinding::new(true, true, false, "F")]);
        bindings.insert(Command::ToggleSidebar, vec![KeyBinding::new(true, false, false, "B")]);
        bindings.insert(Command::ToggleDiffView, vec![KeyBinding::new(true, false, false, "D")]);
        bindings.insert(Command::Palette, vec![KeyBinding::new(true, true, false, "P")]);
        bindings.insert(Command::NextTab, vec![KeyBinding::new(true, false, false, "Tab")]);
        bindings.insert(Command::PrevTab, vec![KeyBinding::new(true, true, false, "Tab")]);
        bindings.insert(
            Command::Exit,
            vec![
                KeyBinding::new(false, false, true, "F4"),
                KeyBinding::new(true, false, false, "Q"),
            ],
        );

        Self {
            scheme: KeyScheme::NotepadPP,
            bindings,
        }
    }

    /// VS Code compatible keybindings.
    pub fn vscode() -> Self {
        let mut bindings = HashMap::new();

        bindings.insert(Command::NewTab, vec![KeyBinding::new(true, false, false, "N")]);
        bindings.insert(Command::OpenFile, vec![KeyBinding::new(true, false, false, "O")]);
        bindings.insert(Command::Save, vec![KeyBinding::new(true, false, false, "S")]);
        bindings.insert(Command::SaveAs, vec![KeyBinding::new(true, true, false, "S")]);
        bindings.insert(Command::CloseTab, vec![KeyBinding::new(true, false, false, "W")]);
        bindings.insert(Command::Undo, vec![KeyBinding::new(true, false, false, "Z")]);
        bindings.insert(Command::Redo, vec![
            KeyBinding::new(true, true, false, "Z"),
            KeyBinding::new(true, false, false, "Y"),
        ]);
        bindings.insert(Command::Cut, vec![KeyBinding::new(true, false, false, "X")]);
        bindings.insert(Command::Copy, vec![KeyBinding::new(true, false, false, "C")]);
        bindings.insert(Command::Paste, vec![KeyBinding::new(true, false, false, "V")]);
        bindings.insert(Command::SelectAll, vec![KeyBinding::new(true, false, false, "A")]);
        bindings.insert(Command::Find, vec![KeyBinding::new(true, false, false, "F")]);
        bindings.insert(Command::Replace, vec![KeyBinding::new(true, false, false, "H")]);
        bindings.insert(Command::GotoLine, vec![KeyBinding::new(true, false, false, "G")]);
        bindings.insert(Command::FindInFiles, vec![KeyBinding::new(true, true, false, "F")]);
        bindings.insert(Command::ToggleSidebar, vec![KeyBinding::new(true, false, false, "B")]);
        bindings.insert(Command::ToggleDiffView, vec![KeyBinding::new(true, false, false, "D")]);
        bindings.insert(Command::Palette, vec![KeyBinding::new(true, true, false, "P")]);
        bindings.insert(Command::NextTab, vec![KeyBinding::new(true, false, false, "Tab")]);
        bindings.insert(Command::PrevTab, vec![KeyBinding::new(true, true, false, "Tab")]);
        bindings.insert(
            Command::Exit,
            vec![
                KeyBinding::new(false, false, true, "F4"),
                KeyBinding::new(true, false, false, "Q"),
            ],
        );

        Self {
            scheme: KeyScheme::VSCode,
            bindings,
        }
    }

    /// Check for a specific command being pressed.
    pub fn is_command_pressed(&self, command: &Command, input: &egui::InputState) -> bool {
        if let Some(bindings) = self.bindings.get(command) {
            bindings.iter().any(|b| b.matches(input))
        } else {
            false
        }
    }

    /// Load keybindings from JSON file.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                if let Ok(bindings) = serde_json::from_str(&text) {
                    return bindings;
                }
            }
        }
        Self::default()
    }

    /// Save keybindings to JSON file.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    /// Check for conflicts in current bindings.
    pub fn find_conflicts(&self) -> Vec<(Command, Command, KeyBinding)> {
        let mut conflicts = Vec::new();
        let entries: Vec<(&Command, &KeyBinding)> = self
            .bindings
            .iter()
            .flat_map(|(cmd, bindings)| bindings.iter().map(move |b| (cmd, b)))
            .collect();

        for (i, (cmd_a, binding_a)) in entries.iter().enumerate() {
            for (cmd_b, binding_b) in entries.iter().skip(i + 1) {
                if cmd_a != cmd_b
                    && binding_a.ctrl == binding_b.ctrl
                    && binding_a.shift == binding_b.shift
                    && binding_a.alt == binding_b.alt
                    && binding_a.key == binding_b.key
                {
                    conflicts.push(((*cmd_a).clone(), (*cmd_b).clone(), (*binding_a).clone()));
                }
            }
        }
        conflicts
    }

    fn config_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("rustpad")
            .join("keybindings.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bindings() {
        let kb = KeyBindings::default();
        assert!(kb.bindings.contains_key(&Command::NewTab));
        assert!(kb.bindings.contains_key(&Command::Save));
    }

    #[test]
    fn test_vscode_bindings() {
        let kb = KeyBindings::vscode();
        assert_eq!(kb.scheme, KeyScheme::VSCode);
        assert!(kb.bindings.contains_key(&Command::Redo));
    }

    #[test]
    fn test_key_binding_display() {
        let binding = KeyBinding::new(true, true, false, "S");
        assert_eq!(binding.display(), "Ctrl+Shift+S");
    }

    #[test]
    fn test_key_binding_display_alt() {
        let binding = KeyBinding::new(false, false, true, "F4");
        assert_eq!(binding.display(), "Alt+F4");
    }

    #[test]
    fn test_conflict_detection() {
        let mut kb = KeyBindings::default();
        // Add a conflicting binding
        kb.bindings.insert(
            Command::Save,
            vec![KeyBinding::new(true, false, false, "N")], // conflicts with NewTab
        );
        let conflicts = kb.find_conflicts();
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_no_conflict_default() {
        let kb = KeyBindings::default();
        let conflicts = kb.find_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_save_load_roundtrip() {
        let kb = KeyBindings::vscode();
        let text = serde_json::to_string_pretty(&kb).unwrap();
        let loaded: KeyBindings = serde_json::from_str(&text).unwrap();
        assert_eq!(loaded.scheme, KeyScheme::VSCode);
    }
}
