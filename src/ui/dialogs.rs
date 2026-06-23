use eframe::egui;
use eframe::epaint::Color32;

use crate::app::RustpadApp;
use crate::editor::EncodingProfile;
use crate::i18n::{self, UiLanguage};

/// Show non-blocking dialogs (preferences, about, goto line).
pub fn show_non_blocking(app: &mut RustpadApp, ctx: &egui::Context) {
    show_goto_line_dialog(app, ctx);
    show_batch_encoding_dialog(app, ctx);
    crate::ui::keybindings_dialog::show(app, ctx);
    show_about_dialog(app, ctx);
    show_preferences_dialog(app, ctx);
    show_html_preview_dialog(app, ctx);
}

fn modal_backdrop(ctx: &egui::Context) {
    let screen = ctx.input(|i| i.screen_rect());
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

    let t = app.tr();
    let mut open = app.show_goto_line;
    egui::Window::new(t.dlg_goto_line)
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(t.goto_line_number);
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

fn show_batch_encoding_dialog(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_batch_encoding {
        return;
    }

    let t = app.tr();
    let mut open = app.show_batch_encoding;
    let mut chosen: Option<EncodingProfile> = None;
    let mut should_close = false;

    egui::Window::new(t.dlg_batch_encoding)
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(t.enc_batch_prompt);
            ui.separator();
            for profile in EncodingProfile::MAIN
                .iter()
                .chain(EncodingProfile::MORE.iter())
            {
                if ui.button(profile.display_name()).clicked() {
                    chosen = Some(*profile);
                    should_close = true;
                }
            }
            ui.separator();
            if ui.button(t.btn_cancel).clicked() {
                should_close = true;
            }
        });

    if should_close {
        open = false;
    }

    app.show_batch_encoding = open;
    if let Some(profile) = chosen {
        app.batch_convert_encoding(profile);
    }
}

fn show_about_dialog(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_about {
        return;
    }

    let t = app.tr();
    let mut open = app.show_about;
    egui::Window::new(t.dlg_about)
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("RustPad");
                ui.label(t.about_tagline);
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.horizontal(|ui| {
                    ui.label(t.about_project);
                    ui.hyperlink_to(
                        "https://github.com/joyoki/rustpad",
                        "https://github.com/joyoki/rustpad",
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(t.about_releases);
                    ui.hyperlink_to(
                        "https://github.com/joyoki/rustpad/releases",
                        "https://github.com/joyoki/rustpad/releases",
                    );
                });
                ui.separator();
                ui.label(t.about_powered);
                ui.label(t.about_syntax);
            });
        });
    app.show_about = open;
}

fn show_preferences_dialog(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_preferences {
        return;
    }

    let t = app.tr();
    let mut open = app.show_preferences;
    egui::Window::new(t.dlg_preferences)
        .default_size([400.0, 360.0])
        .open(&mut open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(t.pref_editor);
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(t.pref_font_size);
                    ui.add(egui::Slider::new(
                        &mut app.config.editor.font_size,
                        8.0..=72.0,
                    ));
                });

                ui.horizontal(|ui| {
                    ui.label(t.pref_tab_size);
                    ui.add(egui::Slider::new(
                        &mut app.config.editor.tab_size,
                        1..=8,
                    ));
                });

                ui.checkbox(&mut app.config.editor.show_line_numbers, t.pref_show_line_numbers);
                ui.checkbox(&mut app.config.editor.word_wrap, t.pref_word_wrap);
                ui.checkbox(&mut app.config.editor.auto_indent, t.pref_auto_indent);
                ui.checkbox(
                    &mut app.config.editor.highlight_current_line,
                    t.pref_highlight_line,
                );

                ui.separator();
                ui.heading(t.pref_ui);
                ui.separator();

                ui.checkbox(&mut app.config.ui.show_minimap, t.pref_show_minimap);

                ui.horizontal(|ui| {
                    ui.label(t.pref_theme);
                    let prev_theme = app.config.ui.theme.clone();
                    egui::ComboBox::from_id_salt("theme_select")
                        .selected_text(&app.config.ui.theme)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.config.ui.theme,
                                "Light".to_string(),
                                t.theme_light,
                            );
                            ui.selectable_value(
                                &mut app.config.ui.theme,
                                "Dark".to_string(),
                                t.theme_dark,
                            );
                        });
                    if app.config.ui.theme != prev_theme {
                        app.on_theme_changed(ctx);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(t.pref_ui_language);
                    let prev_lang = app.config.ui.ui_language.clone();
                    egui::ComboBox::from_id_salt("ui_language_select")
                        .selected_text(UiLanguage::parse(&app.config.ui.ui_language).display_name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.config.ui.ui_language,
                                "en".to_string(),
                                "English",
                            );
                            ui.selectable_value(
                                &mut app.config.ui.ui_language,
                                "zh".to_string(),
                                "中文",
                            );
                        });
                    if app.config.ui.ui_language != prev_lang {
                        app.on_language_changed(ctx);
                    }
                });

                ui.separator();

                if ui.button(t.pref_save).clicked() {
                    app.on_theme_changed(ctx);
                    app.on_language_changed(ctx);
                    let _ = app.config.save();
                    app.show_preferences = false;
                }
            });
        });
    app.show_preferences = open;
}

fn show_html_preview_dialog(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_html_preview {
        return;
    }
    let mut open = app.show_html_preview;
    let title = app.html_preview_title.clone();
    let save_label = if crate::i18n::UiLanguage::parse(&app.config.ui.ui_language)
        == crate::i18n::UiLanguage::Zh
    {
        "保存 HTML..."
    } else {
        "Save HTML..."
    };
    egui::Window::new(&title)
        .default_size([640.0, 480.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button(save_label).clicked() {
                    app.save_html_preview();
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(
                    egui::RichText::new(&app.html_preview_content)
                        .monospace()
                        .size(11.0),
                );
            });
        });
    app.show_html_preview = open;
}

pub fn show_unsaved_dialog(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_unsaved_dialog {
        return;
    }

    let t = app.tr();
    let lang = app.config.ui.ui_language.clone();
    modal_backdrop(ctx);

    let file_label = app
        .pending_close_tab
        .and_then(|idx| app.tab_manager.tabs().get(idx))
        .map(|tab| tab.display_title())
        .unwrap_or_else(|| {
            if UiLanguage::parse(&lang) == UiLanguage::Zh {
                "当前文件".to_string()
            } else {
                "Current file".to_string()
            }
        });

    egui::Window::new(t.dlg_unsaved)
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(i18n::unsaved_message(&lang, &file_label));
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button(t.btn_save).clicked() {
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
                if ui.button(t.btn_dont_save).clicked() {
                    if let Some(idx) = app.pending_close_tab {
                        app.tab_manager.close_tab(idx);
                    }
                    app.pending_close_tab = None;
                    app.show_unsaved_dialog = false;
                }
                if ui.button(t.btn_cancel).clicked() {
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

    let t = app.tr();
    let lang = app.config.ui.ui_language.clone();
    modal_backdrop(ctx);

    let unsaved_count = app.tab_manager.unsaved_tab_indices().len();
    egui::Window::new(t.dlg_quit_save)
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(i18n::quit_unsaved_message(&lang, unsaved_count));
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button(t.btn_save_all).clicked() {
                    let _ = app.confirm_quit_after_save(ctx);
                }
                if ui.button(t.btn_dont_save).clicked() {
                    app.confirm_quit_without_save(ctx);
                }
                if ui.button(t.btn_cancel).clicked() {
                    app.show_quit_unsaved_dialog = false;
                }
            });
        });
}
