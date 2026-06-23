use super::{EditAction, LineEnding};

const MAX_UNDO_HISTORY: usize = 1000;

/// A simple rope-like data structure for efficient text editing.
/// All positions (char_pos, col) are CHARACTER-based, not byte-based.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SimpleRope {
    lines: Vec<String>,
}

/// Convert a character column index to a byte offset within a line.
fn char_col_to_byte(line: &str, col: usize) -> usize {
    line.char_indices()
        .nth(col)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(line.len())
}

impl SimpleRope {
    fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    fn from_str(text: &str) -> Self {
        let lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
        Self {
            lines: if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            },
        }
    }

    fn to_string_val(&self) -> String {
        self.lines.join("\n")
    }

    /// Total character count (including newlines).
    fn len_chars(&self) -> usize {
        let char_count: usize = self.lines.iter().map(|l| l.chars().count()).sum();
        char_count + self.lines.len().saturating_sub(1)
    }

    fn len_lines(&self) -> usize {
        self.lines.len()
    }

    fn line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }

    /// Returns the CHARACTER count of a line (not byte count).
    fn line_len(&self, index: usize) -> usize {
        self.lines.get(index).map_or(0, |s| s.chars().count())
    }

    fn char_at(&self, pos: usize) -> Option<char> {
        let mut remaining = pos;
        for line in &self.lines {
            let char_count = line.chars().count();
            if remaining < char_count {
                return line.chars().nth(remaining);
            }
            remaining -= char_count;
            if remaining == 0 {
                return Some('\n');
            }
            remaining -= 1;
        }
        None
    }

    fn insert(&mut self, char_pos: usize, text: &str) {
        let (line, col) = self.pos_to_line_col(char_pos);
        let line_content = self.lines[line].clone();
        let byte_col = char_col_to_byte(&line_content, col);
        let (before, after) = line_content.split_at(byte_col.min(line_content.len()));

        let new_lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();

        if new_lines.len() == 1 {
            self.lines[line] = format!("{}{}{}", before, text, after);
        } else {
            let first_line = format!("{}{}", before, new_lines[0]);
            let last_line = format!("{}{}", new_lines.last().unwrap(), after);
            let middle_lines = &new_lines[1..new_lines.len() - 1];

            self.lines[line] = first_line;
            for (i, new_line) in middle_lines.iter().enumerate() {
                self.lines.insert(line + 1 + i, new_line.clone());
            }
            self.lines.insert(line + new_lines.len() - 1, last_line);
        }
    }

    fn remove(&mut self, start: usize, end: usize) {
        let (start_line, start_col) = self.pos_to_line_col(start);
        let (end_line, end_col) = self.pos_to_line_col(end);

        if start_line == end_line {
            let line = &self.lines[start_line];
            let start_byte = char_col_to_byte(line, start_col).min(line.len());
            let end_byte = char_col_to_byte(line, end_col).min(line.len());
            let new_line = format!("{}{}", &line[..start_byte], &line[end_byte..]);
            self.lines[start_line] = new_line;
        } else {
            let start_line_content = self.lines[start_line].clone();
            let end_line_content = self.lines[end_line].clone();

            let start_byte = char_col_to_byte(&start_line_content, start_col).min(start_line_content.len());
            let end_byte = char_col_to_byte(&end_line_content, end_col).min(end_line_content.len());

            let new_start = format!(
                "{}{}",
                &start_line_content[..start_byte],
                &end_line_content[end_byte..]
            );

            self.lines[start_line] = new_start;
            self.lines.drain(start_line + 1..=end_line);
        }
    }

    fn slice(&self, start: usize, end: usize) -> String {
        let (start_line, start_col) = self.pos_to_line_col(start);
        let (end_line, end_col) = self.pos_to_line_col(end);

        if start_line == end_line {
            let line = &self.lines[start_line];
            let start_byte = char_col_to_byte(line, start_col).min(line.len());
            let end_byte = char_col_to_byte(line, end_col).min(line.len());
            return line[start_byte..end_byte].to_string();
        }

        let mut result = String::new();
        for i in start_line..=end_line.min(self.lines.len() - 1) {
            let line = &self.lines[i];
            if i == start_line {
                let start_byte = char_col_to_byte(line, start_col).min(line.len());
                result.push_str(&line[start_byte..]);
                result.push('\n');
            } else if i == end_line {
                let end_byte = char_col_to_byte(line, end_col).min(line.len());
                result.push_str(&line[..end_byte]);
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    }

    /// Convert a global CHARACTER position to (line, char_col).
    fn pos_to_line_col(&self, pos: usize) -> (usize, usize) {
        let mut remaining = pos;
        for (i, line) in self.lines.iter().enumerate() {
            let char_count = line.chars().count();
            if remaining <= char_count {
                return (i, remaining);
            }
            remaining -= char_count;
            if remaining == 0 {
                return (i, char_count);
            }
            remaining -= 1; // for '\n'
        }
        let last_line = self.lines.len().saturating_sub(1);
        (last_line, self.lines[last_line].chars().count())
    }

    /// Convert (line, char_col) to a global CHARACTER position.
    fn line_col_to_pos(&self, line: usize, col: usize) -> usize {
        let mut pos = 0;
        for i in 0..line.min(self.lines.len()) {
            pos += self.lines[i].chars().count() + 1; // +1 for '\n'
        }
        let max_col = self.lines.get(line).map_or(0, |l| l.chars().count());
        pos + col.min(max_col)
    }

    fn char_to_line(&self, char_pos: usize) -> usize {
        self.pos_to_line_col(char_pos).0
    }

    fn line_to_char(&self, line: usize) -> usize {
        self.line_col_to_pos(line, 0)
    }
}

/// Undo/redo history using Command pattern.
#[derive(Debug)]
#[allow(dead_code)]
pub struct UndoHistory {
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    max_history: usize,
}

impl UndoHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: MAX_UNDO_HISTORY,
        }
    }

    pub fn push_undo(&mut self, action: EditAction) {
        self.undo_stack.push(action);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn pop_undo(&mut self) -> Option<EditAction> {
        self.undo_stack.pop()
    }

    pub fn push_redo(&mut self, action: EditAction) {
        self.redo_stack.push(action);
    }

    pub fn pop_redo(&mut self) -> Option<EditAction> {
        self.redo_stack.pop()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Text buffer with undo/redo support.
#[allow(dead_code)]
pub struct TextBuffer {
    rope: SimpleRope,
    history: UndoHistory,
    line_ending: LineEnding,
    dirty: bool,
}

#[allow(dead_code)]
impl TextBuffer {
    pub fn new() -> Self {
        Self {
            rope: SimpleRope::new(),
            history: UndoHistory::new(),
            line_ending: LineEnding::default(),
            dirty: false,
        }
    }

    pub fn from_str(text: &str) -> Self {
        let line_ending = crate::editor::detect_line_ending(text);
        Self {
            rope: SimpleRope::from_str(text),
            history: UndoHistory::new(),
            line_ending,
            dirty: false,
        }
    }

    pub fn from_text(text: String, line_ending: LineEnding) -> Self {
        Self {
            rope: SimpleRope::from_str(&text),
            history: UndoHistory::new(),
            line_ending,
            dirty: false,
        }
    }

    pub fn text(&self) -> String {
        self.rope.to_string_val()
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line(&self, index: usize) -> Option<&str> {
        self.rope.line(index)
    }

    pub fn line_len(&self, index: usize) -> usize {
        self.rope.line_len(index)
    }

    pub fn len(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn char_pos_for_line_col(&self, line: usize, col: usize) -> usize {
        self.rope.line_col_to_pos(line, col)
    }

    pub fn line_col_for_char_pos(&self, pos: usize) -> (usize, usize) {
        self.rope.pos_to_line_col(pos)
    }

    pub fn insert_str(&mut self, pos: usize, text: &str) {
        let old_text = text.to_string();
        self.rope.insert(pos, text);
        self.dirty = true;
        self.history.push_undo(EditAction::Insert {
            char_pos: pos,
            text: old_text,
        });
    }

    pub fn delete_range(&mut self, start: usize, end: usize) {
        let deleted = self.rope.slice(start, end);
        self.rope.remove(start, end);
        self.dirty = true;
        self.history.push_undo(EditAction::Delete {
            char_pos: start,
            text: deleted,
        });
    }

    pub fn replace_range(&mut self, start: usize, end: usize, new_text: &str) {
        let old_text = self.rope.slice(start, end);
        self.rope.remove(start, end);
        self.rope.insert(start, new_text);
        self.dirty = true;
        self.history.push_undo(EditAction::Replace {
            char_pos: start,
            old_text,
            new_text: new_text.to_string(),
        });
    }

    pub fn undo(&mut self) -> bool {
        if let Some(action) = self.history.pop_undo() {
            match &action {
                EditAction::Insert { char_pos, text } => {
                    let end = char_pos + text.chars().count();
                    self.rope.remove(*char_pos, end);
                    self.history.push_redo(action);
                }
                EditAction::Delete { char_pos, text } => {
                    self.rope.insert(*char_pos, text);
                    self.history.push_redo(action);
                }
                EditAction::Replace {
                    char_pos,
                    old_text,
                    new_text,
                } => {
                    let end = char_pos + new_text.chars().count();
                    self.rope.remove(*char_pos, end);
                    self.rope.insert(*char_pos, old_text);
                    self.history.push_redo(action);
                }
            }
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.history.pop_redo() {
            match &action {
                EditAction::Insert { char_pos, text } => {
                    self.rope.insert(*char_pos, text);
                    self.history.push_undo(action);
                }
                EditAction::Delete { char_pos, text } => {
                    let end = char_pos + text.chars().count();
                    self.rope.remove(*char_pos, end);
                    self.history.push_undo(action);
                }
                EditAction::Replace {
                    char_pos,
                    old_text,
                    new_text,
                } => {
                    let end = char_pos + old_text.chars().count();
                    self.rope.remove(*char_pos, end);
                    self.rope.insert(*char_pos, new_text);
                    self.history.push_undo(action);
                }
            }
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    #[allow(unknown_lints)]
    pub fn rope(&self) -> &SimpleRope {
        &self.rope
    }

    /// Extract a substring by character range `[start, end)`.
    pub fn slice(&self, start: usize, end: usize) -> String {
        self.rope.slice(start, end)
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rope_basic() {
        let rope = SimpleRope::from_str("hello\nworld");
        assert_eq!(rope.len_lines(), 2);
        assert_eq!(rope.line(0), Some("hello"));
        assert_eq!(rope.line(1), Some("world"));
    }

    #[test]
    fn test_simple_rope_insert() {
        let mut rope = SimpleRope::from_str("hello");
        rope.insert(5, " world");
        assert_eq!(rope.to_string_val(), "hello world");
    }

    #[test]
    fn test_simple_rope_remove() {
        let mut rope = SimpleRope::from_str("hello world");
        rope.remove(5, 11);
        assert_eq!(rope.to_string_val(), "hello");
    }

    #[test]
    fn test_simple_rope_pos_to_line_col() {
        let rope = SimpleRope::from_str("hello\nworld");
        assert_eq!(rope.pos_to_line_col(0), (0, 0));
        assert_eq!(rope.pos_to_line_col(5), (0, 5));
        assert_eq!(rope.pos_to_line_col(6), (1, 0));
        assert_eq!(rope.pos_to_line_col(11), (1, 5));
    }

    #[test]
    fn test_text_buffer_basic() {
        let mut buf = TextBuffer::from_str("hello");
        assert_eq!(buf.text(), "hello");
        assert_eq!(buf.line_count(), 1);

        buf.insert_str(5, " world");
        assert_eq!(buf.text(), "hello world");
        assert!(buf.is_dirty());
    }

    #[test]
    fn test_text_buffer_undo_redo() {
        let mut buf = TextBuffer::from_str("hello");
        buf.insert_str(5, " world");
        assert_eq!(buf.text(), "hello world");
        assert!(buf.can_undo());

        buf.undo();
        assert_eq!(buf.text(), "hello");
        assert!(buf.can_redo());

        buf.redo();
        assert_eq!(buf.text(), "hello world");
    }

    #[test]
    fn test_text_buffer_delete() {
        let mut buf = TextBuffer::from_str("hello world");
        buf.delete_range(5, 11);
        assert_eq!(buf.text(), "hello");
    }

    #[test]
    fn test_text_buffer_unicode() {
        let mut buf = TextBuffer::from_str("你好世界");
        assert_eq!(buf.len(), 4);
        buf.insert_str(4, "！");
        assert_eq!(buf.text(), "你好世界！");
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn test_text_buffer_multiline_unicode() {
        let mut buf = TextBuffer::from_str("你好\n世界");
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line_len(0), 2);
        buf.insert_str(2, "！");
        assert_eq!(buf.text(), "你好！\n世界");
    }
}
