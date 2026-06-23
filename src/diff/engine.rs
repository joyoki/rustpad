use super::{
    DiffAlgorithm, DiffHunk, DiffLine, DiffOptions, DiffResult, DiffRow, DiffStats, DiffTag,
    WordChange,
};

/// Char-index spans `(start, end)` for the changed regions of one line.
type CharSpans = Vec<(usize, usize)>;

/// Diff engine using custom Myers/Patience/LCS algorithms.
pub struct DiffEngine {
    algorithm: DiffAlgorithm,
    options: DiffOptions,
}

impl DiffEngine {
    pub fn new() -> Self {
        Self {
            algorithm: DiffAlgorithm::default(),
            options: DiffOptions::default(),
        }
    }

    pub fn with_algorithm(mut self, algo: DiffAlgorithm) -> Self {
        self.algorithm = algo;
        self
    }

    pub fn with_options(mut self, options: DiffOptions) -> Self {
        self.options = options;
        self
    }

    pub fn set_algorithm(&mut self, algo: DiffAlgorithm) {
        self.algorithm = algo;
    }

    pub fn set_options(&mut self, options: DiffOptions) {
        self.options = options;
    }

    /// Compare two text strings and produce a diff result.
    pub fn diff(&self, old: &str, new: &str) -> DiffResult {
        let old_processed = self.preprocess(old);
        let new_processed = self.preprocess(new);

        let old_lines: Vec<&str> = old_processed.lines().collect();
        let new_lines: Vec<&str> = new_processed.lines().collect();

        let edit_script = match self.algorithm {
            DiffAlgorithm::Myers => myers_diff(&old_lines, &new_lines),
            DiffAlgorithm::Patience => patience_diff(&old_lines, &new_lines),
            DiffAlgorithm::Lcs => lcs_diff(&old_lines, &new_lines),
        };

        self.build_result(&old_lines, &new_lines, &edit_script)
    }

    fn build_result(
        &self,
        old_lines: &[&str],
        new_lines: &[&str],
        edit_script: &[EditOp],
    ) -> DiffResult {
        let mut lines = Vec::new();
        let mut hunks = Vec::new();
        let mut current_hunk: Option<DiffHunk> = None;
        let mut stats = DiffStats::default();

        let mut old_idx = 0;
        let mut new_idx = 0;

        for op in edit_script {
            match op {
                EditOp::Equal => {
                    stats.equal += 1;
                    let content = old_lines.get(old_idx).copied().unwrap_or_default().to_string();
                    lines.push(DiffLine {
                        tag: DiffTag::Equal,
                        line_index: lines.len(),
                        content,
                    });

                    if let Some(h) = current_hunk.take() {
                        hunks.push(h);
                    }

                    old_idx += 1;
                    new_idx += 1;
                }
                EditOp::Delete => {
                    stats.deletions += 1;
                    let content = old_lines.get(old_idx).copied().unwrap_or_default().to_string();
                    let diff_line = DiffLine {
                        tag: DiffTag::Delete,
                        line_index: lines.len(),
                        content,
                    };
                    lines.push(diff_line.clone());

                    let hunk = current_hunk.get_or_insert_with(|| DiffHunk {
                        old_start: old_idx,
                        old_count: 0,
                        new_start: new_idx,
                        new_count: 0,
                        hunk_type: DiffTag::Delete,
                        lines: Vec::new(),
                        word_changes: Vec::new(),
                    });
                    hunk.lines.push(diff_line);
                    hunk.old_count += 1;

                    old_idx += 1;
                }
                EditOp::Insert => {
                    stats.insertions += 1;
                    let content = new_lines.get(new_idx).copied().unwrap_or_default().to_string();
                    let diff_line = DiffLine {
                        tag: DiffTag::Insert,
                        line_index: lines.len(),
                        content,
                    };
                    lines.push(diff_line.clone());

                    let hunk = current_hunk.get_or_insert_with(|| DiffHunk {
                        old_start: old_idx,
                        old_count: 0,
                        new_start: new_idx,
                        new_count: 0,
                        hunk_type: DiffTag::Insert,
                        lines: Vec::new(),
                        word_changes: Vec::new(),
                    });
                    hunk.lines.push(diff_line);
                    hunk.new_count += 1;

                    new_idx += 1;
                }
                EditOp::Replace => {
                    stats.replacements += 1;

                    let old_content = old_lines.get(old_idx).copied().unwrap_or_default().to_string();
                    let new_content = new_lines.get(new_idx).copied().unwrap_or_default().to_string();

                    let word_changes = compute_word_changes(&old_content, &new_content);

                    let del_line = DiffLine {
                        tag: DiffTag::Delete,
                        line_index: lines.len(),
                        content: old_content,
                    };
                    lines.push(del_line.clone());

                    let ins_line = DiffLine {
                        tag: DiffTag::Insert,
                        line_index: lines.len(),
                        content: new_content,
                    };
                    lines.push(ins_line.clone());

                    let hunk = current_hunk.get_or_insert_with(|| DiffHunk {
                        old_start: old_idx,
                        old_count: 0,
                        new_start: new_idx,
                        new_count: 0,
                        hunk_type: DiffTag::Replace,
                        lines: Vec::new(),
                        word_changes,
                    });
                    hunk.lines.push(del_line);
                    hunk.lines.push(ins_line);
                    hunk.old_count += 1;
                    hunk.new_count += 1;

                    old_idx += 1;
                    new_idx += 1;
                }
            }
        }

        if let Some(h) = current_hunk.take() {
            hunks.push(h);
        }

        let (rows, change_starts) = build_aligned_rows(old_lines, new_lines, edit_script);

        DiffResult {
            hunks,
            lines,
            rows,
            change_starts,
            stats,
        }
    }

