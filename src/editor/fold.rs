use crate::editor::TextBuffer;

/// Represents a foldable range in the code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldRange {
    /// Start line (0-indexed, inclusive).
    pub start_line: usize,
    /// End line (0-indexed, inclusive).
    pub end_line: usize,
    /// Whether this range is currently folded.
    pub folded: bool,
    /// Nesting level (for indentation display).
    pub level: usize,
}

/// State of code folding.
#[derive(Debug, Clone)]
pub struct FoldState {
    /// All foldable ranges.
    pub ranges: Vec<FoldRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BracketEvent {
    Open(char),
    Close(char),
}

impl FoldState {
    pub fn new() -> Self {
        Self {
            ranges: Vec::new(),
        }
    }

    /// Split text the same way as [`TextBuffer`] (preserves trailing empty lines).
    pub fn split_lines(text: &str) -> Vec<&str> {
        if text.is_empty() {
            return vec![""];
        }
        text.split('\n').collect()
    }

    /// Detect foldable ranges from a [`TextBuffer`] (line indices match the editor).
    pub fn detect_folds_in_buffer(&mut self, buffer: &TextBuffer) {
        let line_count = buffer.line_count();
        let lines: Vec<&str> = (0..line_count)
            .map(|i| buffer.line(i).unwrap_or(""))
            .collect();
        self.detect_folds_in_lines(&lines);
    }

    /// Detect foldable ranges from text content.
    /// Preserves collapsed/expanded state for ranges that still exist.
    pub fn detect_folds(&mut self, text: &str) {
        let lines = Self::split_lines(text);
        self.detect_folds_in_lines(&lines);
    }

    fn detect_folds_in_lines(&mut self, lines: &[&str]) {
        use std::collections::HashMap;

        let preserved: HashMap<(usize, usize), bool> = self
            .ranges
            .iter()
            .map(|r| ((r.start_line, r.end_line), r.folded))
            .collect();

        self.ranges.clear();

        let mut stack: Vec<(usize, usize, char)> = Vec::new();
        let mut level = 0usize;

        for (i, line) in lines.iter().enumerate() {
            for event in line_bracket_events(line) {
                match event {
                    BracketEvent::Open(open_ch) => {
                        stack.push((i, level, open_ch));
                        level += 1;
                    }
                    BracketEvent::Close(close_ch) => {
                        while let Some((start_line, fold_level, open_ch)) = stack.pop() {
                            if matching_close(open_ch) == close_ch {
                                level = level.saturating_sub(1);
                                if i > start_line {
                                    let folded = preserved
                                        .get(&(start_line, i))
                                        .copied()
                                        .unwrap_or(false);
                                    self.ranges.push(FoldRange {
                                        start_line,
                                        end_line: i,
                                        folded,
                                        level: fold_level,
                                    });
                                }
                                break;
                            }
                            level = level.saturating_sub(1);
                        }
                    }
                }
            }
        }

        self.add_indent_folds(lines, &preserved);
    }

    /// Indent-based folds for languages without braces (and extra regions in braced code).
    fn add_indent_folds(&mut self, lines: &[&str], preserved: &std::collections::HashMap<(usize, usize), bool>) {
        if lines.is_empty() {
            return;
        }

        let indents: Vec<usize> = lines.iter().map(|line| line_indent(line)).collect();
        let mut i = 0usize;
        while i + 1 < lines.len() {
            let start_indent = indents[i];
            let next_indent = indents[i + 1];
            if next_indent <= start_indent || lines[i].trim().is_empty() {
                i += 1;
                continue;
            }

            let mut end = i + 1;
            while end + 1 < lines.len() && indents[end + 1] > start_indent {
                end += 1;
            }

            let overlaps_brace = self.ranges.iter().any(|r| {
                r.start_line <= i && end <= r.end_line
            });
            if !overlaps_brace && end > i {
                let folded = preserved.get(&(i, end)).copied().unwrap_or(false);
                self.ranges.push(FoldRange {
                    start_line: i,
                    end_line: end,
                    folded,
                    level: start_indent / 4,
                });
            }
            i = end + 1;
        }
    }

    /// Toggle fold state for a given line.
    pub fn toggle_fold(&mut self, line: usize) {
        for range in &mut self.ranges {
            if range.start_line == line {
                range.folded = !range.folded;
                return;
            }
        }
    }

    /// Foldable block start line for the given cursor line (if any).
    pub fn fold_start_for_line(&self, line: usize) -> Option<usize> {
        if self.ranges.iter().any(|r| r.start_line == line) {
            return Some(line);
        }
        self.ranges
            .iter()
            .filter(|r| r.start_line < line && line <= r.end_line)
            .max_by_key(|r| r.start_line)
            .map(|r| r.start_line)
    }

