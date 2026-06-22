use eframe::egui;
use eframe::epaint::Color32;

use crate::app::RustpadApp;
use crate::diff::{DiffRow, DiffTag};

const ROW_HEIGHT: f32 = 18.0;
const FONT_SIZE: f32 = 13.0;
const GUTTER_W: f32 = 48.0;
const CENTER_W: f32 = 64.0;

// Notepad-- style color coding.
fn equal_bg() -> Color32 {
    Color32::TRANSPARENT
}
fn insert_bg() -> Color32 {
    Color32::from_rgb(214, 245, 214) // green
}
fn delete_bg() -> Color32 {
    Color32::from_rgb(250, 220, 220) // red
}
fn replace_bg() -> Color32 {
    Color32::from_rgb(250, 244, 205) // yellow
}
fn gap_bg() -> Color32 {
    Color32::from_gray(232) // empty opposite side
}
fn inline_hl() -> Color32 {
    Color32::from_rgb(247, 214, 124) // darker yellow for changed chars
}

/// Show the side-by-side diff view.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_diff_view {
        return;
    }

    let diff_result = match &app.diff_result {
        Some(result) => result.clone(),
        None => return,
    };

    let left_name = app
        .diff_left_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Left".to_string());
    let right_name = app
        .diff_right_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Right".to_string());

    let scroll_target = app.diff_scroll_to_row.take();

    egui::CentralPanel::default().show(ctx, |ui| {
        // Header with file names and dirty markers.
        ui.horizontal(|ui| {
            let left_mark = if app.diff_left_dirty { " *" } else { "" };
            let right_mark = if app.diff_right_dirty { " *" } else { "" };
            ui.label(
                egui::RichText::new(format!("◀ {left_name}{left_mark}"))
                    .strong()
                    .color(Color32::from_rgb(180, 60, 60)),
            );
            ui.separator();
            ui.label(
                egui::RichText::new(format!("{right_name}{right_mark} ▶"))
                    .strong()
                    .color(Color32::from_rgb(60, 140, 60)),
            );
        });
        ui.separator();

        let total_rows = diff_result.rows.len();
        let mut scroll = egui::ScrollArea::vertical().auto_shrink([false, false]);
        if let Some(row) = scroll_target {
            scroll = scroll.vertical_scroll_offset(row as f32 * ROW_HEIGHT);
        }

        scroll.show_rows(ui, ROW_HEIGHT, total_rows, |ui, row_range| {
            let full_w = ui.available_width();
            let col_w = ((full_w - CENTER_W - 2.0 * GUTTER_W) / 2.0).max(80.0);

            for i in row_range {
                let row = &diff_result.rows[i];
                let is_block_start = diff_result.change_starts.contains(&i);
                render_row(app, ui, row, col_w, is_block_start);
            }
        });

        ui.separator();
        ui.horizontal(|ui| {
            let s = &diff_result.stats;
            ui.label(format!(
                "Changes: {} | Equal: {} | Inserted: {} | Deleted: {} | Replaced: {}",
                diff_result.change_count(),
                s.equal,
                s.insertions,
                s.deletions,
                s.replacements,
            ));
            if diff_result.change_count() > 0 {
                ui.separator();
                ui.label(format!(
                    "At change {}/{}",
                    app.diff_current_change + 1,
                    diff_result.change_count()
                ));
            }
        });
    });
}

fn render_row(
    app: &mut RustpadApp,
    ui: &mut egui::Ui,
    row: &DiffRow,
    col_w: f32,
    is_block_start: bool,
) {
    let (left_bg, right_bg) = row_backgrounds(row);

    ui.horizontal(|ui| {
        // Left gutter + content.
        gutter(ui, row.left_line);
        cell(
            ui,
            col_w,
            left_bg,
            row.left_text.as_deref(),
            &row.left_spans,
        );

        // Center merge controls (only on the first row of a change block).
        ui.allocate_ui_with_layout(
            egui::vec2(CENTER_W, ROW_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                if is_block_start {
                    if let Some(change_id) = row.change_id {
                        if ui
                            .small_button("▶")
                            .on_hover_text("Copy this change to the right")
                            .clicked()
                        {
                            app.diff_merge_to_right(change_id);
                        }
                        if ui
                            .small_button("◀")
                            .on_hover_text("Copy this change to the left")
                            .clicked()
                        {
                            app.diff_merge_to_left(change_id);
                        }
                    }
                }
            },
        );

        // Right gutter + content.
        gutter(ui, row.right_line);
        cell(
            ui,
            col_w,
            right_bg,
            row.right_text.as_deref(),
            &row.right_spans,
        );
    });
}

fn row_backgrounds(row: &DiffRow) -> (Color32, Color32) {
    match row.tag {
        DiffTag::Equal => (equal_bg(), equal_bg()),
        DiffTag::Delete => (delete_bg(), gap_bg()),
        DiffTag::Insert => (gap_bg(), insert_bg()),
        DiffTag::Replace => (replace_bg(), replace_bg()),
    }
}

fn gutter(ui: &mut egui::Ui, line: Option<usize>) {
    let text = line.map(|n| format!("{:>4}", n + 1)).unwrap_or_default();
    ui.allocate_ui_with_layout(
        egui::vec2(GUTTER_W, ROW_HEIGHT),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            ui.label(
                egui::RichText::new(text)
                    .monospace()
                    .size(FONT_SIZE - 1.0)
                    .color(Color32::from_gray(120)),
            );
        },
    );
}

/// Render one side's text cell with a background and optional inline highlights.
fn cell(ui: &mut egui::Ui, width: f32, bg: Color32, text: Option<&str>, spans: &[(usize, usize)]) {
    let (rect, _resp) = ui.allocate_exact_size(egui::vec2(width, ROW_HEIGHT), egui::Sense::hover());
    if bg != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 0.0, bg);
    }

    let Some(text) = text else { return };
    let job = build_job(text, spans);
    let galley = ui.fonts(|f| f.layout_job(job));
    let pos = egui::pos2(rect.min.x + 4.0, rect.min.y + 1.0);
    ui.painter().galley(pos, galley, Color32::BLACK);
}

/// Build a layout job for a line, highlighting the changed char spans.
fn build_job(text: &str, spans: &[(usize, usize)]) -> egui::text::LayoutJob {
    use egui::text::{LayoutJob, TextFormat};
    let mut job = LayoutJob::default();
    let font = egui::FontId::monospace(FONT_SIZE);

    let plain = TextFormat {
        font_id: font.clone(),
        color: Color32::BLACK,
        ..Default::default()
    };
    let hl = TextFormat {
        font_id: font,
        color: Color32::BLACK,
        background: inline_hl(),
        ..Default::default()
    };

    if spans.is_empty() {
        job.append(text, 0.0, plain);
        return job;
    }

    let chars: Vec<char> = text.chars().collect();
    let mut idx = 0usize;
    for &(start, end) in spans {
        let start = start.min(chars.len());
        let end = end.min(chars.len());
        if start > idx {
            let seg: String = chars[idx..start].iter().collect();
            job.append(&seg, 0.0, plain.clone());
        }
        if end > start {
            let seg: String = chars[start..end].iter().collect();
            job.append(&seg, 0.0, hl.clone());
        }
        idx = end.max(idx);
    }
    if idx < chars.len() {
        let seg: String = chars[idx..].iter().collect();
        job.append(&seg, 0.0, plain);
    }

    job
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_diff_view_compiles() {
        assert!(true);
    }
}