    /// Preprocess text based on options.
    fn preprocess(&self, text: &str) -> String {
        let mut result = text.to_string();

        if self.options.ignore_line_endings {
            result = result.replace("\r\n", "\n").replace('\r', "\n");
        }

        if self.options.ignore_whitespace {
            result = result
                .lines()
                .map(|l| l.trim())
                .collect::<Vec<_>>()
                .join("\n");
        }

        if self.options.ignore_case {
            result = result.to_lowercase();
        }

        result
    }

    /// Compare two files by path.
    pub fn diff_files(
        &self,
        old_path: &std::path::Path,
        new_path: &std::path::Path,
    ) -> anyhow::Result<DiffResult> {
        let old_text = std::fs::read_to_string(old_path)?;
        let new_text = std::fs::read_to_string(new_path)?;
        Ok(self.diff(&old_text, &new_text))
    }
}

impl Default for DiffEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Edit operation in a diff script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditOp {
    Equal,
    Insert,
    Delete,
    Replace,
}

/// Myers diff algorithm.
fn myers_diff(old: &[&str], new: &[&str]) -> Vec<EditOp> {
    let n = old.len();
    let m = new.len();
    let max = n + m;

    if max == 0 {
        return Vec::new();
    }

    let mut v = vec![0isize; 2 * max + 1];
    let offset = max as isize;
    let mut trace = Vec::new();

    for d in 0..=max {
        let mut new_v = vec![0isize; 2 * max + 1];
        new_v.clone_from_slice(&v);
        trace.push(new_v.clone());

        for k in (-(d as isize)..=d as isize).step_by(2) {
            let idx = (k + offset) as usize;

            let mut x = if k == -(d as isize) || (k != d as isize && v[(k - 1 + offset) as usize] < v[(k + 1 + offset) as usize]) {
                v[(k + 1 + offset) as usize]
            } else {
                v[(k - 1 + offset) as usize] + 1
            };

            let mut y = x - k;

            while x < n as isize && y < m as isize && old[x as usize] == new[y as usize] {
                x += 1;
                y += 1;
            }

            v[idx] = x;

            if x >= n as isize && y >= m as isize {
                return backtrack(&trace, old, new, offset);
            }
        }
    }

    Vec::new()
}

