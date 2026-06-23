//! Editor context-menu actions (comments, bookmarks, conversion, etc.).

use std::collections::HashMap;

use crate::editor::fold::FoldState;
use crate::editor::{Cursor, Selection, TextBuffer};

/// Bookmark on a line (0-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bookmark {
    pub line: usize,
}

/// Colored background highlight on a text range (foreground unchanged).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextMark {
    pub selection: Selection,
    pub color: u8,
}

/// Per-tab editor extras used by the context menu.
#[derive(Debug, Clone, Default)]
pub struct TabEditorExtras {
    pub bookmarks: Vec<Bookmark>,
    /// Line number → color index (0..5), shown as a gutter stripe.
    pub line_marks: HashMap<usize, u8>,
    /// Colored background ranges in the editor text area.
    pub text_marks: Vec<TextMark>,
    pub fold_state: FoldState,
}

/// Comment delimiters for the active syntax.
#[derive(Debug, Clone, Copy)]
pub struct CommentStyle {
    pub line_prefix: &'static str,
    pub block_start: &'static str,
    pub block_end: &'static str,
}

pub fn comment_style_for_syntax(syntax: &str) -> CommentStyle {
    let s = syntax.to_lowercase();
    if s.contains("rust") || s.contains("c++") || s.contains("cpp") || s.contains("java")
        || s.contains("javascript") || s.contains("typescript") || s.contains("go")
        || s.contains("swift") || s.contains("kotlin")
    {
        CommentStyle {
            line_prefix: "//",
            block_start: "/*",
            block_end: "*/",
        }
    } else if s.contains("python") || s.contains("shell") || s.contains("bash")
        || s.contains("ruby") || s.contains("perl") || s.contains("yaml")
    {
        CommentStyle {
            line_prefix: "#",
            block_start: "\"\"\"",
            block_end: "\"\"\"",
        }
    } else if s.contains("html") || s.contains("xml") {
        CommentStyle {
            line_prefix: "",
            block_start: "<!--",
            block_end: "-->",
        }
    } else if s.contains("lua") || s.contains("sql") {
        CommentStyle {
            line_prefix: "--",
            block_start: "/*",
            block_end: "*/",
        }
    } else {
        CommentStyle {
            line_prefix: "//",
            block_start: "/*",
            block_end: "*/",
        }
    }
}

/// Toggle line comments on all lines touched by the selection.
pub fn toggle_line_comments(
    buffer: &mut TextBuffer,
    selection: &Selection,
    style: CommentStyle,
) -> bool {
    if style.line_prefix.is_empty() {
        return false;
    }
    let norm = selection.normalized();
    let start_line = norm.start.line;
    let end_line = norm.end.line.max(start_line);
    let prefix = style.line_prefix;

    let mut all_commented = true;
    for line_idx in start_line..=end_line {
        if let Some(line) = buffer.line(line_idx) {
            if !line.trim_start().starts_with(prefix) {
                all_commented = false;
                break;
            }
        }
    }

    for line_idx in (start_line..=end_line).rev() {
        let Some(line) = buffer.line(line_idx).map(|s| s.to_string()) else {
            continue;
        };
        let line_start = buffer.char_pos_for_line_col(line_idx, 0);
        if all_commented {
            let indent_len = line.len() - line.trim_start().len();
            let trim = line.trim_start();
            if let Some(rest) = trim.strip_prefix(prefix) {
                let mut remove_len = indent_len + prefix.len();
                if rest.starts_with(' ') {
                    remove_len += 1;
                }
                let start = line_start + indent_len;
                let end = line_start + remove_len;
                buffer.delete_range(start, end);
            }
        } else {
            let indent_len = line.len() - line.trim_start().len();
            let insert_at = line_start + indent_len;
            buffer.insert_str(insert_at, &format!("{prefix} "));
        }
    }
    true
}

/// Wrap selection in a block comment.
pub fn add_block_comment(buffer: &mut TextBuffer, selection: &Selection, style: CommentStyle) {
    if style.block_start.is_empty() {
        return;
    }
    let (start, end) = selection.to_byte_range(buffer);
    let snippet = buffer.slice(start, end);
    let wrapped = format!("{} {} {}", style.block_start, snippet, style.block_end);
    buffer.replace_range(start, end, &wrapped);
}

