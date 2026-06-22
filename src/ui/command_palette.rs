use serde::{Deserialize, Serialize};

/// A command entry in the command palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub shortcut: String,
    pub category: String,
}

/// The command palette state.
#[derive(Debug, Clone)]
pub struct CommandPalette {
    pub visible: bool,
    pub query: String,
    pub commands: Vec<CommandEntry>,
    pub filtered: Vec<CommandEntry>,
    pub selected_index: usize,
    pub recent: Vec<String>,
    max_recent: usize,
}

impl CommandPalette {
    pub fn new() -> Self {
        let commands = Self::default_commands();
        Self {
            visible: false,
            query: String::new(),
            commands,
            filtered: Vec::new(),
            selected_index: 0,
            recent: Vec::new(),
            max_recent: 10,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.query.clear();
            self.update_filter();
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.update_filter();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.query.clear();
    }

    pub fn set_query(&mut self, query: &str) {
        self.query = query.to_string();
        self.update_filter();
    }

    fn update_filter(&mut self) {
        self.selected_index = 0;
        let query_lower = self.query.to_lowercase();

        if query_lower.is_empty() {
            self.filtered = self.commands.clone();
        } else {
            self.filtered = self
                .commands
                .iter()
                .filter(|cmd| {
                    let name_lower = cmd.name.to_lowercase();
                    let desc_lower = cmd.description.to_lowercase();
                    fuzzy_match(&query_lower, &name_lower) || fuzzy_match(&query_lower, &desc_lower)
                })
                .cloned()
                .collect();
        }

        let recent = self.recent.clone();
        self.filtered.sort_by(|a, b| {
            let a_recent = recent.iter().position(|id| id == &a.id);
            let b_recent = recent.iter().position(|id| id == &b.id);
            match (a_recent, b_recent) {
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(a_idx), Some(b_idx)) => a_idx.cmp(&b_idx),
                (None, None) => a.name.cmp(&b.name),
            }
        });
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.filtered.len() {
            self.selected_index += 1;
        }
    }

    pub fn selected_command(&self) -> Option<&CommandEntry> {
        self.filtered.get(self.selected_index)
    }

    pub fn accept(&mut self) -> Option<String> {
        if let Some(cmd) = self.filtered.get(self.selected_index) {
            let id = cmd.id.clone();
            self.recent.retain(|r| r != &id);
            self.recent.insert(0, id.clone());
            if self.recent.len() > self.max_recent {
                self.recent.truncate(self.max_recent);
            }
            self.hide();
            Some(id)
        } else {
            None
        }
    }

    fn default_commands() -> Vec<CommandEntry> {
        vec![
            CommandEntry {
                id: "file.new".to_string(),
                name: "New File".to_string(),
                description: "Create a new empty file".to_string(),
                shortcut: "Ctrl+N".to_string(),
                category: "File".to_string(),
            },
            CommandEntry {
                id: "file.open".to_string(),
                name: "Open File".to_string(),
                description: "Open an existing file".to_string(),
                shortcut: "Ctrl+O".to_string(),
                category: "File".to_string(),
            },
            CommandEntry {
                id: "file.save".to_string(),
                name: "Save".to_string(),
                description: "Save the current file".to_string(),
                shortcut: "Ctrl+S".to_string(),
                category: "File".to_string(),
            },
            CommandEntry {
                id: "file.save_as".to_string(),
                name: "Save As".to_string(),
                description: "Save file with a new name".to_string(),
                shortcut: "Ctrl+Shift+S".to_string(),
                category: "File".to_string(),
            },
            CommandEntry {
                id: "edit.undo".to_string(),
                name: "Undo".to_string(),
                description: "Undo last action".to_string(),
                shortcut: "Ctrl+Z".to_string(),
                category: "Edit".to_string(),
            },
            CommandEntry {
                id: "edit.redo".to_string(),
                name: "Redo".to_string(),
                description: "Redo last undone action".to_string(),
                shortcut: "Ctrl+Y".to_string(),
                category: "Edit".to_string(),
            },
            CommandEntry {
                id: "edit.find".to_string(),
                name: "Find".to_string(),
                description: "Open find dialog".to_string(),
                shortcut: "Ctrl+F".to_string(),
                category: "Edit".to_string(),
            },
            CommandEntry {
                id: "edit.replace".to_string(),
                name: "Find and Replace".to_string(),
                description: "Open find and replace dialog".to_string(),
                shortcut: "Ctrl+H".to_string(),
                category: "Edit".to_string(),
            },
            CommandEntry {
                id: "view.toggle_sidebar".to_string(),
                name: "Toggle Sidebar".to_string(),
                description: "Show or hide the sidebar".to_string(),
                shortcut: "Ctrl+B".to_string(),
                category: "View".to_string(),
            },
            CommandEntry {
                id: "search.find_in_files".to_string(),
                name: "Find in Files".to_string(),
                description: "Search across multiple files".to_string(),
                shortcut: "Ctrl+Shift+F".to_string(),
                category: "Search".to_string(),
            },
            CommandEntry {
                id: "editor.goto_line".to_string(),
                name: "Go to Line".to_string(),
                description: "Jump to a specific line number".to_string(),
                shortcut: "Ctrl+G".to_string(),
                category: "Editor".to_string(),
            },
            CommandEntry {
                id: "theme.monokai".to_string(),
                name: "Theme: Monokai".to_string(),
                description: "Switch to Monokai theme".to_string(),
                shortcut: "".to_string(),
                category: "Theme".to_string(),
            },
            CommandEntry {
                id: "theme.github_dark".to_string(),
                name: "Theme: GitHub Dark".to_string(),
                description: "Switch to GitHub Dark theme".to_string(),
                shortcut: "".to_string(),
                category: "Theme".to_string(),
            },
        ]
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

fn fuzzy_match(pattern: &str, text: &str) -> bool {
    let mut pat_chars = pattern.chars();
    let mut current = pat_chars.next();
    for ch in text.chars() {
        if let Some(pc) = current {
            if ch == pc {
                current = pat_chars.next();
            }
        }
    }
    current.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("fb", "file.bar"));
        assert!(fuzzy_match("fb", "foo_bar"));
        assert!(!fuzzy_match("fb", "baz"));
    }

    #[test]
    fn test_toggle() {
        let mut palette = CommandPalette::new();
        assert!(!palette.visible);
        palette.toggle();
        assert!(palette.visible);
        palette.toggle();
        assert!(!palette.visible);
    }

    #[test]
    fn test_filter() {
        let mut palette = CommandPalette::new();
        palette.show();
        palette.set_query("save");
        assert!(palette.filtered.iter().any(|c| c.id == "file.save"));
        assert!(!palette.filtered.iter().any(|c| c.id == "file.open"));
    }

    #[test]
    fn test_navigation_and_accept() {
        let mut palette = CommandPalette::new();
        palette.show();
        palette.set_query("file");
        assert!(!palette.filtered.is_empty());
        palette.select_next();
        let accepted = palette.accept();
        assert!(accepted.is_some());
        assert!(!palette.visible);
    }
}
