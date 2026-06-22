use regex::Regex;
use serde::{Deserialize, Serialize};

/// A single search match result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    /// Character offset of the match start (matches `TextBuffer` positions).
    pub start: usize,
    /// Character offset of the match end (exclusive).
    pub end: usize,
    /// The matched text.
    pub text: String,
    /// Line number (0-based).
    pub line: usize,
    /// Column number (0-based).
    pub col: usize,
}

/// Search options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub wrap_around: bool,
    /// When true, "Find Next" searches backward (Notepad-- style).
    pub backward: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            wrap_around: true,
            backward: false,
        }
    }
}

/// The search engine for text search and replace.
pub struct SearchEngine {
    options: SearchOptions,
    last_results: Vec<SearchMatch>,
    current_index: Option<usize>,
    pattern: String,
}

#[allow(dead_code)]
impl SearchEngine {
    pub fn new() -> Self {
        Self {
            options: SearchOptions::default(),
            last_results: Vec::new(),
            current_index: None,
            pattern: String::new(),
        }
    }

    pub fn options(&self) -> &SearchOptions {
        &self.options
    }

    pub fn set_options(&mut self, options: SearchOptions) {
        self.options = options;
    }

    pub fn results(&self) -> &[SearchMatch] {
        &self.last_results
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn set_current_index(&mut self, index: Option<usize>) {
        self.current_index = index;
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Find all occurrences of `pattern` in `text`.
    pub fn find_all(&mut self, text: &str, pattern: &str) -> Vec<SearchMatch> {
        self.pattern = pattern.to_string();
        self.last_results.clear();
        self.current_index = None;

        if pattern.is_empty() {
            return Vec::new();
        }

        let matches = if self.options.use_regex {
            self.find_regex(text, pattern)
        } else {
            self.find_literal(text, pattern)
        };

        self.last_results = matches;
        self.last_results.clone()
    }

    fn find_literal(&self, text: &str, pattern: &str) -> Vec<SearchMatch> {
        let text_chars: Vec<char> = text.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();
        if pattern_chars.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        let mut i = 0usize;
        while i + pattern_chars.len() <= text_chars.len() {
            let slice = &text_chars[i..i + pattern_chars.len()];
            let matched = if self.options.case_sensitive {
                slice == pattern_chars.as_slice()
            } else {
                slice
                    .iter()
                    .zip(pattern_chars.iter())
                    .all(|(a, b)| a.eq_ignore_ascii_case(b))
            };

            if matched {
                if self.options.whole_word {
                    let before_ok =
                        i == 0 || !text_chars[i - 1].is_alphanumeric() && text_chars[i - 1] != '_';
                    let after = i + pattern_chars.len();
                    let after_ok = after >= text_chars.len()
                        || !text_chars[after].is_alphanumeric() && text_chars[after] != '_';
                    if !before_ok || !after_ok {
                        i += 1;
                        continue;
                    }
                }

                let (line, col) = line_col_for_char_pos(text, i);
                let matched_text: String = slice.iter().collect();
                matches.push(SearchMatch {
                    start: i,
                    end: i + pattern_chars.len(),
                    text: matched_text,
                    line,
                    col,
                });
                i += pattern_chars.len();
            } else {
                i += 1;
            }
        }

        matches
    }

    fn find_regex(&self, text: &str, pattern: &str) -> Vec<SearchMatch> {
        let flags = if self.options.case_sensitive {
            ""
        } else {
            "(?i)"
        };
        let full_pattern = format!("{}{}", flags, pattern);

        let re = match Regex::new(&full_pattern) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut matches = Vec::new();
        for m in re.find_iter(text) {
            let start = byte_offset_to_char_pos(text, m.start());
            let end = byte_offset_to_char_pos(text, m.end());
            let (line, col) = line_col_for_char_pos(text, start);
            matches.push(SearchMatch {
                start,
                end,
                text: m.as_str().to_string(),
                line,
                col,
            });
        }
        matches
    }

    /// Navigate to the next match. Returns the match if found.
    pub fn next_match(&mut self) -> Option<&SearchMatch> {
        if self.last_results.is_empty() {
            return None;
        }
        let idx = match self.current_index {
            Some(i) => {
                if i + 1 < self.last_results.len() {
                    i + 1
                } else if self.options.wrap_around {
                    0
                } else {
                    return None;
                }
            }
            None => 0,
        };
        self.current_index = Some(idx);
        self.last_results.get(idx)
    }

    /// Navigate to the previous match. Returns the match if found.
    pub fn prev_match(&mut self) -> Option<&SearchMatch> {
        if self.last_results.is_empty() {
            return None;
        }
        let idx = match self.current_index {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else if self.options.wrap_around {
                    self.last_results.len() - 1
                } else {
                    return None;
                }
            }
            None => self.last_results.len() - 1,
        };
        self.current_index = Some(idx);
        self.last_results.get(idx)
    }

    /// Replace the current match with `replacement`. Returns the new text for that match.
    pub fn replace_current(
        &self,
        _text: &str,
        replacement: &str,
    ) -> Option<(usize, usize, String)> {
        let idx = self.current_index?;
        let m = self.last_results.get(idx)?;

        let new_text = if self.options.use_regex {
            let flags = if self.options.case_sensitive {
                ""
            } else {
                "(?i)"
            };
            let full_pattern = format!("{}{}", flags, self.pattern);
            if let Ok(re) = Regex::new(&full_pattern) {
                re.replace(&m.text, replacement).to_string()
            } else {
                replacement.to_string()
            }
        } else {
            replacement.to_string()
        };

        Some((m.start, m.end, new_text))
    }

    /// Replace all matches. Returns the new text and the number of replacements made.
    pub fn replace_all(&self, text: &str, replacement: &str) -> (String, usize) {
        if self.last_results.is_empty() {
            return (text.to_string(), 0);
        }

        let count = self.last_results.len();

        if self.options.use_regex {
            let flags = if self.options.case_sensitive {
                ""
            } else {
                "(?i)"
            };
            let full_pattern = format!("{}{}", flags, self.pattern);
            if let Ok(re) = Regex::new(&full_pattern) {
                return (re.replace_all(text, replacement).to_string(), count);
            }
            return (text.to_string(), 0);
        }

        let mut chars: Vec<char> = text.chars().collect();
        let replacement_chars: Vec<char> = replacement.chars().collect();
        for m in self.last_results.iter().rev() {
            chars.splice(m.start..m.end, replacement_chars.iter().copied());
        }
        (chars.into_iter().collect(), count)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn byte_offset_to_char_pos(text: &str, byte: usize) -> usize {
    text[..byte.min(text.len())].chars().count()
}

/// Get (line, col) for a character offset in text.
fn line_col_for_char_pos(text: &str, char_pos: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    for (idx, ch) in text.chars().enumerate() {
        if idx >= char_pos {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_literal() {
        let mut engine = SearchEngine::new();
        let results = engine.find_all("hello world hello", "hello");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].start, 0);
        assert_eq!(results[1].start, 12);
    }

    #[test]
    fn test_find_case_insensitive() {
        let mut engine = SearchEngine::new();
        engine.set_options(SearchOptions {
            case_sensitive: false,
            ..Default::default()
        });
        let results = engine.find_all("Hello HELLO hello", "hello");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_find_regex() {
        let mut engine = SearchEngine::new();
        engine.set_options(SearchOptions {
            use_regex: true,
            ..Default::default()
        });
        let results = engine.find_all("foo123bar456", r"\d+");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_whole_word() {
        let mut engine = SearchEngine::new();
        engine.set_options(SearchOptions {
            whole_word: true,
            ..Default::default()
        });
        let results = engine.find_all("test testing test", "test");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_navigation() {
        let mut engine = SearchEngine::new();
        engine.find_all("a b a b a", "a");
        assert_eq!(engine.results().len(), 3);

        let m = engine.next_match().unwrap();
        assert_eq!(m.start, 0);

        let m = engine.next_match().unwrap();
        assert_eq!(m.start, 4);

        let m = engine.prev_match().unwrap();
        assert_eq!(m.start, 0);
    }

    #[test]
    fn test_replace_all_char_positions() {
        let mut engine = SearchEngine::new();
        engine.find_all("foo bar foo", "foo");
        let (new_text, count) = engine.replace_all("foo bar foo", "baz");
        assert_eq!(count, 2);
        assert_eq!(new_text, "baz bar baz");
    }
}
