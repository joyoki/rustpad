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

impl FoldState {
    pub fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    /// Detect foldable ranges from text content.
    /// Simple heuristic: detect lines ending with { and matching } lines.
    pub fn detect_folds(&mut self, text: &str) {
        self.ranges.clear();

        let lines: Vec<&str> = text.lines().collect();
        let mut stack: Vec<(usize, usize)> = Vec::new();
        let mut level = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.ends_with('{') || trimmed.ends_with('(') || trimmed.ends_with('[') {
                stack.push((i, level));
                level += 1;
            }

            if trimmed.starts_with('}') || trimmed.starts_with(')') || trimmed.starts_with(']') {
                if let Some((start_line, fold_level)) = stack.pop() {
                    level = level.saturating_sub(1);
                    if i > start_line {
                        self.ranges.push(FoldRange {
                            start_line,
                            end_line: i,
                            folded: false,
                            level: fold_level,
                        });
                    }
                }
            }
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

    /// Check if a line is folded (hidden).
    pub fn is_line_hidden(&self, line: usize) -> bool {
        for range in &self.ranges {
            if range.folded && line > range.start_line && line <= range.end_line {
                return true;
            }
        }
        false
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

    #[test]
    fn test_detect_folds() {
        let mut state = FoldState::new();
        let text = "fn main() {\n    let x = 1;\n    let y = 2;\n}\n";
        state.detect_folds(text);
        assert!(!state.ranges.is_empty());
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
}