/// Remove block comment wrapping the selection if present.
pub fn remove_block_comment(buffer: &mut TextBuffer, selection: &Selection, style: CommentStyle) {
    if style.block_start.is_empty() {
        return;
    }
    let (start, end) = selection.to_byte_range(buffer);
    let snippet = buffer.slice(start, end);
    let trimmed = snippet.trim();
    if trimmed.starts_with(style.block_start) && trimmed.ends_with(style.block_end) {
        let inner = trimmed
            .strip_prefix(style.block_start)
            .unwrap_or(trimmed)
            .trim_start()
            .strip_suffix(style.block_end)
            .unwrap_or(trimmed)
            .trim_end();
        buffer.replace_range(start, end, inner);
    }
}

pub fn word_count(text: &str) -> (usize, usize, usize) {
    let chars = text.chars().count();
    let words = text.split_whitespace().filter(|w| !w.is_empty()).count();
    let lines = if text.is_empty() {
        1
    } else {
        text.lines().count()
    };
    (chars, words, lines)
}

/// Very small Markdown → HTML converter for preview / export.
pub fn markdown_to_html(md: &str) -> String {
    let mut out = String::from(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Preview</title>\
         <style>body{font-family:sans-serif;max-width:720px;margin:2em auto;line-height:1.6}</style>\
         </head><body>\n",
    );
    let mut in_code = false;
    for line in md.lines() {
        if line.starts_with("```") {
            if in_code {
                out.push_str("</pre>\n");
                in_code = false;
            } else {
                out.push_str("<pre><code>");
                in_code = true;
            }
            continue;
        }
        if in_code {
            out.push_str(&html_escape(line));
            out.push('\n');
            continue;
        }
        if let Some(rest) = line.strip_prefix("# ") {
            out.push_str(&format!("<h1>{}</h1>\n", html_escape(rest)));
        } else if let Some(rest) = line.strip_prefix("## ") {
            out.push_str(&format!("<h2>{}</h2>\n", html_escape(rest)));
        } else if let Some(rest) = line.strip_prefix("### ") {
            out.push_str(&format!("<h3>{}</h3>\n", html_escape(rest)));
        } else if line.trim().is_empty() {
            out.push_str("<br/>\n");
        } else {
            let escaped = html_escape(line);
            let with_bold = escaped.replace("**", "<b>").replace("__", "<b>");
            out.push_str(&format!("<p>{with_bold}</p>\n"));
        }
    }
    if in_code {
        out.push_str("</pre>\n");
    }
    out.push_str("</body></html>");
    out
}

pub fn text_to_html(text: &str) -> String {
    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body><pre>{}</pre></body></html>",
        html_escape(text)
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Visible replacement for whitespace when "display blank chars" is on.
pub fn visualize_whitespace(line: &str, show_blanks: bool, show_tabs_as_spaces: bool) -> String {
    if !show_blanks && !show_tabs_as_spaces {
        return line.to_string();
    }
    let mut out = String::new();
    for ch in line.chars() {
        match ch {
            ' ' if show_blanks => out.push('·'),
            '\t' if show_blanks || show_tabs_as_spaces => {
                if show_tabs_as_spaces {
                    let tab_size = 4;
                    for _ in 0..tab_size {
                        out.push(' ');
                    }
                } else {
                    out.push('→');
                }
            }
            c => out.push(c),
        }
    }
    out
}

/// Show non-printable characters as Unicode control pictures.
pub fn visualize_non_prints(line: &str) -> String {
    line.chars()
        .map(|c| {
            if c.is_control() && c != '\t' {
                format!("U+{:04X}", c as u32)
            } else {
                c.to_string()
            }
        })
        .collect()
}

pub fn mark_line_color(index: u8) -> (u8, u8, u8) {
    match index % 5 {
        0 => (255, 200, 200),
        1 => (200, 220, 255),
        2 => (200, 255, 200),
        3 => (255, 255, 180),
        4 => (230, 200, 255),
        _ => (220, 220, 220),
    }
}

pub fn toggle_bookmark(extras: &mut TabEditorExtras, line: usize) {
    if let Some(pos) = extras.bookmarks.iter().position(|b| b.line == line) {
        extras.bookmarks.remove(pos);
    } else {
        extras.bookmarks.push(Bookmark { line });
    }
}

pub fn clear_all_bookmarks(extras: &mut TabEditorExtras) {
    extras.bookmarks.clear();
    extras.line_marks.clear();
    extras.text_marks.clear();
}

/// Text mark stored relative to clipboard / pasted text (character offsets).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeTextMark {
    pub start: usize,
    pub end: usize,
    pub color: u8,
}

