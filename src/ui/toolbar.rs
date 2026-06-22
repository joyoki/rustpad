use eframe::egui;

use crate::app::RustpadApp;

/// Render the toolbar with common action buttons.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui
                .button("📄 New")
                .on_hover_text("New file (Ctrl+N)")
                .clicked()
            {
                app.tab_manager.new_tab();
            }
            if ui
                .button("📂 Open")
                .on_hover_text("Open file (Ctrl+O)")
                .clicked()
            {
                app.pending_open_file = true;
            }
            if ui
                .button("💾 Save")
                .on_hover_text("Save (Ctrl+S)")
                .clicked()
            {
                app.save_current_tab();
            }
            ui.separator();
            let can_undo = app.tab_manager.active().buffer.can_undo();
            if ui
                .add_enabled(can_undo, egui::Button::new("↩ Undo"))
                .on_hover_text("Undo (Ctrl+Z)")
                .clicked()
            {
                app.tab_manager.active_mut().buffer.undo();
            }
            let can_redo = app.tab_manager.active().buffer.can_redo();
            if ui
                .add_enabled(can_redo, egui::Button::new("↪ Redo"))
                .on_hover_text("Redo (Ctrl+Y)")
                .clicked()
            {
                app.tab_manager.active_mut().buffer.redo();
            }
            ui.separator();
            if ui
                .button("🔍 Find")
                .on_hover_text("Find (Ctrl+F)")
                .clicked()
            {
                app.open_find(false);
            }
            if ui
                .button("⇔ Compare")
                .on_hover_text("Compare files (Ctrl+D)")
                .clicked()
            {
                app.pending_compare_files = true;
            }
            ui.separator();
            if ui
                .button("A-")
                .on_hover_text("Decrease font size")
                .clicked()
            {
                app.config.editor.font_size = (app.config.editor.font_size - 1.0).max(8.0);
            }
            ui.label(format!("{}px", app.config.editor.font_size as u32));
            if ui
                .button("A+")
                .on_hover_text("Increase font size")
                .clicked()
            {
                app.config.editor.font_size = (app.config.editor.font_size + 1.0).min(72.0);
            }
        });
    });
}
