/// Represents a single cursor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// Line index (0-based).
    pub line: usize,
    /// Column index in characters (0-based).
    pub col: usize,
    /// Preferred column for vertical movement (sticky column).
    pub preferred_col: Option<usize>,
}

impl Cursor {
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            line,
            col,
            preferred_col: None,
        }
    }

    pub fn set(&mut self, line: usize, col: usize) {
        self.line = line;
        self.col = col;
        self.preferred_col = None;
    }

    pub fn move_up(&mut self, n: usize, max_col: usize) {
        if self.preferred_col.is_none() {
            self.preferred_col = Some(self.col);
        }
        self.line = self.line.saturating_sub(n);
        self.col = self.preferred_col.unwrap_or(0).min(max_col);
    }

    pub fn move_down(&mut self, n: usize, max_col: usize) {
        if self.preferred_col.is_none() {
            self.preferred_col = Some(self.col);
        }
        self.line += n;
        self.col = self.preferred_col.unwrap_or(0).min(max_col);
    }

    pub fn move_left(&mut self, n: usize) {
        self.col = self.col.saturating_sub(n);
        self.preferred_col = None;
    }

    pub fn move_right(&mut self, n: usize) {
        self.col += n;
        self.preferred_col = None;
    }

    pub fn move_to_line_start(&mut self) {
        self.col = 0;
        self.preferred_col = None;
    }

    pub fn move_to_line_end(&mut self, max_col: usize) {
        self.col = max_col;
        self.preferred_col = None;
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Represents a selection range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub start: Cursor,
    pub end: Cursor,
}

impl Selection {
    pub fn new(start: Cursor, end: Cursor) -> Self {
        Self { start, end }
    }