fn selection_to_chars(selection: Selection, buffer: &TextBuffer) -> (usize, usize) {
    let norm = selection.normalized();
    let start = buffer.char_pos_for_line_col(norm.start.line, norm.start.col);
    let end = buffer.char_pos_for_line_col(norm.end.line, norm.end.col);
    (start, end)
}

fn chars_to_selection(start: usize, end: usize, buffer: &TextBuffer) -> Selection {
    let (sl, sc) = buffer.line_col_for_char_pos(start);
    let (el, ec) = buffer.line_col_for_char_pos(end);
    Selection::new(Cursor::new(sl, sc), Cursor::new(el, ec)).normalized()
}

/// Sync gutter stripes from persisted text marks (drop stale line-only entries).
pub fn rebuild_line_marks(extras: &mut TabEditorExtras) {
    extras.line_marks.clear();
    for mark in &extras.text_marks {
        let norm = mark.selection.normalized();
        for line in norm.start.line..=norm.end.line {
            extras.line_marks.insert(line, mark.color);
        }
    }
}

/// Collect marks overlapping `[range_start, range_end)` as offsets relative to `range_start`.
pub fn marks_within_char_range(
    extras: &TabEditorExtras,
    buffer: &TextBuffer,
    range_start: usize,
    range_end: usize,
) -> Vec<RelativeTextMark> {
    let mut out = Vec::new();
    for mark in &extras.text_marks {
        let (ms, me) = selection_to_chars(mark.selection, buffer);
        if ms >= range_end || me <= range_start {
            continue;
        }
        let rel_start = ms.max(range_start) - range_start;
        let rel_end = me.min(range_end) - range_start;
        if rel_start < rel_end {
            out.push(RelativeTextMark {
                start: rel_start,
                end: rel_end,
                color: mark.color,
            });
        }
    }
    out
}

fn map_mark_through_delete(
    ms: usize,
    me: usize,
    del_start: usize,
    del_end: usize,
) -> Option<(usize, usize)> {
    let del_len = del_end.saturating_sub(del_start);
    if me <= del_start {
        return Some((ms, me));
    }
    if ms >= del_end {
        return Some((ms - del_len, me - del_len));
    }
    if ms < del_start && me > del_end {
        return Some((ms, me - del_len));
    }
    if ms >= del_start && me <= del_end {
        return None;
    }
    if ms < del_start {
        return Some((ms, del_start));
    }
    // Mark starts inside deleted region but extends past it.
    let ns = del_start;
    let ne = me - del_len;
    if ns < ne {
        Some((ns, ne))
    } else {
        None
    }
}

/// Recompute marks after deleting `[del_start, del_end)` (call before `delete_range`).
pub fn remap_marks_for_delete(
    extras: &TabEditorExtras,
    buffer: &TextBuffer,
    del_start: usize,
    del_end: usize,
) -> Vec<(usize, usize, u8)> {
    let mut mapped = Vec::new();
    for mark in &extras.text_marks {
        let (ms, me) = selection_to_chars(mark.selection, buffer);
        if let Some((ns, ne)) = map_mark_through_delete(ms, me, del_start, del_end) {
            if ns < ne {
                mapped.push((ns, ne, mark.color));
            }
        }
    }
    mapped
}

/// Recompute marks after inserting `insert_len` chars at `insert_pos` (call before `insert_str`).
pub fn remap_marks_for_insert(
    extras: &TabEditorExtras,
    buffer: &TextBuffer,
    insert_pos: usize,
    insert_len: usize,
    added: &[RelativeTextMark],
) -> Vec<(usize, usize, u8)> {
    let mut mapped = Vec::new();
    for mark in &extras.text_marks {
        let (ms, me) = selection_to_chars(mark.selection, buffer);
        let (ns, ne) = if me <= insert_pos {
            (ms, me)
        } else if ms >= insert_pos {
            (ms + insert_len, me + insert_len)
        } else {
            (ms, me + insert_len)
        };
        if ns < ne {
            mapped.push((ns, ne, mark.color));
        }
    }
    for m in added {
        let ns = insert_pos + m.start;
        let ne = insert_pos + m.end;
        if ns < ne {
            mapped.push((ns, ne, m.color));
        }
    }
    mapped
}