fn backtrack(trace: &[Vec<isize>], old: &[&str], new: &[&str], offset: isize) -> Vec<EditOp> {
    let mut ops = Vec::new();
    let mut x = old.len() as isize;
    let mut y = new.len() as isize;

    for d in (0..trace.len()).rev() {
        let v = &trace[d];
        let k = x - y;
        let _idx = (k + offset) as usize;

        let prev_k = if k == -(d as isize) || (k != d as isize && v[(k - 1 + offset) as usize] < v[(k + 1 + offset) as usize]) {
            k + 1
        } else {
            k - 1
        };

        let prev_x = v[(prev_k + offset) as usize];
        let prev_y = prev_x - prev_k;

        while x > prev_x && y > prev_y {
            ops.push(EditOp::Equal);
            x -= 1;
            y -= 1;
        }

        if d > 0 {
            if x == prev_x {
                ops.push(EditOp::Insert);
                y -= 1;
            } else {
                ops.push(EditOp::Delete);
                x -= 1;
            }
        }
    }

    ops.reverse();
    ops
}

/// Patience diff algorithm.
fn patience_diff(old: &[&str], new: &[&str]) -> Vec<EditOp> {
    let common = unique_common(old, new);

    if common.is_empty() {
        if old.is_empty() && new.is_empty() {
            return Vec::new();
        } else if old.is_empty() {
            return vec![EditOp::Insert; new.len()];
        } else if new.is_empty() {
            return vec![EditOp::Delete; old.len()];
        } else {
            return lcs_diff(old, new);
        }
    }

    let mut ops = Vec::new();
    let mut old_start = 0;
    let mut new_start = 0;

    for &(old_idx, new_idx) in &common {
        let old_chunk = &old[old_start..old_idx];
        let new_chunk = &new[new_start..new_idx];

        let sub_ops = if old_chunk.is_empty() && new_chunk.is_empty() {
            Vec::new()
        } else {
            myers_diff(old_chunk, new_chunk)
        };
        ops.extend(sub_ops);

        ops.push(EditOp::Equal);
        old_start = old_idx + 1;
        new_start = new_idx + 1;
    }

    let old_chunk = &old[old_start..];
    let new_chunk = &new[new_start..];
    let sub_ops = myers_diff(old_chunk, new_chunk);
    ops.extend(sub_ops);

    ops
}

fn unique_common<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<(usize, usize)> {
    use std::collections::HashMap;

    let mut new_positions: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, line) in new.iter().enumerate() {
        new_positions.entry(line).or_default().push(i);
    }

    let mut unique_old: HashMap<&str, usize> = HashMap::new();
    for (i, line) in old.iter().enumerate() {
        if let Some(pos) = new_positions.get(line) {
            if pos.len() == 1 {
                unique_old.insert(line, i);
            }
        }
    }

    let mut unique_new: HashMap<&str, usize> = HashMap::new();
    for (i, line) in new.iter().enumerate() {
        if unique_old.contains_key(line) {
            if let Some(new_pos) = new_positions.get(line) {
                if new_pos.len() == 1 {
                    unique_new.insert(line, i);
                }
            }
        }
    }

    let mut common: Vec<(usize, usize)> = unique_old
        .iter()
        .filter_map(|(&line, &old_idx)| {
            unique_new.get(line).map(|&new_idx| (old_idx, new_idx))
        })
        .collect();

    common.sort_by_key(|&(o, _)| o);
    common
}

/// LCS diff algorithm.
fn lcs_diff(old: &[&str], new: &[&str]) -> Vec<EditOp> {
    let lcs = longest_common_subsequence(old, new);
    let mut ops = Vec::new();
    let mut oi = 0;
    let mut ni = 0;
    let mut li = 0;

    while oi < old.len() || ni < new.len() {
        if li < lcs.len() && oi < old.len() && ni < new.len()
            && old[oi] == lcs[li] && new[ni] == lcs[li]
        {
            ops.push(EditOp::Equal);
            oi += 1;
            ni += 1;
            li += 1;
        } else if oi < old.len() && ni < new.len()
            && (li >= lcs.len() || old[oi] != lcs[li])
            && (li >= lcs.len() || new[ni] != lcs[li])
        {
            ops.push(EditOp::Replace);
            oi += 1;
            ni += 1;
        } else if oi < old.len() && (li >= lcs.len() || old[oi] != lcs[li]) {
            ops.push(EditOp::Delete);
            oi += 1;
        } else if ni < new.len() {
            ops.push(EditOp::Insert);
            ni += 1;
        } else {
            break;
        }
    }

    ops
}