    /// Collapse the foldable block at `line` (or containing it).
    pub fn fold_at_line(&mut self, line: usize) {
        if let Some(start) = self.fold_start_for_line(line) {
            for range in &mut self.ranges {
                if range.start_line == start {
                    range.folded = true;
                    return;
                }
            }
        }
    }

    /// Expand every fold that contains `line`.
    pub fn unfold_at_line(&mut self, line: usize) {
        for range in &mut self.ranges {
            if range.start_line <= line && line <= range.end_line {
                range.folded = false;
            }
        }
    }

    /// Toggle fold at the block for `line`.
    pub fn toggle_fold_at_line(&mut self, line: usize) {
        if let Some(start) = self.fold_start_for_line(line) {
            self.toggle_fold(start);
        }
    }

    /// Check if a line is folded (hidden).
    pub fn is_line_hidden(&self, line: usize) -> bool {
        for range in &self.ranges {
            if range.folded && line > range.start_line && line <= range.end_line {
                return true;
            }
        }
        false
    }

    /// Number of buffer lines that remain visible.
    pub fn visible_line_count(&self, total_lines: usize) -> usize {
        (0..total_lines)
            .filter(|line| !self.is_line_hidden(*line))
            .count()
    }

    /// Map a buffer line to its index among visible lines (None if hidden).
    pub fn visible_line_index(&self, buffer_line: usize) -> Option<usize> {
        if self.is_line_hidden(buffer_line) {
            return None;
        }
        Some(
            (0..buffer_line)
                .filter(|line| !self.is_line_hidden(*line))
                .count(),
        )
    }

    /// Map a visible-line index back to a buffer line.
    pub fn buffer_line_at_visible_index(
        &self,
        visible_index: usize,
        total_lines: usize,
    ) -> Option<usize> {
        let mut seen = 0usize;
        for line in 0..total_lines {
            if self.is_line_hidden(line) {
                continue;
            }
            if seen == visible_index {
                return Some(line);
            }
            seen += 1;
        }
        None
    }

    /// Y position for a visible line index (after folds collapse hidden lines).
    pub fn visible_line_y(origin_y: f32, visible_index: usize, line_height: f32, scroll: f32) -> f32 {
        origin_y + visible_index as f32 * line_height - scroll
    }

    /// Get all foldable ranges that start at a given line.
    pub fn folds_at_line(&self, line: usize) -> Vec<&FoldRange> {
        self.ranges
            .iter()
            .filter(|r| r.start_line == line)
            .collect()
    }

    /// Check if a line is a fold start.
    pub fn is_fold_start(&self, line: usize) -> bool {
        self.ranges.iter().any(|r| r.start_line == line)
    }

    /// Get fold state (expanded/collapsed) for a line.
    pub fn fold_state(&self, line: usize) -> FoldStateIndicator {
        for range in &self.ranges {
            if range.start_line == line {
                if range.folded {
                    return FoldStateIndicator::Collapsed;
                } else {
                    return FoldStateIndicator::Expanded;
                }
            }
        }
        FoldStateIndicator::None
    }

    /// Fold range that begins on `line` (for gutter icons).
    pub fn range_at_start(&self, line: usize) -> Option<&FoldRange> {
        self.ranges.iter().find(|r| r.start_line == line)
    }

    /// Expanded ranges whose body still contains `line` (for scope guide lines).
    pub fn expanded_ranges_containing<'a>(&'a self, line: usize) -> Vec<&'a FoldRange> {
        self.ranges
            .iter()
            .filter(|r| !r.folded && r.start_line <= line && line <= r.end_line)
            .collect()
    }
}

fn matching_close(open: char) -> char {
    match open {
        '{' => '}',
        '(' => ')',
        '[' => ']',
        _ => '}',
    }
}

fn line_indent(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

/// Collect bracket open/close events left-to-right, ignoring strings and comments.
fn line_bracket_events(line: &str) -> Vec<BracketEvent> {
    let mut in_double = false;
    let mut in_single = false;
    let mut escape = false;
    let mut events = Vec::new();

    for (idx, c) in line.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if in_double {
            if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_double = false;
            }
            continue;
        }
        if in_single {
            if c == '\\' {
                escape = true;
            } else if c == '\'' {
                in_single = false;
            }
            continue;
        }

        if c == '/' && line[idx..].starts_with("//") {
            break;
        }
        if c == '#' {
            break;
        }

        match c {
            '"' => in_double = true,
            '\'' => in_single = true,
            '{' => events.push(BracketEvent::Open(c)),
            '}' => events.push(BracketEvent::Close(c)),
            _ => {}
        }
    }

    events
}