pub fn apply_char_marks(
    extras: &mut TabEditorExtras,
    buffer: &TextBuffer,
    ranges: &[(usize, usize, u8)],
) {
    extras.text_marks = ranges
        .iter()
        .filter(|(s, e, _)| s < e)
        .map(|(s, e, color)| TextMark {
            selection: chars_to_selection(*s, *e, buffer),
            color: *color,
        })
        .collect();
    rebuild_line_marks(extras);
}

pub fn delete_at_cursor(buffer: &mut TextBuffer, cursor: &Cursor, selection: &Selection) -> bool {
    if !selection.is_empty() {
        let (start, end) = selection.to_byte_range(buffer);
        buffer.delete_range(start, end);
        return true;
    }
    let pos = buffer.char_pos_for_line_col(cursor.line, cursor.col);
    let len = buffer.len();
    if pos < len {
        let next = pos + 1;
        buffer.delete_range(pos, next.min(len));
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::TextBuffer;

    #[test]
    fn test_word_count() {
        let (c, w, l) = word_count("hello world\nfoo");
        assert_eq!(w, 3);
        assert!(c >= 11);
        assert_eq!(l, 2);
    }

    #[test]
    fn test_toggle_line_comment() {
        let mut buf = TextBuffer::from_text("fn main() {\n}\n".to_string(), crate::editor::LineEnding::Lf);
        let sel = Selection::new(Cursor::new(0, 0), Cursor::new(1, 0));
        let style = comment_style_for_syntax("Rust");
        toggle_line_comments(&mut buf, &sel, style);
        assert!(buf.text().contains("// fn main()"));
    }

    #[test]
    fn test_remap_marks_for_delete() {
        let mut buffer = TextBuffer::new();
        buffer.insert_str(0, "hello world");
        let mut extras = TabEditorExtras::default();
        extras.text_marks.push(TextMark {
            selection: Selection::new(Cursor::new(0, 0), Cursor::new(0, 5)),
            color: 1,
        });
        let mapped = remap_marks_for_delete(&extras, &buffer, 0, 5);
        assert!(mapped.is_empty());
        buffer.delete_range(0, 5);
        apply_char_marks(&mut extras, &buffer, &mapped);
        assert!(extras.text_marks.is_empty());
        assert!(extras.line_marks.is_empty());
    }

    #[test]
    fn test_rebuild_line_marks_multiline() {
        let mut extras = TabEditorExtras::default();
        extras.text_marks.push(TextMark {
            selection: Selection::new(Cursor::new(1, 0), Cursor::new(3, 4)),
            color: 2,
        });
        rebuild_line_marks(&mut extras);
        assert_eq!(extras.line_marks.get(&1), Some(&2));
        assert_eq!(extras.line_marks.get(&2), Some(&2));
        assert_eq!(extras.line_marks.get(&3), Some(&2));
        assert!(extras.line_marks.get(&0).is_none());
    }

    #[test]
    fn test_clipboard_mark_roundtrip() {
        let mut buffer = TextBuffer::new();
        buffer.insert_str(0, "abcdef");
        let mut extras = TabEditorExtras::default();
        extras.text_marks.push(TextMark {
            selection: Selection::new(Cursor::new(0, 1), Cursor::new(0, 4)),
            color: 2,
        });
        let rel = marks_within_char_range(&extras, &buffer, 1, 4);
        assert_eq!(rel.len(), 1);
        let mapped = remap_marks_for_insert(&extras, &buffer, 6, 3, &rel);
        buffer.insert_str(6, "xyz");
        apply_char_marks(&mut extras, &buffer, &mapped);
        assert_eq!(extras.text_marks.len(), 2);
        assert!(extras.text_marks.iter().any(|m| {
            let n = m.selection.normalized();
            n.start.col == 1 && n.end.col == 4
        }));
        assert!(extras.text_marks.iter().any(|m| {
            let n = m.selection.normalized();
            n.start.col == 6 && n.end.col == 9
        }));
    }
}