fn longest_common_subsequence<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}

/// Build side-by-side aligned rows from an edit script.
///
/// Within each contiguous change block, buffered deletions and insertions are
/// paired positionally into `Replace` rows (so a modified line shows old/new on
/// the same row); any surplus becomes `Delete` or `Insert` rows with a gap on
/// the opposite side. Returns the rows and the starting row index of each block.
#[allow(unused_assignments)]
fn build_aligned_rows(
    old_lines: &[&str],
    new_lines: &[&str],
    edit_script: &[EditOp],
) -> (Vec<DiffRow>, Vec<usize>) {
    let mut rows: Vec<DiffRow> = Vec::new();
    let mut change_starts: Vec<usize> = Vec::new();
    let mut change_id: usize = 0;

    let mut old_idx = 0usize;
    let mut new_idx = 0usize;

    // Buffered (line_index, content) for the current change block.
    let mut dels: Vec<(usize, String)> = Vec::new();
    let mut inss: Vec<(usize, String)> = Vec::new();
    let mut block_old_start = 0usize;
    let mut block_new_start = 0usize;

    macro_rules! flush_block {
        () => {{
            if !dels.is_empty() || !inss.is_empty() {
                change_starts.push(rows.len());
                let n = dels.len().max(inss.len());
                for k in 0..n {
                    match (dels.get(k), inss.get(k)) {
                        (Some((ol, oc)), Some((nl, nc))) => {
                            let (ls, rs) = inline_char_spans(oc, nc);
                            rows.push(DiffRow {
                                tag: DiffTag::Replace,
                                left_line: Some(*ol),
                                right_line: Some(*nl),
                                left_at: *ol,
                                right_at: *nl,
                                left_text: Some(oc.clone()),
                                right_text: Some(nc.clone()),
                                left_spans: ls,
                                right_spans: rs,
                                change_id: Some(change_id),
                            });
                        }
                        (Some((ol, oc)), None) => {
                            rows.push(DiffRow {
                                tag: DiffTag::Delete,
                                left_line: Some(*ol),
                                right_line: None,
                                left_at: *ol,
                                right_at: block_new_start,
                                left_text: Some(oc.clone()),
                                right_text: None,
                                change_id: Some(change_id),
                                ..Default::default()
                            });
                        }
                        (None, Some((nl, nc))) => {
                            rows.push(DiffRow {
                                tag: DiffTag::Insert,
                                left_line: None,
                                right_line: Some(*nl),
                                left_at: block_old_start,
                                right_at: *nl,
                                left_text: None,
                                right_text: Some(nc.clone()),
                                change_id: Some(change_id),
                                ..Default::default()
                            });
                        }
                        (None, None) => {}
                    }
                }
                change_id += 1;
                dels.clear();
                inss.clear();
            }
        }};
    }

    for op in edit_script {
        match op {
            EditOp::Equal => {
                flush_block!();
                let content = old_lines.get(old_idx).copied().unwrap_or_default().to_string();
                rows.push(DiffRow {
                    tag: DiffTag::Equal,
                    left_line: Some(old_idx),
                    right_line: Some(new_idx),
                    left_at: old_idx,
                    right_at: new_idx,
                    left_text: Some(content.clone()),
                    right_text: Some(content),
                    change_id: None,
                    ..Default::default()
                });
                old_idx += 1;
                new_idx += 1;
            }
            EditOp::Delete => {
                if dels.is_empty() && inss.is_empty() {
                    block_old_start = old_idx;
                    block_new_start = new_idx;
                }
                let content = old_lines.get(old_idx).copied().unwrap_or_default().to_string();
                dels.push((old_idx, content));
                old_idx += 1;
            }
            EditOp::Insert => {
                if dels.is_empty() && inss.is_empty() {
                    block_old_start = old_idx;
                    block_new_start = new_idx;
                }
                let content = new_lines.get(new_idx).copied().unwrap_or_default().to_string();
                inss.push((new_idx, content));
                new_idx += 1;
            }
            EditOp::Replace => {
                if dels.is_empty() && inss.is_empty() {
                    block_old_start = old_idx;
                    block_new_start = new_idx;
                }
                let oc = old_lines.get(old_idx).copied().unwrap_or_default().to_string();
                let nc = new_lines.get(new_idx).copied().unwrap_or_default().to_string();
                dels.push((old_idx, oc));
                inss.push((new_idx, nc));
                old_idx += 1;
                new_idx += 1;
            }
        }
    }
    flush_block!();

    (rows, change_starts)
}

