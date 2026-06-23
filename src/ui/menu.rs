use eframe::egui;

use crate::app::RustpadApp;

/// Render the top menu bar.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    let t = app.tr();
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(t.menu_file, |ui| {
                if ui.button(t.file_new).clicked() {
                    app.tab_manager.new_tab();
                    ui.close_menu();
                }
                if ui.button(t.file_open).clicked() {
                    app.pending_open_file = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.file_save).clicked() {
                    app.save_current_tab();
                    ui.close_menu();
                }
                if ui.button(t.file_save_as).clicked() {
                    app.save_as_dialog();
                    ui.close_menu();
                }
                if ui.button(t.file_save_all).clicked() {
                    app.save_all_tabs();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.file_close_tab).clicked() {
                    app.close_current_tab();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.file_compare).clicked() {
                    app.pending_compare_files = true;
                    ui.close_menu();
                }
                if ui.button(t.file_compare_current).clicked() {
                    app.pending_compare_current = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.file_exit).clicked() {
                    app.request_exit(ctx);
                    ui.close_menu();
                }
            });

            ui.menu_button(t.menu_edit, |ui| {
                if ui.button(t.edit_undo).clicked() {
                    app.tab_manager.active_mut().buffer.undo();
                    ui.close_menu();
                }
                if ui.button(t.edit_redo).clicked() {
                    app.tab_manager.active_mut().buffer.redo();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.edit_cut).clicked() {
                    app.cut();
                    ui.close_menu();
                }
                if ui.button(t.edit_copy).clicked() {
                    app.copy();
                    ui.close_menu();
                }
                if ui.button(t.edit_paste).clicked() {
                    app.paste();
                    ui.close_menu();
                }
                if ui.button(t.edit_select_all).clicked() {
                    app.select_all();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.edit_find).clicked() {
                    app.open_find(false);
                    ui.close_menu();
                }
                if ui.button(t.edit_replace).clicked() {
                    app.open_find(true);
                    ui.close_menu();
                }
                if ui.button(t.edit_goto_line).clicked() {
                    app.show_goto_line = true;
                    ui.close_menu();
                }
            });

            ui.menu_button(t.menu_view, |ui| {
                if ui.button(t.view_toggle_sidebar).clicked() {
                    app.show_sidebar = !app.show_sidebar;
                    ui.close_menu();
                }
                if ui.button(t.view_toggle_minimap).clicked() {
                    app.config.ui.show_minimap = !app.config.ui.show_minimap;
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button(t.view_language, |ui| {
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

                    if ui.selectable_label(is_auto, t.view_auto_detect).clicked() {
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
                ui.menu_button(t.view_font_size, |ui| {
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
                if ui.button(t.view_word_wrap).clicked() {
                    app.config.editor.word_wrap = !app.config.editor.word_wrap;
                    ui.close_menu();
                }
                if ui.button(t.view_line_numbers).clicked() {
                    app.config.editor.show_line_numbers = !app.config.editor.show_line_numbers;
                    ui.close_menu();
                }
            });

            ui.menu_button(t.menu_tools, |ui| {
                if ui.button(t.tools_compare).clicked() {
                    app.pending_compare_files = true;
                    ui.close_menu();
                }
                if ui.button(t.tools_macro).clicked() {
                    app.toggle_macro_recording();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.tools_preferences).clicked() {
                    app.show_preferences = true;
                    ui.close_menu();
                }
            });

            ui.menu_button(t.menu_help, |ui| {
                if ui.button(t.help_about).clicked() {
                    app.show_about = true;
                    ui.close_menu();
                }
            });
        });
    });
}
