use eframe::egui;
use eframe::epaint::Color32;

use crate::app::RustpadApp;

/// Show non-blocking dialogs (preferences, about, goto line).
pub fn show_non_blocking(app: &mut RustpadApp, ctx: &egui::Context) {
    show_goto_line_dialog(app, ctx);
    show_about_dialog(app, ctx);
    show_preferences_dialog(app, ctx);
}

fn modal_backdrop(ctx: &egui::Context) {
    let screen = ctx.input(|i| i.screen_rect());
    // A very light scrim that hints at modality without darkening the UI, so the
    // window keeps its bright styling.
    ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("modal_backdrop"),
    ))
    .rect_filled(screen, 0.0, Color32::from_white_alpha(40));
}

fn show_goto_line_dialog(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_goto_line {
        return;
    }

    let mut open = app.show_goto_line;
    egui::Window::new("Go to Line")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Line number:");
                let response = ui.text_edit_singleline(&mut app.goto_line_text);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(line) = app.goto_line_text.parse::<usize>() {
                        if line > 0 {
                            let max_line = app.tab_manager.active().line_count();
                            let target = (line - 1).min(max_line.saturating_sub(1));
                            let tab = app.tab_manager.active_mut();
                            tab.cursor.line = target;
                            tab.cursor.col = 0;
                            tab.scroll_offset = target as f32 * 20.0;
                        }
                    }
                    app.show_goto_line = false;
                    app.goto_line_text.clear();
                }
            });
        });
    app.show_goto_line = open;
}

fn show_about_dialog(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_about {
        return;
    }

    let mut open = app.show_about;
    egui::Window::new("About Rustpad")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Rustpad");
                ui.label("A modern code editor built with Rust");
                ui.label("Version 0.1.0");
                ui.separator();
                ui.label("Powered by egui + eframe");
                ui.label("Syntax highlighting by syntect");
                ui.label("Diff engine by similar");
            });
        });
    app.show_about = open;
}

fn show_preferences_dialog(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_preferences {
        return;
    }

    let mut open = app.show_preferences;
    egui::Window::new("Preferences")
        .default_size([400.0, 300.0])
        .open(&mut open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Editor");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Font Size:");
                    ui.add(egui::Slider::new(
                        &mut app.config.editor.font_size,
                        8.0..=72.0,
                    ));
                });

                ui.horizontal(|ui| {
                    ui.label("Tab Size:");
                    ui.add(egui::Slider::new(&mut app.config.editor.tab_size, 1..=8));
                });

                ui.checkbox(
                    &mut app.config.editor.show_line_numbers,
                    "Show line numbers",
                );
                ui.checkbox(&mut app.config.editor.word_wrap, "Word wrap");
                ui.checkbox(&mut app.config.editor.auto_indent, "Auto indent");
                ui.checkbox(
                    &mut app.config.editor.highlight_current_line,
                    "Highlight current line",
                );

                ui.separator();
                ui.heading("UI");
                ui.separator();

                ui.checkbox(&mut app.config.ui.show_minimap, "Show minimap");

                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    let prev_theme = app.config.ui.theme.clone();
                    egui::ComboBox::from_id_salt("theme_select")
                        .selected_text(&app.config.ui.theme)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.config.ui.theme,
                                "Light".to_string(),
                                "Light",
                            );
                            ui.selectable_value(
                                &mut app.config.ui.theme,
                                "Dark".to_string(),
                                "Dark",
                            );
                        });
                    if app.config.ui.theme != prev_theme {
                        app.on_theme_changed(ctx);
                    }
                });

                ui.separator();

                if ui.button("Save Preferences").clicked() {
                    app.on_theme_changed(ctx);
                    let _ = app.config.save();
                    app.show_preferences = false;
                }
            });
        });
    app.show_preferences = open;
}

pub fn show_unsaved_dialog(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_unsaved_dialog {
        return;
    }

    modal_backdrop(ctx);

    let file_label = app
        .pending_close_tab
        .and_then(|idx| app.tab_manager.tabs().get(idx))
        .map(|tab| tab.display_title())
        .unwrap_or_else(|| "Current file".to_string());

    egui::Window::new("Unsaved Changes")
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "\"{file_label}\" has unsaved changes. What would you like to do?"
            ));
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    if let Some(idx) = app.pending_close_tab {
                        let needs_save_as = app.tab_manager.tabs()[idx].file_path.is_none();
                        if needs_save_as {
                            app.tab_manager.set_active(idx);
                            app.save_as_dialog();
                        } else {
                            let _ = app.tab_manager.tabs_mut()[idx].save();
                        }
                        if !app.tab_manager.tabs()[idx].buffer.is_dirty()
                            && !app.tab_manager.tabs()[idx].modified
                        {
                            app.tab_manager.close_tab(idx);
                            app.pending_close_tab = None;
                            app.show_unsaved_dialog = false;
                        }
                    }
                }
                if ui.button("Don't Save").clicked() {
                    if let Some(idx) = app.pending_close_tab {
                        app.tab_manager.close_tab(idx);
                    }
                    app.pending_close_tab = None;
                    app.show_unsaved_dialog = false;
                }
                if ui.button("Cancel").clicked() {
                    app.pending_close_tab = None;
                    app.show_unsaved_dialog = false;
                }
            });
        });
}

pub fn show_quit_unsaved_dialog(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_quit_unsaved_dialog {
        return;
    }

    modal_backdrop(ctx);

    let unsaved_count = app.tab_manager.unsaved_tab_indices().len();
    egui::Window::new("Save Changes?")
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "You have {unsaved_count} file(s) with unsaved changes. Save before closing RustPad?"
            ));
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Save All").clicked() {
                    let _ = app.confirm_quit_after_save(ctx);
                }
                if ui.button("Don't Save").clicked() {
                    app.confirm_quit_without_save(ctx);
                }
                if ui.button("Cancel").clicked() {
                    app.show_quit_unsaved_dialog = false;
                }
            });
        });
}
