//! Compare menu (file / folder / history) shared by in-window and macOS menus.

use eframe::egui;

use crate::app::RustpadApp;
use crate::config::ComparePair;
use crate::ui::menu_actions;
use crate::ui::text_util;

pub const HISTORY_MAX: usize = 20;

pub fn history_label(pair: &ComparePair) -> String {
    let left = pair.left.display().to_string();
    let right = pair.right.display().to_string();
    text_util::ellipsis_middle(&format!("{left}  |  {right}"), 80)
}

/// Render the top-level Compare menu (Windows/Linux in-window menu bar).
pub fn show(app: &mut RustpadApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    let t = app.tr();
    let file_history: Vec<_> = app.config.recent_file_compares.clone();
    let folder_history: Vec<_> = app.config.recent_folder_compares.clone();

    ui.menu_button(t.menu_compare, |ui| {
        if ui.button(t.cmp_menu_files).clicked() {
            menu_actions::dispatch(app, "compare.files", ctx);
            ui.close_menu();
        }
        if ui.button(t.cmp_menu_folders).clicked() {
            menu_actions::dispatch(app, "compare.folders", ctx);
            ui.close_menu();
        }
        ui.menu_button(t.cmp_recent, |ui| {
            ui.menu_button(t.cmp_history_folders, |ui| {
                show_history_list(
                    ui,
                    app,
                    ctx,
                    &folder_history,
                    "compare.history.folder.",
                );
            });
            ui.menu_button(t.cmp_history_files, |ui| {
                show_history_list(ui, app, ctx, &file_history, "compare.history.file.");
            });
        });
    });
}

fn show_history_list(
    ui: &mut egui::Ui,
    app: &mut RustpadApp,
    ctx: &egui::Context,
    history: &[ComparePair],
    id_prefix: &str,
) {
    let t = app.tr();
    if history.is_empty() {
        ui.add_enabled(false, egui::Button::new(t.cmp_history_empty));
        return;
    }
    for (index, pair) in history.iter().enumerate() {
        let label = history_label(pair);
        if ui.button(label).clicked() {
            menu_actions::dispatch(app, &format!("{id_prefix}{index}"), ctx);
            ui.close_menu();
        }
    }
}
