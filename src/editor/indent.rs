/// Configuration for indentation behavior.
#[derive(Debug, Clone)]
pub struct IndentConfig {
    /// Use spaces instead of tabs.
    pub use_spaces: bool,
    /// Number of spaces per tab.
    pub tab_size: usize,
    /// Auto-indent on newline.
    pub auto_indent: bool,
    /// Increase indent after {, [, (.
    pub smart_indent: bool,
}

impl Default for IndentConfig {
    fn default() -> Self {
        Self {
            use_spaces: true,
            tab_size: 4,
            auto_indent: true,
            smart_indent: true,
        }
    }
}

/// Smart indentation engine.
pub struct IndentEngine {
    config: IndentConfig,
}

impl IndentEngine {
    pub fn new(config: IndentConfig) -> Self {
        Self { config }
    }

    /// Get the indentation string for a given level.
    pub fn indent_string(&self, level: usize) -> String {
        if self.config.use_spaces {
            " ".repeat(self.config.tab_size * level)
        } else {
            "\t".repeat(level)
        }
    }

    /// Get the indentation level of a line.
    pub fn line_indent_level(&self, line: &str) -> usize {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            return 0;
        }

        let indent_chars = line.len() - trimmed.len();
        if self.config.use_spaces {
            indent_chars / self.config.tab_size
        } else {
            indent_chars
        }
    }

    /// Get the leading whitespace of a line.
    pub fn leading_whitespace(&self, line: &str) -> String {
        let mut ws = String::new();
        for ch in line.chars() {
            if ch == ' ' || ch == '\t' {
                ws.push(ch);
            } else {
                break;
            }
        }
        ws
    }

    /// Calculate auto-indent for a new line after the given line.
    pub fn auto_indent_after(&self, prev_line: &str) -> String {
        if !self.config.auto_indent {
            return String::new();
        }

        let indent_level = self.line_indent_level(prev_line);
        let trimmed = prev_line.trim_end();

        // Check if previous line ends with indent-increasing characters
        let should_increase = if self.config.smart_indent {
            trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(')
        } else {
            false
        };

        let new_level = if should_increase {
            indent_level + 1
        } else {
            indent_level
        };

        self.indent_string(new_level)
    }

    /// Indent a single line by one level.
    pub fn indent_line(&self, line: &str) -> String {
        let indent = if self.config.use_spaces {
            " ".repeat(self.config.tab_size)
        } else {
            "\t".to_string()
        };
        format!("{}{}", indent, line)
    }

    /// Unindent a single line by one level.
    pub fn unindent_line(&self, line: &str) -> String {
        if line.is_empty() {
            return line.to_string();
        }

        let chars: Vec<char> = line.chars().collect();
        let mut remove_count = 0;

        if self.config.use_spaces {
            // Remove up to tab_size leading spaces
            for i in 0..chars.len().min(self.config.tab_size) {
                if chars[i] == ' ' {
                    remove_count += 1;
                } else {
                    break;
                }
            }
        } else {
            // Remove one leading tab
            if chars[0] == '\t' {
                remove_count = 1;
            }
        }

        if remove_count > 0 {
            chars[remove_count..].iter().collect()
        } else {
            line.to_string()
        }
    }

    /// Indent multiple lines (for Tab on selection).
    pub fn indent_lines(&self, lines: &[String]) -> Vec<String> {
        lines.iter().map(|line| self.indent_line(line)).collect()
    }

    /// Unindent multiple lines (for Shift+Tab on selection).
    pub fn unindent_lines(&self, lines: &[String]) -> Vec<String> {
        lines.iter().map(|line| self.unindent_line(line)).collect()
    }

    /// Convert tabs to spaces in a string.
    pub fn tabs_to_spaces(&self, text: &str) -> String {
        let spaces = " ".repeat(self.config.tab_size);
        text.replace('\t', &spaces)
    }

    /// Convert spaces to tabs in a string.
    pub fn spaces_to_tabs(&self, text: &str) -> String {
        let spaces = " ".repeat(self.config.tab_size);
        text.replace(&spaces, "\t")
    }

    /// Get the indentation for a new line, considering bracket completion.
    pub fn newline_indent(&self, prev_line: &str) -> (String, bool) {
        let indent = self.auto_indent_after(prev_line);
        let trimmed = prev_line.trim_end();

        // Check if we need to add a closing bracket line
        let needs_closing = if self.config.smart_indent {
            (trimmed.ends_with('{') && !trimmed.contains('}'))
                || (trimmed.ends_with('[') && !trimmed.contains(']'))
                || (trimmed.ends_with('(') && !trimmed.contains(')'))
        } else {
            false
        };

        (indent, needs_closing)
    }
}

impl Default for IndentEngine {
    fn default() -> Self {
        Self::new(IndentConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_string() {
        let engine = IndentEngine::default();
        assert_eq!(engine.indent_string(0), "");
        assert_eq!(engine.indent_string(1), "    ");
        assert_eq!(engine.indent_string(2), "        ");
    }

    #[test]
    fn test_line_indent_level() {
        let engine = IndentEngine::default();
        assert_eq!(engine.line_indent_level("hello"), 0);
        assert_eq!(engine.line_indent_level("    hello"), 1);
        assert_eq!(engine.line_indent_level("        hello"), 2);
    }

    #[test]
    fn test_auto_indent_after() {
        let engine = IndentEngine::default();
        // No increase
        assert_eq!(engine.auto_indent_after("hello"), "");
        assert_eq!(engine.auto_indent_after("    hello"), "    ");
        // Increase after {
        assert_eq!(engine.auto_indent_after("fn main() {"), "    ");
        assert_eq!(engine.auto_indent_after("    if true {"), "        ");
    }

    #[test]
    fn test_indent_unindent_line() {
        let engine = IndentEngine::default();
        assert_eq!(engine.indent_line("hello"), "    hello");
        assert_eq!(engine.unindent_line("    hello"), "hello");
        assert_eq!(engine.unindent_line("hello"), "hello");
    }

    #[test]
    fn test_indent_lines() {
        let engine = IndentEngine::default();
        let lines = vec!["hello".to_string(), "world".to_string()];
        let indented = engine.indent_lines(&lines);
        assert_eq!(indented[0], "    hello");
        assert_eq!(indented[1], "    world");
    }

    #[test]
    fn test_tabs_to_spaces() {
        let engine = IndentEngine::default();
        assert_eq!(engine.tabs_to_spaces("\thello"), "    hello");
    }

    #[test]
    fn test_smart_indent_config() {
        let config = IndentConfig {
            tab_size: 2,
            ..Default::default()
        };
        let engine = IndentEngine::new(config);
        assert_eq!(engine.indent_string(1), "  ");
    }
}
