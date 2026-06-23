use serde::{Deserialize, Serialize};

use super::{DiffEngine, DiffResult, DiffTag};

/// A conflict region in three-way merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRegion {
    pub base_start: usize,
    pub base_end: usize,
    pub left_start: usize,
    pub left_end: usize,
    pub right_start: usize,
    pub right_end: usize,
    pub base_content: Vec<String>,
    pub left_content: Vec<String>,
    pub right_content: Vec<String>,
}

/// Result of a three-way merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeWayResult {
    pub merged_lines: Vec<String>,
    pub conflicts: Vec<ConflictRegion>,
    pub has_conflicts: bool,
}

/// Three-way merge engine.
pub struct ThreeWayMerge {
    engine: DiffEngine,
}

impl ThreeWayMerge {
    pub fn new() -> Self {
        Self {
            engine: DiffEngine::new(),
        }
    }

    /// Perform a three-way merge.
    pub fn merge(&self, base: &str, left: &str, right: &str) -> ThreeWayResult {
        let base_diff = self.engine.diff(base, left);
        let right_diff = self.engine.diff(base, right);

        let base_lines: Vec<&str> = base.lines().collect();
        let left_lines: Vec<&str> = left.lines().collect();
        let right_lines: Vec<&str> = right.lines().collect();

        let mut merged = Vec::new();
        let mut conflicts = Vec::new();

        let mut base_idx = 0;
        let mut left_idx = 0;
        let mut right_idx = 0;

        while base_idx < base_lines.len() || left_idx < left_lines.len() || right_idx < right_lines.len() {
            let left_changed = self.line_changed(&base_diff, base_idx);
            let right_changed = self.line_changed(&right_diff, base_idx);

            if !left_changed && !right_changed {
                if base_idx < base_lines.len() {
                    merged.push(base_lines[base_idx].to_string());
                }
                base_idx += 1;
                left_idx += 1;
                right_idx += 1;
            } else if left_changed && !right_changed {
                if left_idx < left_lines.len() {
                    merged.push(left_lines[left_idx].to_string());
                }
                left_idx += 1;
                base_idx += 1;
                right_idx += 1;
            } else if !left_changed && right_changed {
                if right_idx < right_lines.len() {
                    merged.push(right_lines[right_idx].to_string());
                }
                right_idx += 1;
                base_idx += 1;
                left_idx += 1;
            } else {
                let conflict = ConflictRegion {
                    base_start: base_idx,
                    base_end: base_idx + 1,
                    left_start: left_idx,
                    left_end: left_idx + 1,
                    right_start: right_idx,
                    right_end: right_idx + 1,
                    base_content: base_lines.get(base_idx..base_idx + 1)
                        .unwrap_or(&[])
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    left_content: left_lines.get(left_idx..left_idx + 1)
                        .unwrap_or(&[])
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    right_content: right_lines.get(right_idx..right_idx + 1)
                        .unwrap_or(&[])
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                };
                conflicts.push(conflict);

                if left_idx < left_lines.len() {
                    merged.push(left_lines[left_idx].to_string());
                }
                left_idx += 1;
                right_idx += 1;
                base_idx += 1;
            }
        }

        ThreeWayResult {
            merged_lines: merged,
            has_conflicts: !conflicts.is_empty(),
            conflicts,
        }
    }

    fn line_changed(&self, diff: &DiffResult, line_idx: usize) -> bool {
        for hunk in &diff.hunks {
            if hunk.hunk_type != DiffTag::Equal {
                for line in &hunk.lines {
                    if line.line_index == line_idx {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn accept_left(&self, conflict: &ConflictRegion) -> Vec<String> {
        conflict.left_content.clone()
    }

    pub fn accept_right(&self, conflict: &ConflictRegion) -> Vec<String> {
        conflict.right_content.clone()
    }

    pub fn accept_both(&self, conflict: &ConflictRegion) -> Vec<String> {
        let mut result = conflict.left_content.clone();
        result.extend(conflict.right_content.clone());
        result
    }
}

impl Default for ThreeWayMerge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_way_no_conflict() {
        let merger = ThreeWayMerge::new();
        let result = merger.merge("line1\nline2\n", "line1\nline2\n", "line1\nline2\n");
        assert!(!result.has_conflicts);
        assert_eq!(result.merged_lines, vec!["line1", "line2"]);
    }

    #[test]
    fn test_three_way_left_change() {
        let merger = ThreeWayMerge::new();
        let result = merger.merge("line1\nline2\n", "modified\nline2\n", "line1\nline2\n");
        assert!(!result.has_conflicts);
        assert_eq!(result.merged_lines[0], "modified");
    }
}
