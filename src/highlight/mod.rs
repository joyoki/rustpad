use std::collections::HashMap;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Supported themes.
pub const SUPPORTED_THEMES: &[&str] = &[
    "Monokai",
    "Solarized (dark)",
    "Solarized (light)",
    "base16-ocean.dark",
    "base16-ocean.light",
    "InspiredGitHub",
];

/// Cached highlight data for a single line.
#[derive(Debug, Clone)]
pub struct CachedLine {
    /// Styled spans: (style, text) pairs.
    pub spans: Vec<(Style, String)>,
    /// Whether this line needs re-highlighting.
    pub dirty: bool,
}

/// Highlight cache for a document.
#[derive(Debug)]
pub struct HighlightCache {
    /// Cached line data keyed by line index.
    lines: HashMap<usize, CachedLine>,
    /// The theme used for this cache.
    theme_name: String,
    /// The syntax name used for this cache.
    syntax_name: String,
}

impl HighlightCache {
    pub fn new(theme_name: String, syntax_name: String) -> Self {
        Self {
            lines: HashMap::new(),
            theme_name,
            syntax_name,
        }
    }

    /// Get a cached line if it exists and is not dirty.
    pub fn get(&self, line_index: usize) -> Option<&CachedLine> {
        self.lines.get(&line_index).filter(|l| !l.dirty)
    }

    /// Set a cached line.
    pub fn set(&mut self, line_index: usize, spans: Vec<(Style, String)>) {
        self.lines.insert(
            line_index,
            CachedLine {
                spans,
                dirty: false,
            },
        );
    }

    /// Mark a line as dirty (needs re-highlighting).
    pub fn mark_dirty(&mut self, line_index: usize) {
        if let Some(line) = self.lines.get_mut(&line_index) {
            line.dirty = true;
        }
    }

    /// Mark a range of lines as dirty.
    pub fn mark_dirty_range(&mut self, start: usize, end: usize) {
        for i in start..=end {
            self.mark_dirty(i);
        }
    }

    /// Invalidate all cached data.
    pub fn invalidate_all(&mut self) {
        for line in self.lines.values_mut() {
            line.dirty = true;
        }
    }

    /// Clear the cache completely.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Check if the cache is valid for the current theme and syntax.
    pub fn is_valid_for(&self, theme_name: &str, syntax_name: &str) -> bool {
        self.theme_name == theme_name && self.syntax_name == syntax_name
    }