    pub fn cursor(cursor: Cursor) -> Self {
        Self {
            start: cursor,
            end: cursor,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn normalized(&self) -> Self {
        if self.start.line > self.end.line
            || (self.start.line == self.end.line && self.start.col > self.end.col)
        {
            Self {
                start: self.end,
                end: self.start,
            }
        } else {
            *self
        }
    }

    pub fn contains(&self, line: usize, col: usize) -> bool {
        let norm = self.normalized();
        if line < norm.start.line || line > norm.end.line {
            return false;
        }
        if line == norm.start.line && col < norm.start.col {
            return false;
        }
        if line == norm.end.line && col >= norm.end.col {
            return false;
        }
        true
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_byte_range(&self, buffer: &super::TextBuffer) -> (usize, usize) {
        let norm = self.normalized();
        let start = buffer.char_pos_for_line_col(norm.start.line, norm.start.col);
        let end = buffer.char_pos_for_line_col(norm.end.line, norm.end.col);
        (start, end)
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::cursor(Cursor::default())
    }
}

/// Multi-cursor state with support for rectangular selections.
#[derive(Debug, Clone)]
pub struct CursorState {
    pub primary: Cursor,
    pub anchor: Option<Cursor>,
    pub extra_cursors: Vec<Cursor>,
    pub column_selection: bool,
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            primary: Cursor::default(),
            anchor: None,
            extra_cursors: Vec::new(),
            column_selection: false,
        }
    }

    pub fn selection(&self) -> Selection {
        match self.anchor {
            Some(anchor) => Selection::new(anchor, self.primary),
            None => Selection::cursor(self.primary),
        }
    }

    pub fn start_selection(&mut self) {
        self.anchor = Some(self.primary);
    }

    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    pub fn has_selection(&self) -> bool {
        self.anchor.is_some() && self.anchor != Some(self.primary)
    }

    pub fn move_left(&mut self, line_len_fn: &dyn Fn(usize) -> usize, select: bool) {
        if !select {
            self.anchor = None;
        } else if self.anchor.is_none() {
            self.anchor = Some(self.primary);
        }

        if self.primary.col > 0 {
            self.primary.col -= 1;
        } else if self.primary.line > 0 {
            self.primary.line -= 1;
            self.primary.col = line_len_fn(self.primary.line);
        }
        self.primary.preferred_col = None;
    }

    pub fn move_right(
        &mut self,
        line_len_fn: &dyn Fn(usize) -> usize,
        max_line: usize,
        select: bool,
    ) {
        if !select {
            self.anchor = None;
        } else if self.anchor.is_none() {
            self.anchor = Some(self.primary);
        }

        let line_len = line_len_fn(self.primary.line);
        if self.primary.col < line_len {
            self.primary.col += 1;
        } else if self.primary.line + 1 < max_line {
            self.primary.line += 1;
            self.primary.col = 0;
        }
        self.primary.preferred_col = None;
    }

    pub fn move_up(&mut self, line_len_fn: &dyn Fn(usize) -> usize, select: bool) {
        if !select {
            self.anchor = None;
        } else if self.anchor.is_none() {
            self.anchor = Some(self.primary);
        }

        let pref = self.primary.preferred_col.unwrap_or(self.primary.col);
        if self.primary.line > 0 {
            self.primary.line -= 1;
            let max_col = line_len_fn(self.primary.line);
            self.primary.col = pref.min(max_col);
        }
        self.primary.preferred_col = Some(pref);
    }

    pub fn move_down(
        &mut self,
        line_len_fn: &dyn Fn(usize) -> usize,
        max_line: usize,
        select: bool,
    ) {
        if !select {
            self.anchor = None;
        } else if self.anchor.is_none() {
            self.anchor = Some(self.primary);
        }

        let pref = self.primary.preferred_col.unwrap_or(self.primary.col);
        if self.primary.line + 1 < max_line {
            self.primary.line += 1;
            let max_col = line_len_fn(self.primary.line);
            self.primary.col = pref.min(max_col);
        }
        self.primary.preferred_col = Some(pref);
    }

    pub fn move_word_left(&mut self, text: &str, select: bool) {
        if !select {
            self.anchor = None;
        } else if self.anchor.is_none() {
            self.anchor = Some(self.primary);
        }

        let pos = self.char_pos(text);
        if pos == 0 {
            return;
        }

        let chars: Vec<char> = text.chars().collect();
        let mut new_pos = pos;

        while new_pos > 0 && is_word_char(chars[new_pos - 1]) {
            new_pos -= 1;
        }
        while new_pos > 0 && chars[new_pos - 1].is_whitespace() {
            new_pos -= 1;
        }
        while new_pos > 0 && is_word_char(chars[new_pos - 1]) {
            new_pos -= 1;
        }

        let (line, col) = pos_to_line_col(text, new_pos);
        self.primary.set(line, col);
    }

    pub fn move_word_right(&mut self, text: &str, select: bool) {
        if !select {
            self.anchor = None;
        } else if self.anchor.is_none() {
            self.anchor = Some(self.primary);
        }

        let pos = self.char_pos(text);
        let chars: Vec<char> = text.chars().collect();

        if pos >= chars.len() {
            return;
        }

        let mut new_pos = pos;
        while new_pos < chars.len() && is_word_char(chars[new_pos]) {
            new_pos += 1;
        }
        while new_pos < chars.len() && chars[new_pos].is_whitespace() {
            new_pos += 1;
        }

        let (line, col) = pos_to_line_col(text, new_pos);
        self.primary.set(line, col);
    }

    pub fn move_to_line_start(&mut self, select: bool) {
        if !select {
            self.anchor = None;
        } else if self.anchor.is_none() {
            self.anchor = Some(self.primary);
        }
        self.primary.col = 0;
        self.primary.preferred_col = None;
    }

    pub fn move_to_line_end(&mut self, line_len: usize, select: bool) {
        if !select {
            self.anchor = None;
        } else if self.anchor.is_none() {
            self.anchor = Some(self.primary);
        }
        self.primary.col = line_len;
        self.primary.preferred_col = None;
    }

    pub fn select_all(&mut self, total_lines: usize, last_line_len: usize) {
        self.anchor = Some(Cursor::new(0, 0));
        self.primary = Cursor::new(total_lines.saturating_sub(1), last_line_len);
    }

    pub fn select_word(&mut self, text: &str) {
        let pos = self.char_pos(text);
        let chars: Vec<char> = text.chars().collect();

        if pos >= chars.len() {
            return;
        }

        let mut start = pos;
        let mut end = pos;

        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }

        let (start_line, start_col) = pos_to_line_col(text, start);
        let (end_line, end_col) = pos_to_line_col(text, end);

        self.anchor = Some(Cursor::new(start_line, start_col));
        self.primary = Cursor::new(end_line, end_col);
    }