impl Default for FoldState {
    fn default() -> Self {
        Self::new()
    }
}

/// Indicator for fold state in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldStateIndicator {
    None,
    Expanded,
    Collapsed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{LineEnding, TextBuffer};

    #[test]
    fn test_detect_folds() {
        let mut state = FoldState::new();
        let text = "fn main() {\n    let x = 1;\n    let y = 2;\n}\n";
        state.detect_folds(text);
        assert!(!state.ranges.is_empty());
    }

    #[test]
    fn test_detect_folds_brace_on_next_line() {
        let mut state = FoldState::new();
        let text = "fn main()\n{\n    let x = 1;\n}\n";
        state.detect_folds(text);
        assert!(state.ranges.iter().any(|r| r.start_line == 1 && r.end_line == 3));
    }

    #[test]
    fn test_detect_folds_ignores_braces_in_strings() {
        let mut state = FoldState::new();
        let text = "fn main() {\n    let s = \"}\";\n    let x = 1;\n}\n";
        state.detect_folds(text);
        let outer = state
            .ranges
            .iter()
            .find(|r| r.start_line == 0)
            .expect("outer block");
        assert_eq!(outer.end_line, 3);
    }

    #[test]
    fn test_detect_folds_buffer_line_count_matches() {
        let mut state = FoldState::new();
        let buffer = TextBuffer::from_text("fn main() {\n}\n".to_string(), LineEnding::Lf);
        assert_eq!(buffer.line_count(), 3);
        state.detect_folds_in_buffer(&buffer);
        assert!(!state.ranges.is_empty());
        // `text.lines()` would only yield 1 line here; buffer-aware detection must still work.
        let mut naive = FoldState::new();
        naive.detect_folds("fn main() {\n}\n");
        assert!(
            state.ranges.len() >= naive.ranges.len(),
            "buffer-aware folds should not miss trailing-line blocks"
        );
    }

    #[test]
    fn test_fold_at_line_inside_block() {
        let mut state = FoldState::new();
        let text = "fn main() {\n    let x = 1;\n}\n";
        state.detect_folds(text);
        state.fold_at_line(1);
        assert!(state.is_line_hidden(1));
        assert!(!state.is_line_hidden(0));
    }

    #[test]
    fn test_toggle_fold() {
        let mut state = FoldState::new();
        state.ranges.push(FoldRange {
            start_line: 0,
            end_line: 3,
            folded: false,
            level: 0,
        });
        state.toggle_fold(0);
        assert!(state.ranges[0].folded);
        state.toggle_fold(0);
        assert!(!state.ranges[0].folded);
    }

    #[test]
    fn test_is_line_hidden() {
        let mut state = FoldState::new();
        state.ranges.push(FoldRange {
            start_line: 0,
            end_line: 3,
            folded: true,
            level: 0,
        });
        assert!(!state.is_line_hidden(0));
        assert!(state.is_line_hidden(1));
        assert!(state.is_line_hidden(2));
        assert!(state.is_line_hidden(3));
    }

    #[test]
    fn test_fold_state_indicator() {
        let mut state = FoldState::new();
        state.ranges.push(FoldRange {
            start_line: 0,
            end_line: 3,
            folded: false,
            level: 0,
        });
        assert_eq!(state.fold_state(0), FoldStateIndicator::Expanded);
        state.toggle_fold(0);
        assert_eq!(state.fold_state(0), FoldStateIndicator::Collapsed);
        assert_eq!(state.fold_state(1), FoldStateIndicator::None);
    }

    #[test]
    fn test_detect_folds_preserves_collapsed_state() {
        let mut state = FoldState::new();
        let text = "fn main() {\n    let x = 1;\n}\n";
        state.detect_folds(text);
        state.toggle_fold(0);
        state.detect_folds(text);
        assert!(state.ranges.iter().any(|r| r.start_line == 0 && r.folded));
    }

    #[test]
    fn test_visible_line_layout_when_folded() {
        let mut state = FoldState::new();
        state.ranges.push(FoldRange {
            start_line: 0,
            end_line: 3,
            folded: true,
            level: 0,
        });
        assert_eq!(state.visible_line_count(4), 1);
        assert_eq!(state.visible_line_index(0), Some(0));
        assert_eq!(state.visible_line_index(1), None);
        assert_eq!(state.buffer_line_at_visible_index(0, 4), Some(0));
    }

    #[test]
    fn test_indent_folds() {
        let mut state = FoldState::new();
        let text = "def foo():\n    pass\n    pass\n\ndef bar():\n";
        state.detect_folds(text);
        assert!(state.ranges.iter().any(|r| r.start_line == 0 && r.end_line >= 2));
    }
}
