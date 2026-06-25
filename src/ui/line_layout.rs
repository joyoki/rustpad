//! Caret column layout — Scintilla-style per-character X positions.
//!
//! notepad-- uses QScintilla/Scintilla (`LineLayout::FindPositionFromX` in
//! `PositionCache.cpp`): build cumulative glyph widths, map pointer X to a caret
//! column with per-character midpoint snapping (`charPosition=false`).

use eframe::egui;
use eframe::epaint::Color32;

use crate::editor::context_actions;

/// Display options shared by layout measurement and editor painting.
#[derive(Debug, Clone, Copy)]
pub struct EditorDisplayOpts {
    pub font_size: f32,
    pub display_blank: bool,
    pub show_tabs_as_spaces: bool,
    pub display_non_print: bool,
}

pub fn display_line_text(
    line_text: &str,
    display_blank: bool,
    show_tabs_as_spaces: bool,
    display_non_print: bool,
) -> String {
    let mut display = line_text.to_string();
    if display_blank || show_tabs_as_spaces {
        display = context_actions::visualize_whitespace(
            &display,
            display_blank,
            show_tabs_as_spaces,
        );
    }
    if display_non_print {
        display = context_actions::visualize_non_prints(&display);
    }
    display
}

/// Cumulative X offset from the line start for each caret column `0..=line_len`.
#[derive(Debug, Clone)]
pub struct LineCaretLayout {
    positions: Vec<f32>,
}

impl LineCaretLayout {
    pub fn empty() -> Self {
        Self {
            positions: vec![0.0],
        }
    }

    pub fn line_len(&self) -> usize {
        self.positions.len().saturating_sub(1)
    }

    /// Measure caret columns using the same per-character display rules as the editor.
    pub fn measure(
        painter: &egui::Painter,
        opts: EditorDisplayOpts,
        line_text: &str,
    ) -> Self {
        let mut positions = vec![0.0];
        let mut x = 0.0f32;
        let font = egui::FontId::monospace(opts.font_size);
        for ch in line_text.chars() {
            let display = char_display_text(ch, opts);
            let galley = painter.layout_no_wrap(display, font.clone(), Color32::TRANSPARENT);
            x += galley.size().x;
            positions.push(x);
        }
        Self { positions }
    }

    pub fn col_to_x(&self, base_x: f32, col: usize) -> f32 {
        let idx = col.min(self.positions.len().saturating_sub(1));
        base_x + self.positions[idx]
    }

    pub fn caret_col_from_rel_x(&self, rel_x: f32) -> usize {
        caret_col_from_x(&self.positions, rel_x)
    }

    pub fn char_index_from_rel_x(&self, rel_x: f32) -> usize {
        char_index_from_x(&self.positions, rel_x)
    }
}

fn char_display_text(ch: char, opts: EditorDisplayOpts) -> String {
    let mut s = ch.to_string();
    if opts.display_blank || opts.show_tabs_as_spaces {
        s = context_actions::visualize_whitespace(
            &s,
            opts.display_blank,
            opts.show_tabs_as_spaces,
        );
    }
    if opts.display_non_print {
        s = context_actions::visualize_non_prints(&s);
    }
    s
}

/// Scintilla `LineLayout::FindBefore`.
fn find_x_before(positions: &[f32], x: f32) -> usize {
    let end = positions.len().saturating_sub(1);
    if end == 0 {
        return 0;
    }
    let mut lower = 0usize;
    let mut upper = end;
    while lower < upper {
        let middle = (upper + lower).div_ceil(2);
        if x < positions[middle] {
            upper = middle - 1;
        } else {
            lower = middle;
        }
    }
    lower
}

/// Scintilla `FindPositionFromX` with `charPosition=false` (caret / stream selection).
pub fn caret_col_from_x(positions: &[f32], x: f32) -> usize {
    let end = positions.len().saturating_sub(1);
    if end == 0 {
        return 0;
    }
    let x = x.max(0.0);
    let mut pos = find_x_before(positions, x);
    while pos < end {
        let mid = (positions[pos] + positions[pos + 1]) / 2.0;
        if x < mid {
            return pos;
        }
        pos += 1;
    }
    end
}

/// Scintilla `FindPositionFromX` with `charPosition=true` (glyph hit tests).
pub fn char_index_from_x(positions: &[f32], x: f32) -> usize {
    let end = positions.len().saturating_sub(1);
    if end == 0 {
        return 0;
    }
    let x = x.max(0.0);
    let mut pos = find_x_before(positions, x);
    while pos < end {
        if x < positions[pos + 1] {
            return pos;
        }
        pos += 1;
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_col_uses_glyph_midpoint() {
        // Three glyphs: 10px + 20px + 10px
        let positions = vec![0.0, 10.0, 30.0, 40.0];
        assert_eq!(caret_col_from_x(&positions, 0.0), 0);
        assert_eq!(caret_col_from_x(&positions, 4.0), 0);
        assert_eq!(caret_col_from_x(&positions, 10.0), 1);
        assert_eq!(caret_col_from_x(&positions, 19.0), 1);
        assert_eq!(caret_col_from_x(&positions, 20.0), 2);
        assert_eq!(caret_col_from_x(&positions, 39.0), 3);
    }

    #[test]
    fn char_index_covers_whole_glyph() {
        let positions = vec![0.0, 10.0, 30.0, 40.0];
        assert_eq!(char_index_from_x(&positions, 0.0), 0);
        assert_eq!(char_index_from_x(&positions, 9.9), 0);
        assert_eq!(char_index_from_x(&positions, 10.0), 1);
        assert_eq!(char_index_from_x(&positions, 29.9), 1);
        assert_eq!(char_index_from_x(&positions, 30.0), 2);
    }

    #[test]
    fn single_char_selection_span() {
        let positions = vec![0.0, 12.0];
        assert_eq!(caret_col_from_x(&positions, 5.9), 0);
        assert_eq!(caret_col_from_x(&positions, 6.0), 1);
        assert_eq!(caret_col_from_x(&positions, 11.9), 1);
    }
}