/// Compute char-level changed spans for two replaced lines.
///
/// Returns `(left_spans, right_spans)` where each span is a `(start, end)` char
/// index range that differs, so the UI can highlight exactly what changed inside
/// the line (Notepad-- style inline diff). Leading/trailing common runs are kept
/// unhighlighted.
fn inline_char_spans(old: &str, new: &str) -> (CharSpans, CharSpans) {
    let a: Vec<char> = old.chars().collect();
    let b: Vec<char> = new.chars().collect();

    // Common prefix length.
    let mut prefix = 0;
    while prefix < a.len() && prefix < b.len() && a[prefix] == b[prefix] {
        prefix += 1;
    }

    // Common suffix length (not overlapping the prefix).
    let mut suffix = 0;
    while suffix < a.len() - prefix
        && suffix < b.len() - prefix
        && a[a.len() - 1 - suffix] == b[b.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let left = if prefix + suffix < a.len() {
        vec![(prefix, a.len() - suffix)]
    } else {
        Vec::new()
    };
    let right = if prefix + suffix < b.len() {
        vec![(prefix, b.len() - suffix)]
    } else {
        Vec::new()
    };

    (left, right)
}

/// Compute word-level changes between two strings.
fn compute_word_changes(old: &str, new: &str) -> Vec<WordChange> {
    let old_words: Vec<&str> = old.split_whitespace().collect();
    let new_words: Vec<&str> = new.split_whitespace().collect();

    let lcs = longest_common_subsequence(&old_words, &new_words);
    let mut changes = Vec::new();
    let mut oi = 0;
    let mut ni = 0;
    let mut li = 0;

    while oi < old_words.len() || ni < new_words.len() {
        if li < lcs.len() && oi < old_words.len() && ni < new_words.len()
            && old_words[oi] == lcs[li] && new_words[ni] == lcs[li]
        {
            oi += 1;
            ni += 1;
            li += 1;
        } else {
            let change_start_old = oi;
            let change_start_new = ni;

            while oi < old_words.len() && (li >= lcs.len() || old_words[oi] != lcs[li]) {
                oi += 1;
            }
            while ni < new_words.len() && (li >= lcs.len() || new_words[ni] != lcs[li]) {
                ni += 1;
            }

            if oi > change_start_old || ni > change_start_new {
                changes.push(WordChange {
                    old_start: change_start_old,
                    old_end: oi,
                    new_start: change_start_new,
                    new_end: ni,
                });
            }
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_equal() {
        let engine = DiffEngine::new();
        let result = engine.diff("hello\n", "hello\n");
        assert_eq!(result.stats.insertions, 0);
        assert_eq!(result.stats.deletions, 0);
    }

    #[test]
    fn test_diff_insert() {
        let engine = DiffEngine::new();
        let result = engine.diff("hello\n", "hello\nworld\n");
        assert!(result.stats.insertions > 0);
    }

    #[test]
    fn test_diff_delete() {
        let engine = DiffEngine::new();
        let result = engine.diff("hello\nworld\n", "hello\n");
        assert!(result.stats.deletions > 0);
    }

    #[test]
    fn test_diff_replace() {
        let engine = DiffEngine::new();
        let result = engine.diff("hello\n", "world\n");
        assert!(result.stats.replacements > 0 || result.stats.deletions > 0);
    }

    #[test]
    fn test_diff_ignore_whitespace() {
        let engine = DiffEngine::new().with_options(DiffOptions {
            ignore_whitespace: true,
            ..Default::default()
        });
        let result = engine.diff("hello\n", "  hello  \n");
        assert_eq!(result.stats.insertions, 0);
        assert_eq!(result.stats.deletions, 0);
    }

    #[test]
    fn test_diff_ignore_case() {
        let engine = DiffEngine::new().with_options(DiffOptions {
            ignore_case: true,
            ..Default::default()
        });
        let result = engine.diff("Hello\n", "hello\n");
        assert_eq!(result.stats.insertions, 0);
        assert_eq!(result.stats.deletions, 0);
    }

    #[test]
    fn test_patience_diff() {
        let engine = DiffEngine::new().with_algorithm(DiffAlgorithm::Patience);
        let result = engine.diff("line1\nline2\n", "line1\nline3\n");
        assert!(result.stats.deletions > 0 || result.stats.replacements > 0);
    }

    #[test]
    fn test_myers_diff() {
        let old = vec!["a", "b", "c"];
        let new = vec!["a", "x", "c"];
        let ops = myers_diff(&old, &new);
        assert!(!ops.is_empty());
    }

    #[test]
    fn test_lcs() {
        let a = vec!["a", "b", "c", "d"];
        let b = vec!["a", "c", "d", "e"];
        let result = longest_common_subsequence(&a, &b);
        assert_eq!(result, vec!["a", "c", "d"]);
    }

    #[test]
    fn test_aligned_rows_replace_paired() {
        // A replaced line must appear on a single side-by-side row.
        let engine = DiffEngine::new();
        let result = engine.diff("hello world\n", "hello rust\n");
        let replace_rows: Vec<_> = result
            .rows
            .iter()
            .filter(|r| r.tag == DiffTag::Replace)
            .collect();
        assert_eq!(replace_rows.len(), 1);
        let r = replace_rows[0];
        assert_eq!(r.left_text.as_deref(), Some("hello world"));
        assert_eq!(r.right_text.as_deref(), Some("hello rust"));
        // The common prefix "hello " is not part of the inline span.
        assert!(!r.left_spans.is_empty());
        assert_eq!(r.left_spans[0].0, 6);
    }

    #[test]
    fn test_change_blocks_navigation() {
        let engine = DiffEngine::new();
        let result = engine.diff("a\nb\nc\nd\n", "a\nB\nc\nD\n");
        // Two separate single-line changes => two change blocks.
        assert_eq!(result.change_count(), 2);
        assert_eq!(result.change_starts.len(), 2);
    }

    #[test]
    fn test_inline_char_spans_basic() {
        // Pure insertion in the middle: left has no changed chars, right covers "XYZ".
        let (left, right) = inline_char_spans("foobar", "fooXYZbar");
        assert!(left.is_empty());
        assert_eq!(right, vec![(3, 6)]);

        // Substitution: both sides have a changed middle span.
        let (left, right) = inline_char_spans("hello world", "hello rust!");
        assert_eq!(left, vec![(6, 11)]);
        assert_eq!(right, vec![(6, 11)]);
    }

    #[test]
    fn test_aligned_rows_insert_delete_gap() {
        let engine = DiffEngine::new();
        let result = engine.diff("a\nc\n", "a\nb\nc\n");
        // The inserted "b" row should have no left line.
        let ins: Vec<_> = result
            .rows
            .iter()
            .filter(|r| r.tag == DiffTag::Insert)
            .collect();
        assert_eq!(ins.len(), 1);
        assert!(ins[0].left_line.is_none());
        assert_eq!(ins[0].right_text.as_deref(), Some("b"));
    }
}
