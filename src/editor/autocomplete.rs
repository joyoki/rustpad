use std::collections::HashMap;

/// A completion item to suggest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    /// The text to insert.
    pub text: String,
    /// Optional display label (different from text).
    pub label: String,
    /// Completion kind for icon display.
    pub kind: CompletionKind,
    /// Sort priority (lower = higher priority).
    pub priority: usize,
}

/// Type of completion item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Word,
    Bracket,
    Quote,
    Keyword,
    Function,
    Variable,
    Snippet,
}

/// Auto-completion engine.
pub struct AutocompleteEngine {
    /// Word frequency map for the current document.
    word_freq: HashMap<String, usize>,
    /// Whether auto-pairing is enabled.
    pub auto_pair: bool,
    /// Current completions being shown.
    pub completions: Vec<CompletionItem>,
    /// Selected completion index.
    pub selected_index: usize,
    /// Whether the completion popup is visible.
    pub visible: bool,
    /// Current prefix being completed.
    pub prefix: String,
    /// Auto-pair map.
    pair_map: HashMap<char, char>,
}

impl AutocompleteEngine {
    pub fn new() -> Self {
        let mut pair_map = HashMap::new();
        pair_map.insert('{', '}');
        pair_map.insert('(', ')');
        pair_map.insert('[', ']');
        pair_map.insert('"', '"');
        pair_map.insert('\'', '\'');
        pair_map.insert('`', '`');

        Self {
            word_freq: HashMap::new(),
            auto_pair: true,
            completions: Vec::new(),
            selected_index: 0,
            visible: false,
            prefix: String::new(),
            pair_map,
        }
    }

    /// Build word frequency map from text.
    pub fn build_from_text(&mut self, text: &str) {
        self.word_freq.clear();
        let mut current_word = String::new();

        for ch in text.chars() {
            if ch.is_alphanumeric() || ch == '_' {
                current_word.push(ch);
            } else {
                if current_word.len() >= 2 {
                    let word = current_word.clone();
                    *self.word_freq.entry(word).or_insert(0) += 1;
                }
                current_word.clear();
            }
        }
        if current_word.len() >= 2 {
            *self.word_freq.entry(current_word).or_insert(0) += 1;
        }
    }

    /// Get completions for a given prefix.
    pub fn get_completions(&mut self, prefix: &str) -> Vec<CompletionItem> {
        self.prefix = prefix.to_string();
        self.completions.clear();
        self.selected_index = 0;

        if prefix.is_empty() {
            self.visible = false;
            return Vec::new();
        }

        let prefix_lower = prefix.to_lowercase();
        let mut items: Vec<CompletionItem> = self
            .word_freq
            .iter()
            .filter(|(word, _)| {
                let word_lower = word.to_lowercase();
                word_lower.starts_with(&prefix_lower) && word_lower != prefix_lower
            })
            .map(|(word, freq)| CompletionItem {
                text: word.clone(),
                label: word.clone(),
                kind: CompletionKind::Word,
                priority: 1000 - freq.min(&999),
            })
            .collect();

        items.sort_by_key(|a| a.priority);
        items.truncate(20);

        self.visible = !items.is_empty();
        self.completions = items.clone();
        items
    }

    /// Move selection up.
    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down.
    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.completions.len() {
            self.selected_index += 1;
        }
    }

    /// Get the currently selected completion.
    pub fn selected_completion(&self) -> Option<&CompletionItem> {
        self.completions.get(self.selected_index)
    }

    /// Accept the current selection.
    pub fn accept(&mut self) -> Option<String> {
        if let Some(item) = self.completions.get(self.selected_index) {
            let text = item.text.clone();
            self.visible = false;
            self.completions.clear();
            Some(text)
        } else {
            None
        }
    }

    /// Cancel the completion popup.
    pub fn cancel(&mut self) {
        self.visible = false;
        self.completions.clear();
        self.selected_index = 0;
    }

    /// Get the auto-pair character for a given character.
    pub fn get_pair(&self, ch: char) -> Option<char> {
        self.pair_map.get(&ch).copied()
    }

    /// Check if a character is an opening bracket/quote.
    pub fn is_pair_opener(&self, ch: char) -> bool {
        self.pair_map.contains_key(&ch)
    }
}

impl Default for AutocompleteEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_word_freq() {
        let mut engine = AutocompleteEngine::new();
        engine.build_from_text("hello world hello foo bar foo foo");
        assert_eq!(engine.word_freq.get("hello"), Some(&2));
        assert_eq!(engine.word_freq.get("foo"), Some(&3));
    }

    #[test]
    fn test_get_completions() {
        let mut engine = AutocompleteEngine::new();
        engine.build_from_text("hello world help");
        let completions = engine.get_completions("hel");
        assert!(completions.iter().any(|c| c.text == "hello"));
        assert!(completions.iter().any(|c| c.text == "help"));
    }

    #[test]
    fn test_select_and_accept() {
        let mut engine = AutocompleteEngine::new();
        engine.build_from_text("hello world help");
        engine.get_completions("hel");
        engine.select_next();
        let accepted = engine.accept();
        assert!(accepted.is_some());
    }

    #[test]
    fn test_auto_pair() {
        let engine = AutocompleteEngine::new();
        assert_eq!(engine.get_pair('{'), Some('}'));
        assert_eq!(engine.get_pair('('), Some(')'));
        assert_eq!(engine.get_pair('a'), None);
    }

    #[test]
    fn test_cancel() {
        let mut engine = AutocompleteEngine::new();
        engine.build_from_text("hello world");
        engine.get_completions("hel");
        assert!(engine.visible);
        engine.cancel();
        assert!(!engine.visible);
    }
}