    pub fn select_line(&mut self, total_lines: usize, line_len: usize) {
        self.anchor = Some(Cursor::new(self.primary.line, 0));
        if self.primary.line + 1 < total_lines {
            self.primary = Cursor::new(self.primary.line + 1, 0);
        } else {
            self.primary = Cursor::new(self.primary.line, line_len);
        }
    }

    fn char_pos(&self, text: &str) -> usize {
        let mut pos = 0;
        for (i, line) in text.split('\n').enumerate() {
            if i == self.primary.line {
                return pos + self.primary.col.min(line.len());
            }
            pos += line.len() + 1;
        }
        pos
    }
}

impl Default for CursorState {
    fn default() -> Self {
        Self::new()
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn pos_to_line_col(text: &str, pos: usize) -> (usize, usize) {
    let mut remaining = pos;
    for (i, line) in text.split('\n').enumerate() {
        if remaining <= line.len() {
            return (i, remaining);
        }
        remaining -= line.len() + 1;
    }
    let lines: Vec<&str> = text.split('\n').collect();
    (lines.len().saturating_sub(1), 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_movement() {
        let mut c = Cursor::new(5, 10);
        c.move_up(3, 20);
        assert_eq!(c.line, 2);
        assert_eq!(c.col, 10);
    }

    #[test]
    fn test_selection_empty() {
        let s = Selection::cursor(Cursor::new(0, 0));
        assert!(s.is_empty());
    }

    #[test]
    fn test_selection_normalize() {
        let s = Selection::new(Cursor::new(5, 10), Cursor::new(2, 3));
        let norm = s.normalized();
        assert_eq!(norm.start, Cursor::new(2, 3));
        assert_eq!(norm.end, Cursor::new(5, 10));
    }

    #[test]
    fn test_selection_contains() {
        let s = Selection::new(Cursor::new(1, 0), Cursor::new(3, 5));
        assert!(s.contains(1, 5));
        assert!(s.contains(2, 0));
        assert!(!s.contains(0, 0));
        assert!(!s.contains(3, 5));
    }

    #[test]
    fn test_cursor_state_move_left_right() {
        let mut cs = CursorState::new();
        cs.primary = Cursor::new(0, 3);

        let line_len_fn = |_line: usize| -> usize { 5 };
        cs.move_right(&line_len_fn, 1, false);
        assert_eq!(cs.primary.col, 4);

        cs.move_left(&line_len_fn, false);
        assert_eq!(cs.primary.col, 3);
    }

    #[test]
    fn test_select_all() {
        let mut cs = CursorState::new();
        cs.select_all(10, 5);
        assert!(cs.has_selection());
        let sel = cs.selection();
        assert_eq!(sel.start.line, 0);
        assert_eq!(sel.start.col, 0);
    }

    #[test]
    fn test_select_word() {
        let mut cs = CursorState::new();
        cs.primary = Cursor::new(0, 2);
        cs.select_word("hello world");
        let sel = cs.selection().normalized();
        assert_eq!(sel.start.col, 0);
        assert_eq!(sel.end.col, 5);
    }

    #[test]
    fn test_move_word_left_right() {
        let mut cs = CursorState::new();
        cs.primary = Cursor::new(0, 0);

        cs.move_word_right("hello world foo", false);
        assert_eq!(cs.primary.col, 6);

        cs.move_word_right("hello world foo", false);
        assert_eq!(cs.primary.col, 12);
    }
}
