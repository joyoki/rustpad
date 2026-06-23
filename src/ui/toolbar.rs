use eframe::egui;

use crate::app::RustpadApp;

/// Render the toolbar with common action buttons.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    let t = app.tr();
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button(t.tb_new).on_hover_text(t.tip_new).clicked() {
                app.tab_manager.new_tab();
            }
            if ui.button(t.tb_open).on_hover_text(t.tip_open).clicked() {
                app.pending_open_file = true;
            }
            if ui.button(t.tb_save).on_hover_text(t.tip_save).clicked() {
                app.save_current_tab();
            }
            ui.separator();
            let can_undo = app.tab_manager.active().buffer.can_undo();
            if ui
                .add_enabled(can_undo, egui::Button::new(t.tb_undo))
                .on_hover_text(t.tip_undo)
                .clicked()
            {
                app.tab_manager.active_mut().buffer.undo();
            }
            let can_redo = app.tab_manager.active().buffer.can_redo();
            if ui
                .add_enabled(can_redo, egui::Button::new(t.tb_redo))
                .on_hover_text(t.tip_redo)
                .clicked()
            {
                app.tab_manager.active_mut().buffer.redo();
            }
            ui.separator();
            if ui.button(t.tb_find).on_hover_text(t.tip_find).clicked() {
                app.open_find(false);
            }
            if ui.button(t.tb_compare).on_hover_text(t.tip_compare).clicked() {
                app.pending_compare_files = true;
            }
            ui.separator();
            if ui.button("A-").on_hover_text(t.tip_font_dec).clicked() {
                app.config.editor.font_size = (app.config.editor.font_size - 1.0).max(8.0);
            }
            ui.label(format!("{}px", app.config.editor.font_size as u32));
            if ui.button("A+").on_hover_text(t.tip_font_inc).clicked() {
                app.config.editor.font_size = (app.config.editor.font_size + 1.0).min(72.0);
            }
        });
    });
}