    /// Get the number of cached lines.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// Manages syntax highlighting via syntect with caching.
pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
    cache: HighlightCache,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            current_theme: "InspiredGitHub".to_string(),
            cache: HighlightCache::new("base16-ocean.dark".to_string(), "Plain Text".to_string()),
        }
    }

    pub fn syntax_set(&self) -> &SyntaxSet {
        &self.syntax_set
    }

    pub fn theme_set(&self) -> &ThemeSet {
        &self.theme_set
    }

    pub fn current_theme(&self) -> &str {
        &self.current_theme
    }

    pub fn set_theme(&mut self, theme: &str) {
        if self.theme_set.themes.contains_key(theme) {
            self.current_theme = theme.to_string();
            self.cache.clear();
        }
    }

    pub fn theme(&self) -> &syntect::highlighting::Theme {
        self.theme_set
            .themes
            .get(&self.current_theme)
            .unwrap_or_else(|| {
                self.theme_set
                    .themes
                    .get("InspiredGitHub")
                    .expect("default theme must exist")
            })
    }

    /// List all available syntax/language names, sorted alphabetically.
    pub fn syntax_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .syntax_set
            .syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .filter(|n| !n.is_empty())
            .collect();
        names.sort();
        names.dedup();
        names
    }

    /// Detect syntax for a file based on extension.
    pub fn detect_syntax(&self, filename: &str) -> &syntect::parsing::SyntaxReference {
        if let Some(name) = extension_syntax_hint(filename) {
            if let Some(syntax) = self.syntax_set.find_syntax_by_name(name) {
                return syntax;
            }
        }
        self.syntax_set
            .find_syntax_for_file(filename)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
    }

    /// Effective syntax name for a tab (manual override or auto-detected from filename).
    pub fn syntax_name_for_file(&self, filename: &str, syntax_override: Option<&str>) -> String {
        if let Some(name) = syntax_override {
            return name.to_string();
        }
        self.detect_syntax(filename).name.clone()
    }

    /// Highlight a document by syntax name (avoids borrow conflicts).
    pub fn highlight_document_by_name(
        &mut self,
        text: &str,
        syntax_name: &str,
    ) -> Vec<Vec<(Style, String)>> {
        let theme_name = self.current_theme.clone();

        if !self.cache.is_valid_for(&theme_name, syntax_name) {
            self.cache = HighlightCache::new(theme_name, syntax_name.to_string());
        }

        let syntax = self
            .syntax_set
            .find_syntax_by_name(syntax_name)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let theme = self
            .theme_set
            .themes
            .get(&self.current_theme)
            .unwrap_or_else(|| {
                self.theme_set
                    .themes
                    .get("InspiredGitHub")
                    .expect("default theme must exist")
            });

        let mut h = HighlightLines::new(syntax, theme);
        let mut result = Vec::new();
        let mut spans_to_cache = Vec::new();

        let lines: Vec<String> = LinesWithEndings::from(text)
            .map(|s| s.to_string())
            .collect();
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(cached) = self.cache.get(line_idx) {
                result.push(cached.spans.clone());
                continue;
            }

            let ranges = h.highlight_line(line, &self.syntax_set).unwrap_or_default();
            let spans: Vec<(Style, String)> = ranges
                .into_iter()
                .map(|(s, t)| (s, t.to_string()))
                .collect();

            spans_to_cache.push((line_idx, spans.clone()));
            result.push(spans);
        }

        for (idx, spans) in spans_to_cache {
            self.cache.set(idx, spans);
        }

        result
    }

    /// Highlight a document with incremental caching.
    pub fn highlight_document(
        &mut self,
        text: &str,
        syntax: &syntect::parsing::SyntaxReference,
    ) -> Vec<Vec<(Style, String)>> {
        let syntax_name = syntax.name.clone();
        let theme_name = self.current_theme.clone();

        if !self.cache.is_valid_for(&theme_name, &syntax_name) {
            self.cache = HighlightCache::new(theme_name, syntax_name);
        }

        let theme = self
            .theme_set
            .themes
            .get(&self.current_theme)
            .unwrap_or_else(|| {
                self.theme_set
                    .themes
                    .get("InspiredGitHub")
                    .expect("default theme must exist")
            });

        let mut h = HighlightLines::new(syntax, theme);
        let mut result = Vec::new();
        let mut spans_to_cache = Vec::new();

        let lines: Vec<String> = LinesWithEndings::from(text)
            .map(|s| s.to_string())
            .collect();
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(cached) = self.cache.get(line_idx) {
                result.push(cached.spans.clone());
                continue;
            }

            let ranges = h.highlight_line(line, &self.syntax_set).unwrap_or_default();
            let spans: Vec<(Style, String)> = ranges
                .into_iter()
                .map(|(s, t)| (s, t.to_string()))
                .collect();

            spans_to_cache.push((line_idx, spans.clone()));
            result.push(spans);
        }

        for (idx, spans) in spans_to_cache {
            self.cache.set(idx, spans);
        }

        result
    }

    /// Highlight only specific visible lines with buffer.
    pub fn highlight_visible_lines(
        &mut self,
        text: &str,
        syntax: &syntect::parsing::SyntaxReference,
        first_line: usize,
        last_line: usize,
    ) -> Vec<(usize, Vec<(Style, String)>)> {
        let theme = self
            .theme_set
            .themes
            .get(&self.current_theme)
            .unwrap_or_else(|| {
                self.theme_set
                    .themes
                    .get("InspiredGitHub")
                    .expect("default theme must exist")
            });

        let mut h = HighlightLines::new(syntax, theme);
        let mut result = Vec::new();
        let mut spans_to_cache = Vec::new();

        let lines: Vec<String> = LinesWithEndings::from(text)
            .map(|s| s.to_string())
            .collect();
        for (line_idx, line) in lines.iter().enumerate() {
            if line_idx < first_line || line_idx > last_line {
                continue;
            }

            if let Some(cached) = self.cache.get(line_idx) {
                result.push((line_idx, cached.spans.clone()));
                continue;
            }

            let ranges = h.highlight_line(line, &self.syntax_set).unwrap_or_default();
            let spans: Vec<(Style, String)> = ranges
                .into_iter()
                .map(|(s, t)| (s, t.to_string()))
                .collect();

            spans_to_cache.push((line_idx, spans.clone()));
            result.push((line_idx, spans));
        }

        for (idx, spans) in spans_to_cache {
            self.cache.set(idx, spans);
        }

        result
    }

    /// Invalidate cache for a range of lines.
    pub fn invalidate_lines(&mut self, start: usize, end: usize) {
        self.cache.mark_dirty_range(start, end);
    }

    /// Invalidate the entire cache.
    pub fn invalidate_all(&mut self) {
        self.cache.invalidate_all();
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// List available theme names.
    pub fn theme_names(&self) -> Vec<&str> {
        self.theme_set.themes.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Common languages shown in the View > Language menu.
pub const MENU_LANGUAGES: &[&str] = &[
    "Plain Text",
    "C",
    "C++",
    "Java",
    "Markdown",
    "Rust",
    "Python",
    "JavaScript",
    "TypeScript",
    "Go",
    "HTML",
    "CSS",
    "JSON",
    "TOML",
    "YAML",
    "XML",
    "Bash",
    "SQL",
    "Ruby",
    "PHP",
];

/// Map common file extensions to syntect syntax names.
fn extension_syntax_hint(filename: &str) -> Option<&'static str> {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())?;

    let ext = ext.as_str();
    Some(match ext {
        "c" | "h" => "C",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "C++",
        "java" => "Java",
        "md" | "markdown" => "Markdown",
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "go" => "Go",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "json" => "JSON",
        "toml" => "TOML",
        "yaml" | "yml" => "YAML",
        "xml" => "XML",
        "sh" | "bash" => "Bash",
        "sql" => "SQL",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "kt" | "kts" => "Kotlin",
        "cs" => "C#",
        "txt" => "Plain Text",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_plain_text() {
        let mut hl = Highlighter::new();
        let lines = hl.highlight_document_by_name("hello\nworld\n", "Plain Text");
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_detect_syntax_rust() {
        let hl = Highlighter::new();
        let syntax = hl.detect_syntax("test.rs");
        assert_eq!(syntax.name, "Rust");
    }

    #[test]
    fn test_detect_syntax_java_and_markdown() {
        let hl = Highlighter::new();
        assert_eq!(hl.detect_syntax("Main.java").name, "Java");
        assert_eq!(hl.detect_syntax("README.md").name, "Markdown");
        assert_eq!(hl.detect_syntax("main.c").name, "C");
        assert_eq!(hl.detect_syntax("main.cpp").name, "C++");
    }

    #[test]
    fn test_set_invalid_theme_ignored() {
        let mut hl = Highlighter::new();
        let original = hl.current_theme().to_string();
        hl.set_theme("nonexistent_theme_12345");
        assert_eq!(hl.current_theme(), original);
    }

    #[test]
    fn test_cache_hit() {
        let mut hl = Highlighter::new();

        // First call populates cache
        let lines1 = hl.highlight_document_by_name("hello\nworld", "Plain Text");
        assert_eq!(lines1.len(), 2);

        // Second call should use cache
        let lines2 = hl.highlight_document_by_name("hello\nworld", "Plain Text");
        assert_eq!(lines2.len(), 2);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut hl = Highlighter::new();

        hl.highlight_document_by_name("hello\nworld", "Plain Text");
        assert!(!hl.cache.is_empty());

        hl.invalidate_lines(0, 1);
        // Cache entries exist but are dirty
    }

    #[test]
    fn test_highlight_by_name_various() {
        let mut hl = Highlighter::new();
        // Test with Rust syntax
        let lines = hl.highlight_document_by_name("fn main() {}", "Rust");
        assert!(!lines.is_empty());
        // Test with plain text
        let lines = hl.highlight_document_by_name("hello", "Plain Text");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_supported_themes() {
        assert!(!SUPPORTED_THEMES.is_empty());
        assert!(SUPPORTED_THEMES.contains(&"Monokai"));
    }
}
