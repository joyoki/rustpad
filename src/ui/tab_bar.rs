use eframe::egui;

use crate::app::RustpadApp;

/// Render the tab bar showing all open tabs.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let tab_count = app.tab_manager.tab_count();
            let active_idx = app.tab_manager.active_index();
            let mut tab_to_close: Option<usize> = None;
            let mut tab_to_activate: Option<usize> = None;

            for i in 0..tab_count {
                let title = app.tab_manager.tabs()[i].display_title();
                let is_active = i == active_idx;

                let response = ui.selectable_label(is_active, &title);
                if response.clicked() {
                    tab_to_activate = Some(i);
                }

                // Close button on hover
                if is_active || response.hovered() {
                    let close_btn = ui.small_button("×");
                    if close_btn.clicked() {
                        tab_to_close = Some(i);
                    }
                }

                if i < tab_count - 1 {
                    ui.separator();
                }
            }

            // New tab button
            if ui.button("+").on_hover_text("New tab").clicked() {
                app.tab_manager.new_tab();
            }

            // Apply deferred actions
            if let Some(idx) = tab_to_activate {
                if idx != app.tab_manager.active_index() {
                    app.tab_manager.set_active(idx);
                    app.highlighter.clear_cache();
                    app.highlighter.invalidate_all();
                }
            }
            if let Some(idx) = tab_to_close {
                let is_dirty = app.tab_manager.tabs()[idx].buffer.is_dirty()
                    || app.tab_manager.tabs()[idx].modified;
                if is_dirty {
                    app.pending_close_tab = Some(idx);
                    app.show_unsaved_dialog = true;
                } else {
                    app.tab_manager.close_tab(idx);
                }
            }
        });
    });
}
