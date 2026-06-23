use super::buffer::TextBuffer;
use super::cursor::Selection;

/// Column bounds `(start_col, end_col)` from a selection anchor and cursor.
pub fn column_col_range(selection: &Selection) -> (usize, usize) {
    let norm = selection.normalized();
    (
        norm.start.col.min(norm.end.col),
        norm.start.col.max(norm.end.col),
    )
}

/// Extract rectangular column text (one segment per line, joined with `\n`).
pub fn extract_text(buffer: &TextBuffer, selection: &Selection) -> String {
    let norm = selection.normalized();
    if norm.is_empty() {
        return String::new();
    }
    let (col_start, col_end) = column_col_range(&norm);
    let mut lines = Vec::new();
    for line_idx in norm.start.line..=norm.end.line {
        let line = buffer.line(line_idx).unwrap_or("");
        let chars: Vec<char> = line.chars().collect();
        let end = col_end.min(chars.len());
        let slice: String = if col_start < end {
            chars[col_start..end].iter().collect()
        } else {
            String::new()
        };
        lines.push(slice);
    }
    lines.join("\n")
}

/// Delete the rectangular column covered by `selection`.
pub fn delete_column(buffer: &mut TextBuffer, selection: &Selection) -> bool {
    let norm = selection.normalized();
    if norm.is_empty() {
        return false;
    }
    let (col_start, col_end) = column_col_range(&norm);
    if col_start >= col_end && norm.start.line == norm.end.line {
        return false;
    }

    for line_idx in (norm.start.line..=norm.end.line).rev() {
        let line_len = buffer.line_len(line_idx);
        let start = col_start.min(line_len);
        let end = col_end.min(line_len);
        if start < end {
            let char_start = buffer.char_pos_for_line_col(line_idx, start);
            let char_end = buffer.char_pos_for_line_col(line_idx, end);
            buffer.delete_range(char_start, char_end);
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{Cursor, LineEnding};

    #[test]
    fn test_extract_column() {
        let text = "abcdef\n123456\nABCDEF";
        let buffer = TextBuffer::from_text(text.to_string(), LineEnding::Lf);
        let sel = Selection::new(Cursor::new(0, 2), Cursor::new(2, 4));
        assert_eq!(extract_text(&buffer, &sel), "cd\n34\nCD");
    }

    #[test]
    fn test_delete_column() {
        let text = "abcdef\n123456";
        let mut buffer = TextBuffer::from_text(text.to_string(), LineEnding::Lf);
        let sel = Selection::new(Cursor::new(0, 2), Cursor::new(1, 4));
        assert!(delete_column(&mut buffer, &sel));
        assert_eq!(buffer.text(), "abef\n1256");
    }
}
