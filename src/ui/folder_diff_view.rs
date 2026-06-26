use eframe::egui;
use eframe::epaint::Color32;

use crate::app::RustpadApp;
use crate::diff::folder_diff::{format_mtime, format_size};
use crate::diff::{FileStatus, FolderDiffEntry};
use crate::ui::compare_session::CompareSession;
use crate::ui::compare_window;

const ROW_HEIGHT: f32 = 22.0;

fn status_label(status: FileStatus, t: &crate::i18n::Locale) -> &'static str {
    match status {
        FileStatus::Identical => t.fdiff_status_identical,
        FileStatus::Different => t.fdiff_status_different,
        FileStatus::LeftOnly => t.fdiff_status_left_only,
        FileStatus::RightOnly => t.fdiff_status_right_only,
    }
}

fn status_color(status: FileStatus) -> Color32 {
    match status {
        FileStatus::Identical => Color32::from_gray(120),
        FileStatus::Different => Color32::from_rgb(180, 130, 0),
        FileStatus::LeftOnly => Color32::from_rgb(180, 60, 60),
        FileStatus::RightOnly => Color32::from_rgb(60, 140, 60),
    }
}

fn row_bg(status: FileStatus, selected: bool) -> Color32 {
    if selected {
        return Color32::from_rgb(180, 210, 255);
    }
    match status {
        FileStatus::Identical => Color32::TRANSPARENT,
        FileStatus::Different => Color32::from_rgb(250, 244, 205),
        FileStatus::LeftOnly => Color32::from_rgb(250, 230, 230),
        FileStatus::RightOnly => Color32::from_rgb(230, 245, 230),
    }
}

pub fn show_folder_content(app: &RustpadApp, session: &mut CompareSession, ui: &mut egui::Ui) {
    compare_window::show_folder_extras(app, session, ui);

    let Some(result) = session.folder_result.clone() else {
        let t = app.tr();
        ui.centered_and_justified(|ui| ui.label(t.fdiff_pick_dirs));
        return;
    };

    let t = app.tr();
    let filter = session.folder_filter;
    let entries: Vec<(usize, FolderDiffEntry)> = result
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| filter.matches(e.status))
        .map(|(i, e)| (i, e.clone()))
        .collect();

    let left_root = result.left_root.display().to_string();
    let right_root = result.right_root.display().to_string();
    let stats = result.stats;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("◀ {left_root}"))
                .strong()
                .color(Color32::from_rgb(180, 60, 60)),
        );
        ui.separator();
        ui.label(
            egui::RichText::new(format!("{right_root} ▶"))
                .strong()
                .color(Color32::from_rgb(60, 140, 60)),
        );
    });
    ui.label(format!(
        "{}: {} | {}: {} | {}: {} | {}: {}",
        t.fdiff_stat_identical,
        stats.identical,
        t.fdiff_stat_different,
        stats.different,
        t.fdiff_stat_left_only,
        stats.left_only,
        t.fdiff_stat_right_only,
        stats.right_only,
    ));
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(72.0, ROW_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| ui.strong(t.fdiff_col_status),
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width() * 0.32, ROW_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| ui.strong(t.fdiff_col_name),
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(72.0, ROW_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| ui.strong(t.fdiff_col_left_size),
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(72.0, ROW_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| ui.strong(t.fdiff_col_right_size),
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(56.0, ROW_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| ui.strong(t.fdiff_col_left_time),
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(56.0, ROW_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| ui.strong(t.fdiff_col_right_time),
                );
            });
            ui.separator();

            if entries.is_empty() {
                ui.label(t.fdiff_no_entries);
                return;
            }

            for (idx, entry) in entries {
                let selected = session.folder_selected == Some(idx);
                let row_rect = ui.available_rect_before_wrap();
                let row_rect = egui::Rect::from_min_size(
                    row_rect.min,
                    egui::vec2(row_rect.width(), ROW_HEIGHT),
                );
                let response = ui.allocate_rect(row_rect, egui::Sense::click());

                if response.clicked() {
                    session.folder_selected = Some(idx);
                }
                if response.double_clicked() {
                    session.pending_open_file_compare = Some(idx);
                }

                let painter = ui.painter();
                painter.rect_filled(row_rect, 0.0, row_bg(entry.status, selected));

                let mut x = row_rect.left() + 4.0;
                let cy = row_rect.center().y;

                painter.text(
                    egui::pos2(x, cy),
                    egui::Align2::LEFT_CENTER,
                    status_label(entry.status, t),
                    egui::FontId::proportional(12.0),
                    status_color(entry.status),
                );
                x += 72.0;

                let name_w = row_rect.width() * 0.32;
                painter.text(
                    egui::pos2(x, cy),
                    egui::Align2::LEFT_CENTER,
                    &entry.relative_path,
                    egui::FontId::monospace(12.0),
                    ui.style().visuals.text_color(),
                );
                x += name_w;

                let left_size = if entry.left_path.is_some() {
                    format_size(entry.left_size)
                } else {
                    "—".to_string()
                };
                painter.text(
                    egui::pos2(x, cy),
                    egui::Align2::LEFT_CENTER,
                    left_size,
                    egui::FontId::monospace(11.0),
                    ui.style().visuals.text_color(),
                );
                x += 72.0;

                let right_size = if entry.right_path.is_some() {
                    format_size(entry.right_size)
                } else {
                    "—".to_string()
                };
                painter.text(
                    egui::pos2(x, cy),
                    egui::Align2::LEFT_CENTER,
                    right_size,
                    egui::FontId::monospace(11.0),
                    ui.style().visuals.text_color(),
                );
                x += 72.0;

                painter.text(
                    egui::pos2(x, cy),
                    egui::Align2::LEFT_CENTER,
                    format_mtime(entry.left_mtime),
                    egui::FontId::monospace(11.0),
                    ui.style().visuals.text_color(),
                );
                x += 56.0;

                painter.text(
                    egui::pos2(x, cy),
                    egui::Align2::LEFT_CENTER,
                    format_mtime(entry.right_mtime),
                    egui::FontId::monospace(11.0),
                    ui.style().visuals.text_color(),
                );

                response.context_menu(|ui| {
                    if entry.status == FileStatus::Different
                        && entry.left_path.is_some()
                        && entry.right_path.is_some()
                    {
                        if ui.button(t.fdiff_open_file_compare).clicked() {
                            session.pending_open_file_compare = Some(idx);
                            ui.close_menu();
                        }
                        if ui.button(t.fdiff_open_binary_compare).clicked() {
                            session.pending_open_binary_compare = Some(idx);
                            ui.close_menu();
                        }
                        ui.separator();
                    }
                    if entry.left_path.is_some() && ui.button(t.fdiff_copy_to_right).clicked() {
                        session.folder_copy_to_right(idx);
                        ui.close_menu();
                    }
                    if entry.right_path.is_some() && ui.button(t.fdiff_copy_to_left).clicked() {
                        session.folder_copy_to_left(idx);
                        ui.close_menu();
                    }
                });
            }
        });
}
