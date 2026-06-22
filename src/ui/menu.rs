use eframe::egui;

use crate::app::RustpadApp;

/// Render the top menu bar.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New  Ctrl+N").clicked() {
                    app.tab_manager.new_tab();
                    ui.close_menu();
                }
                if ui.button("Open...  Ctrl+O").clicked() {
                    app.pending_open_file = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Save  Ctrl+S").clicked() {
                    app.save_current_tab();
                    ui.close_menu();
                }
                if ui.button("Save As...  Ctrl+Shift+S").clicked() {
                    app.save_as_dialog();
                    ui.close_menu();
                }
                if ui.button("Save All").clicked() {
                    app.save_all_tabs();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Close Tab  Ctrl+W").clicked() {
                    app.close_current_tab();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Compare Files...  Ctrl+D").clicked() {
                    app.pending_compare_files = true;
                    ui.close_menu();
                }
                if ui.button("Compare Current File With...").clicked() {
                    app.pending_compare_current = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit  Alt+F4").clicked() {
                    app.request_exit(ctx);
                    ui.close_menu();
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo  Ctrl+Z").clicked() {
                    app.tab_manager.active_mut().buffer.undo();
                    ui.close_menu();
                }
                if ui.button("Redo  Ctrl+Y").clicked() {
                    app.tab_manager.active_mut().buffer.redo();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Cut  Ctrl+X").clicked() {
                    app.cut();
                    ui.close_menu();
                }
                if ui.button("Copy  Ctrl+C").clicked() {
                    app.copy();
                    ui.close_menu();
                }
                if ui.button("Paste  Ctrl+V").clicked() {
                    app.paste();
                    ui.close_menu();
                }
                if ui.button("Select All  Ctrl+A").clicked() {
                    app.select_all();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Find...  Ctrl+F").clicked() {
                    app.open_find(false);
                    ui.close_menu();
                }
                if ui.button("Replace...  Ctrl+H").clicked() {
                    app.open_find(true);
                    ui.close_menu();
                }
                if ui.button("Go to Line...  Ctrl+G").clicked() {
                    app.show_goto_line = true;
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Toggle Sidebar  Ctrl+B").clicked() {
                    app.show_sidebar = !app.show_sidebar;
                    ui.close_menu();
                }
                if ui.button("Toggle Minimap").clicked() {
                    app.config.ui.show_minimap = !app.config.ui.show_minimap;
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("Language", |ui| {
                    let filename = app
                        .tab_manager
                        .active()
                        .file_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| app.tab_manager.active().title.clone());
                    let current = app.highlighter.syntax_name_for_file(
                        &filename,
                        app.tab_manager.active().syntax_override.as_deref(),
                    );

                    let is_auto = app.tab_manager.active().syntax_override.is_none();

                    if ui.selectable_label(is_auto, "Auto Detect").clicked() {
                        app.clear_active_language();
                        ui.close_menu();
                    }
                    ui.separator();
                    for &lang in crate::highlight::MENU_LANGUAGES {
                        if ui.selectable_label(current == lang, lang).clicked() {
                            app.set_active_language(lang);
                            ui.close_menu();
                        }
                    }
                });
                ui.separator();
                ui.menu_button("Font Size", |ui| {
                    for size in [10, 12, 14, 16, 18, 20, 24] {
                        if ui
                            .selectable_label(
                                app.config.editor.font_size == size as f32,
                                format!("{}px", size),
                            )
                            .clicked()
                        {
                            app.config.editor.font_size = size as f32;
                        }
                    }
                });
                ui.separator();
                if ui.button("Word Wrap").clicked() {
                    app.config.editor.word_wrap = !app.config.editor.word_wrap;
                    ui.close_menu();
                }
                if ui.button("Line Numbers").clicked() {
                    app.config.editor.show_line_numbers = !app.config.editor.show_line_numbers;
                    ui.close_menu();
                }
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("Compare Files...  Ctrl+D").clicked() {
                    app.pending_compare_files = true;
                    ui.close_menu();
                }
                if ui.button("Macro Recording").clicked() {
                    app.toggle_macro_recording();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Preferences...").clicked() {
                    app.show_preferences = true;
                    ui.close_menu();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About Rustpad").clicked() {
                    app.show_about = true;
                    ui.close_menu();
                }
            });
        });
    });
}
