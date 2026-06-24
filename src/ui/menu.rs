use eframe::egui;

use crate::app::RustpadApp;
#[cfg(not(target_os = "macos"))]
use crate::highlight::MENU_LANGUAGES;
#[cfg(not(target_os = "macos"))]
use crate::ui::encoding_menu::{self, EncodingMenuAction};
#[cfg(not(target_os = "macos"))]
use crate::ui::menu_actions::{self, MENU_FONT_SIZES};

/// Render the top menu bar (in-window on Windows/Linux; macOS uses the system menu bar).
#[cfg_attr(target_os = "macos", allow(unused_variables))]
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    #[cfg(not(target_os = "macos"))]
    show_in_window(app, ctx);
}

#[cfg(not(target_os = "macos"))]
fn show_in_window(app: &mut RustpadApp, ctx: &egui::Context) {
    let t = app.tr();
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(t.menu_file, |ui| {
                if ui.button(t.file_new).clicked() {
                    menu_actions::dispatch(app, "file.new", ctx);
                    ui.close_menu();
                }
                if ui.button(t.file_open).clicked() {
                    menu_actions::dispatch(app, "file.open", ctx);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.file_save).clicked() {
                    menu_actions::dispatch(app, "file.save", ctx);
                    ui.close_menu();
                }
                if ui.button(t.file_save_as).clicked() {
                    menu_actions::dispatch(app, "file.save_as", ctx);
                    ui.close_menu();
                }
                if ui.button(t.file_save_all).clicked() {
                    menu_actions::dispatch(app, "file.save_all", ctx);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.file_close_tab).clicked() {
                    menu_actions::dispatch(app, "file.close_tab", ctx);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.file_compare).clicked() {
                    menu_actions::dispatch(app, "file.compare", ctx);
                    ui.close_menu();
                }
                if ui.button(t.file_compare_current).clicked() {
                    menu_actions::dispatch(app, "file.compare_current", ctx);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.file_exit).clicked() {
                    menu_actions::dispatch(app, "file.exit", ctx);
                    ui.close_menu();
                }
            });

            ui.menu_button(t.menu_edit, |ui| {
                if ui.button(t.edit_undo).clicked() {
                    menu_actions::dispatch(app, "edit.undo", ctx);
                    ui.close_menu();
                }
                if ui.button(t.edit_redo).clicked() {
                    menu_actions::dispatch(app, "edit.redo", ctx);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.edit_cut).clicked() {
                    menu_actions::dispatch(app, "edit.cut", ctx);
                    ui.close_menu();
                }
                if ui.button(t.edit_copy).clicked() {
                    menu_actions::dispatch(app, "edit.copy", ctx);
                    ui.close_menu();
                }
                if ui.button(t.edit_paste).clicked() {
                    menu_actions::dispatch(app, "edit.paste", ctx);
                    ui.close_menu();
                }
                if ui.button(t.edit_select_all).clicked() {
                    menu_actions::dispatch(app, "edit.select_all", ctx);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.edit_find).clicked() {
                    menu_actions::dispatch(app, "edit.find", ctx);
                    ui.close_menu();
                }
                if ui.button(t.edit_replace).clicked() {
                    menu_actions::dispatch(app, "edit.replace", ctx);
                    ui.close_menu();
                }
                if ui.button(t.edit_goto_line).clicked() {
                    menu_actions::dispatch(app, "edit.goto_line", ctx);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t.edit_copy_column).clicked() {
                    menu_actions::dispatch(app, "edit.copy_column", ctx);
                    ui.close_menu();
                }
            });

            ui.menu_button(t.menu_view, |ui| {
                if ui.button(t.view_toggle_sidebar).clicked() {
                    menu_actions::dispatch(app, "view.sidebar", ctx);
                    ui.close_menu();
                }
                if ui.button(t.view_toggle_minimap).clicked() {
                    menu_actions::dispatch(app, "view.minimap", ctx);
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
                        menu_actions::dispatch(app, "view.lang.auto", ctx);
                        ui.close_menu();
                    }
                    ui.separator();
                    for (index, &lang) in MENU_LANGUAGES.iter().enumerate() {
                        if ui.selectable_label(current == lang, lang).clicked() {
                            menu_actions::dispatch(app, &format!("view.lang.{index}"), ctx);
                            ui.close_menu();
                        }
                    }
                });
                ui.separator();
                ui.menu_button(t.view_font_size, |ui| {
                    for size in MENU_FONT_SIZES {
                        if ui
                            .selectable_label(
                                app.config.editor.font_size == size as f32,
                                format!("{size}px"),
                            )
                            .clicked()
                        {
                            menu_actions::dispatch(app, &format!("view.font.{size}"), ctx);
                        }
                    }
                });
                ui.separator();
                if ui.button(t.view_word_wrap).clicked() {
                    menu_actions::dispatch(app, "view.word_wrap", ctx);
                    ui.close_menu();
                }
                if ui.button(t.view_line_numbers).clicked() {
                    menu_actions::dispatch(app, "view.line_numbers", ctx);
                    ui.close_menu();
                }
            });

            let mut enc_action = EncodingMenuAction::None;
            ui.menu_button(t.menu_encoding, |ui| {
                enc_action = encoding_menu::show_menu(ui, app);
            });
            menu_actions::apply_encoding_action(app, enc_action);

            ui.menu_button(t.menu_tools, |ui| {
                if ui.button(t.tools_compare).clicked() {
                    menu_actions::dispatch(app, "tools.compare", ctx);
                    ui.close_menu();
                }
                if ui.button(t.tools_macro).clicked() {
                    menu_actions::dispatch(app, "tools.macro", ctx);
                    ui.close_menu();
                }
            });

            ui.menu_button(t.menu_settings, |ui| {
                if ui.button(t.settings_preferences).clicked() {
                    menu_actions::dispatch(app, "settings.preferences", ctx);
                    ui.close_menu();
                }
                if ui.button(t.settings_keybindings).clicked() {
                    menu_actions::dispatch(app, "settings.keybindings", ctx);
                    ui.close_menu();
                }
            });

            ui.menu_button(t.menu_help, |ui| {
                if ui.button(t.help_about).clicked() {
                    menu_actions::dispatch(app, "help.about", ctx);
                    ui.close_menu();
                }
            });
        });
    });
}
