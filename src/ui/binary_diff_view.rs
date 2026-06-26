use eframe::egui;
use eframe::epaint::Color32;

use crate::app::RustpadApp;
use crate::diff::binary_diff::{format_ascii_byte, format_hex_byte, BinaryHexCell};
use crate::ui::compare_session::CompareSession;

const ROW_HEIGHT: f32 = 18.0;
const OFFSET_W: f32 = 80.0;
const HEX_GROUP_W: f32 = 24.0;
const ASCII_W: f32 = 10.0;

fn cell_bg(cell: BinaryHexCell) -> Color32 {
    if cell.differs() {
        Color32::from_rgb(250, 220, 220)
    } else {
        Color32::TRANSPARENT
    }
}

pub fn show_binary_content(app: &RustpadApp, session: &mut CompareSession, ui: &mut egui::Ui) {
    let Some(result) = session.binary_result.clone() else {
        let t = app.tr();
        ui.centered_and_justified(|ui| ui.label(t.bdiff_pick_files_hint));
        return;
    };

    let t = app.tr();
    let scroll_target = session.binary_scroll_to_row.take();
    let mono_size = session.font_size;

    ui.label(format!(
        "{}: {} B | {}: {} B | {}: {} | {}: {}",
        t.bdiff_left_size,
        result.left_size,
        t.bdiff_right_size,
        result.right_size,
        t.bdiff_identical,
        result.identical_bytes,
        t.bdiff_different,
        result.different_bytes,
    ));
    if result.truncated {
        ui.colored_label(Color32::from_rgb(180, 100, 0), t.bdiff_truncated);
    }
    if session.binary_current_diff + 1 > 0 && result.diff_count() > 0 {
        ui.label(format!(
            "{} {}/{}",
            t.bdiff_diff_at,
            session.binary_current_diff + 1,
            result.diff_count()
        ));
    }
    ui.separator();

    let mut scroll = egui::ScrollArea::vertical().auto_shrink([false, false]);
    if let Some(row) = scroll_target {
        scroll = scroll.vertical_scroll_offset(row as f32 * ROW_HEIGHT);
    }

    scroll.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        let mono = egui::FontId::monospace(mono_size);

        for row in &result.rows {
            let row_rect = ui.available_rect_before_wrap();
            let row_rect = egui::Rect::from_min_size(
                row_rect.min,
                egui::vec2(row_rect.width(), ROW_HEIGHT),
            );
            ui.allocate_rect(row_rect, egui::Sense::hover());
            let painter = ui.painter();
            let cy = row_rect.center().y;
            let mut x = row_rect.left() + 4.0;

            painter.text(
                egui::pos2(x, cy),
                egui::Align2::LEFT_CENTER,
                format!("{:08X}", row.offset),
                mono.clone(),
                Color32::from_gray(100),
            );
            x += OFFSET_W;

            for cell in &row.cells {
                let bg = cell_bg(*cell);
                let hex = format_hex_byte(cell.left);
                let hex_rect = egui::Rect::from_min_size(
                    egui::pos2(x, row_rect.top()),
                    egui::vec2(HEX_GROUP_W, ROW_HEIGHT),
                );
                if bg != Color32::TRANSPARENT {
                    painter.rect_filled(hex_rect, 0.0, bg);
                }
                painter.text(
                    egui::pos2(x + 2.0, cy),
                    egui::Align2::LEFT_CENTER,
                    hex,
                    mono.clone(),
                    ui.style().visuals.text_color(),
                );
                x += HEX_GROUP_W;
            }
            x += 8.0;
            for cell in &row.cells {
                let ch = format_ascii_byte(cell.left);
                painter.text(
                    egui::pos2(x, cy),
                    egui::Align2::LEFT_CENTER,
                    ch.to_string(),
                    mono.clone(),
                    ui.style().visuals.text_color(),
                );
                x += ASCII_W;
            }

            x += 16.0;
            painter.line_segment(
                [egui::pos2(x, row_rect.top()), egui::pos2(x, row_rect.bottom())],
                egui::Stroke::new(1.0, Color32::from_gray(180)),
            );
            x += 8.0;

            for cell in &row.cells {
                let bg = cell_bg(*cell);
                let hex = format_hex_byte(cell.right);
                let hex_rect = egui::Rect::from_min_size(
                    egui::pos2(x, row_rect.top()),
                    egui::vec2(HEX_GROUP_W, ROW_HEIGHT),
                );
                if bg != Color32::TRANSPARENT {
                    painter.rect_filled(hex_rect, 0.0, bg);
                }
                painter.text(
                    egui::pos2(x + 2.0, cy),
                    egui::Align2::LEFT_CENTER,
                    hex,
                    mono.clone(),
                    ui.style().visuals.text_color(),
                );
                x += HEX_GROUP_W;
            }
            x += 8.0;
            for cell in &row.cells {
                let ch = format_ascii_byte(cell.right);
                painter.text(
                    egui::pos2(x, cy),
                    egui::Align2::LEFT_CENTER,
                    ch.to_string(),
                    mono.clone(),
                    ui.style().visuals.text_color(),
                );
                x += ASCII_W;
            }
        }
    });
}
