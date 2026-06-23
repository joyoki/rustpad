use eframe::egui;

use crate::app::RustpadApp;

/// Render the bottom status bar.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    // Pre-compute read-only values to avoid borrow conflicts inside the closure.
    let tab = app.tab_manager.active();
    let cursor_line = tab.cursor.line;
    let cursor_col = tab.cursor.col;
    let line_count = tab.line_count();
    let encoding = format!("{:?}", tab.encoding);
    let line_ending = format!("{:?}", tab.buffer.line_ending());
    let size = tab.buffer.len();
    let file_path = tab.file_path.clone();

    // Determine the effective language name (manual override or auto-detected).
    let filename = file_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "untitled.txt".to_string());
    let current_language = app.highlighter.syntax_name_for_file(
        &filename,
        tab.syntax_override.as_deref(),
    );

    let t = app.tr();

    let mut selected_language: Option<String> = None;
    let transient = app.transient_message.clone();

    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if !transient.is_empty() {
                ui.label(
                    egui::RichText::new(&transient)
                        .italics()
                        .color(egui::Color32::from_rgb(0, 100, 180)),
                );
                ui.separator();
            }
            // Cursor position
            ui.label(format!("Ln {}, Col {}", cursor_line + 1, cursor_col + 1));
            ui.separator();

            // Line count
            ui.label(format!("{} {}", line_count, t.status_lines));
            ui.separator();

            // Encoding
            ui.label(&encoding);
            ui.separator();

            // Line ending
            ui.label(&line_ending);
            ui.separator();

            // File size
            if size > 1024 * 1024 {
                ui.label(format!("{:.1} MB", size as f64 / 1024.0 / 1024.0));
            } else if size > 1024 {
                ui.label(format!("{:.1} KB", size as f64 / 1024.0));
            } else {
                ui.label(format!("{} B", size));
            }
            ui.separator();

            // Language selector
            ui.label(t.status_lang);
            egui::ComboBox::from_id_salt("language_selector")
                .selected_text(&current_language)
                .width(150.0)
                .show_ui(ui, |ui| {
                    for name in app.highlighter.syntax_names() {
                        if ui
                            .selectable_label(name == current_language, &name)
                            .clicked()
                        {
                            selected_language = Some(name);
                        }
                    }
                });

            // Right side: file path
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(path) = &file_path {
                    ui.label(path.to_string_lossy());
                }
            });
        });
    });

    // Apply a manual language change after the UI closure (avoids borrow issues).
    if let Some(name) = selected_language {
        app.tab_manager.active_mut().syntax_override = Some(name);
        app.highlighter.clear_cache();
        app.highlighter.invalidate_all();
    }
}
