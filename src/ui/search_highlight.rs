//! Shared yellow/orange styling for search match highlights (editor + results panel).

use eframe::egui::{self, Color32, RichText};

use crate::config::theme::EditorTheme;

/// Whether the editor should paint in-buffer search match highlights.
pub fn should_paint_editor_highlights(
    show_search: bool,
    show_search_results: bool,
    pattern_empty: bool,
    match_count: usize,
) -> bool {
    !pattern_empty
        && match_count > 0
        && (show_search || show_search_results)
}

/// Split a line preview into before / match / after for UI highlighting.
pub fn split_line_match(preview: &str, col: usize, len: usize) -> (String, String, String) {
    let chars: Vec<char> = preview.chars().collect();
    let col = col.min(chars.len());
    let len = len.min(chars.len().saturating_sub(col));
    let before: String = chars[..col].iter().collect();
    let matched: String = chars[col..col + len].iter().collect();
    let after: String = chars[col + len..].iter().collect();
    (before, matched, after)
}

/// Paint one search-result row with the matched substring on a yellow background.
pub fn paint_result_preview(
    ui: &mut egui::Ui,
    preview: &str,
    col: usize,
    match_len: usize,
    is_current: bool,
    theme: &EditorTheme,
) {
    let (before, matched, after) = split_line_match(preview, col, match_len);
    let match_bg = if is_current {
        theme.search_current_highlight_bg_color()
    } else {
        theme.search_highlight_bg_color()
    };

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if !before.is_empty() {
            ui.label(RichText::new(before).monospace());
        }
        if !matched.is_empty() {
            ui.label(
                RichText::new(matched)
                    .monospace()
                    .background_color(match_bg)
                    .color(Color32::BLACK),
            );
        } else if match_len > 0 {
            // Column points past visible preview (trimmed line); still show a marker.
            ui.label(
                RichText::new("▏")
                    .monospace()
                    .background_color(match_bg)
                    .color(Color32::BLACK),
            );
        }
        if !after.is_empty() {
            ui.label(RichText::new(after).monospace());
        }
    });
}

/// Painter-only preview for scrollable result rows (no nested widgets stealing input).
pub fn paint_result_preview_painter(
    ui: &egui::Ui,
    rect: egui::Rect,
    preview: &str,
    col: usize,
    match_len: usize,
    is_current: bool,
    theme: &EditorTheme,
) {
    use egui::text::{LayoutJob, TextFormat};

    let (before, matched, after) = split_line_match(preview, col, match_len);
    let match_bg = if is_current {
        theme.search_current_highlight_bg_color()
    } else {
        theme.search_highlight_bg_color()
    };

    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let plain = TextFormat {
        font_id: font.clone(),
        color: ui.visuals().text_color(),
        ..Default::default()
    };
    let hl = TextFormat {
        font_id: font,
        color: Color32::BLACK,
        background: match_bg,
        ..Default::default()
    };

    let mut job = LayoutJob::default();
    if !before.is_empty() {
        job.append(&before, 0.0, plain.clone());
    }
    if !matched.is_empty() {
        job.append(&matched, 0.0, hl.clone());
    } else if match_len > 0 {
        job.append("▏", 0.0, hl);
    }
    if !after.is_empty() {
        job.append(&after, 0.0, plain);
    }

    let galley = ui.fonts(|f| f.layout_job(job));
    let pos = egui::pos2(
        rect.min.x,
        rect.center().y - galley.size().y * 0.5,
    );
    ui.painter()
        .with_clip_rect(rect)
        .galley(pos, galley, ui.visuals().text_color());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_line_match_basic() {
        let (a, m, b) = split_line_match("hello world", 6, 5);
        assert_eq!(a, "hello ");
        assert_eq!(m, "world");
        assert_eq!(b, "");
    }
}
