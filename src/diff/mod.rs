use serde::{Deserialize, Serialize};

pub mod engine;
pub mod folder_diff;
pub mod three_way;

pub use engine::DiffEngine;

/// A single diff line with its change type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub tag: DiffTag,
    pub line_index: usize,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DiffTag {
    #[default]
    Equal,
    Insert,
    Delete,
    Replace,
}

/// Word-level change within a line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordChange {
    pub old_start: usize,
    pub old_end: usize,
    pub new_start: usize,
    pub new_end: usize,
}

/// A hunk of related changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub hunk_type: DiffTag,
    pub lines: Vec<DiffLine>,
    pub word_changes: Vec<WordChange>,
}

/// One side-by-side aligned row in the comparison view.
///
/// Unlike `DiffLine` (a flat list), a row pairs the left (old) and right (new)
/// content so a replaced line shows old/new on the *same* row, and inserted /
/// deleted lines leave the opposite side empty. This is what enables a proper
/// Notepad-- style side-by-side view.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffRow {
    pub tag: DiffTag,
    /// 0-based source line number on the left side (None = no line here).
    pub left_line: Option<usize>,
    pub right_line: Option<usize>,
    /// Source cursor positions when the row was produced. These give a valid
    /// insertion index into each side even when that side has no line on this
    /// row (used by merge operations).
    pub left_at: usize,
    pub right_at: usize,
    pub left_text: Option<String>,
    pub right_text: Option<String>,
    /// Char ranges (start, end) changed inside the left/right text (for Replace).
    pub left_spans: Vec<(usize, usize)>,
    pub right_spans: Vec<(usize, usize)>,
    /// Index of the change block this row belongs to (None for equal rows).
    pub change_id: Option<usize>,
}

/// The complete result of a diff operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffResult {
    pub hunks: Vec<DiffHunk>,
    pub lines: Vec<DiffLine>,
    /// Side-by-side aligned rows for rendering.
    pub rows: Vec<DiffRow>,
    /// Row index at which each change block starts (for next/prev navigation).
    pub change_starts: Vec<usize>,
    pub stats: DiffStats,
}

impl DiffResult {
    /// Total number of change blocks (consecutive non-equal regions).
    pub fn change_count(&self) -> usize {
        self.change_starts.len()
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DiffStats {
    pub equal: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub replacements: usize,
}

/// Diff algorithm choice.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DiffAlgorithm {
    #[default]
    Myers,
    Patience,
    Lcs,
}

/// Diff options for controlling comparison behavior.
#[derive(Debug, Clone)]
pub struct DiffOptions {
    pub ignore_whitespace: bool,
    pub ignore_case: bool,
    pub ignore_line_endings: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            ignore_whitespace: false,
            ignore_case: false,
            ignore_line_endings: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_tag_equality() {
        assert_eq!(DiffTag::Equal, DiffTag::Equal);
        assert_ne!(DiffTag::Equal, DiffTag::Insert);
    }

    #[test]
    fn test_diff_stats_default() {
        let stats = DiffStats::default();
        assert_eq!(stats.equal, 0);
        assert_eq!(stats.insertions, 0);
        assert_eq!(stats.deletions, 0);
    }
}
