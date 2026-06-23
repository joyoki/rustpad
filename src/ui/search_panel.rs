use eframe::egui;

use crate::app::{RustpadApp, SearchDialogTab};
use crate::i18n::{self, UiLanguage};

/// Dockable "Search results" panel that lists every match with its line number
/// and lets the user click a row to jump to it (Notepad++ style).
pub fn show_results_panel(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_search_results {
        return;
    }

    let mut close = false;
    let mut jump_target: Option<usize> = None;

    let t = app.tr();
    let lang = app.config.ui.ui_language.clone();

    egui::TopBottomPanel::bottom("search_results_panel")
        .resizable(true)
        .default_height(180.0)
        .min_height(80.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.strong(t.search_results);
                ui.label(
                    egui::RichText::new(i18n::matches_count(&lang, app.search_result_items.len()))
                        .color(egui::Color32::from_gray(120)),
                );
                if !app.search_pattern.is_empty() {
                    ui.label(
                        egui::RichText::new(i18n::search_for(&lang, &app.search_pattern))
                            .italics()
                            .color(egui::Color32::from_gray(120)),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(format!("✖ {}", t.btn_close)).clicked() {
                        close = true;
                    }
                    if ui.button(t.btn_clear).clicked() {
                        app.search_result_items.clear();
                    }
                });
            });
            ui.separator();

            if app.search_result_items.is_empty() {
                ui.weak(t.no_search_results);
                return;
            }

            // Width reserved for the line-number gutter so previews line up.
            let line_no_width = 64.0;
            let current = app.search_engine.current_index();
            let multi_doc = app
                .search_result_items
                .iter()
                .any(|i| i.doc != app.search_result_items[0].doc);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut last_doc: Option<String> = None;
                    for (idx, item) in app.search_result_items.iter().enumerate() {
                        // Group header per document when searching multiple files.
                        if multi_doc && last_doc.as_deref() != Some(item.doc.as_str()) {
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(format!("▸ {}", item.doc))
                                    .strong()
                                    .color(egui::Color32::from_rgb(80, 120, 200)),
                            );
                            last_doc = Some(item.doc.clone());
                        }

                        let is_current = current == Some(idx)
                            && item.tab == app.tab_manager.active_index();
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [line_no_width, ui.available_height()],
                                egui::Label::new(
                                    egui::RichText::new(i18n::line_label(&lang, item.line + 1))
                                        .monospace()
                                        .color(egui::Color32::from_rgb(120, 140, 160)),
                                ),
                            );
                            let label = egui::RichText::new(&item.preview).monospace();
                            let label = if is_current { label.strong() } else { label };
                            if ui.selectable_label(is_current, label).clicked() {
                                jump_target = Some(idx);
                            }
                        });
                    }
                });
        });

    if let Some(idx) = jump_target {
        app.jump_to_result_item(idx);
    }
    if close {
        app.show_search_results = false;
    }
}

/// Floating find/replace dialog (Notepad-- style).
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_search {
        return;
    }

    let mut open = app.show_search;
    let mut request_close = false;
    let t = app.tr();
    let lang = app.config.ui.ui_language.clone();
    // Initial position only (no anchor) so the window stays draggable.
    let screen = ctx.input(|i| i.screen_rect());
    let default_pos = egui::pos2((screen.width() - 560.0).max(0.0) * 0.5, 80.0);
    egui::Window::new(t.find_replace_title)
        .id(egui::Id::new("find_replace_dialog"))
        .collapsible(false)
        .resizable(true)
        .movable(true)
        .default_size([560.0, 320.0])
        .default_pos(default_pos)
        .order(egui::Order::Foreground)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(app.search_dialog_tab == SearchDialogTab::Find, t.find_tab)
                    .clicked()
                {
                    app.search_dialog_tab = SearchDialogTab::Find;
                    app.search_replace_mode = false;
                    app.search_focus_input = true;
                }
                if ui
                    .selectable_label(
                        app.search_dialog_tab == SearchDialogTab::Replace,
                        t.replace_tab,
                    )
                    .clicked()
                {
                    app.search_dialog_tab = SearchDialogTab::Replace;
                    app.search_replace_mode = true;
                    app.search_focus_input = true;
                }
                if ui.button(t.dir_find).on_hover_text(t.find_in_files).clicked() {
                    request_close = true;
                    app.editor_has_focus = false;
                    app.show_cross_file_search = true;
                }
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(340.0);

                    ui.horizontal(|ui| {
                        ui.label(t.find_what);
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut app.search_pattern)
                                .id(egui::Id::new("find_what_input"))
                                .desired_width(220.0)
                                .hint_text("Search text…"),
                        );
                        if app.search_focus_input {
                            response.request_focus();
                            app.search_focus_input = false;
                        }
                        if response.changed() {
                            app.refresh_search_results(true);
                            let n = app.search_engine.results().len();
                            app.search_status_message = if n == 0 {
                                i18n::msg_no_matches(&lang)
                            } else {
                                i18n::msg_found_matches(&lang, n)
                            };
                            if n > 0 {
                                app.show_search_results = true;
                            }
                        }
                        if response.has_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            app.find_next_in_dialog();
                        }

                        // Single history dropdown (the ComboBox provides its own
                        // arrow, so we don't add another glyph).
                        egui::ComboBox::from_id_salt("find_history")
                            .width(0.0)
                            .selected_text("")
                            .show_ui(ui, |ui| {
                                ui.set_min_width(220.0);
                                if app.search_history.is_empty() {
                                    ui.weak(t.no_recent_searches);
                                } else {
                                    for item in app.search_history.clone() {
                                        if ui.selectable_label(false, &item).clicked() {
                                            app.search_pattern = item;
                                            app.refresh_search_results(true);
                                            if !app.search_engine.results().is_empty() {
                                                app.show_search_results = true;
                                            }
                                        }
                                    }
                                }
                            });
                    });

                    if app.search_dialog_tab == SearchDialogTab::Replace {
                        ui.horizontal(|ui| {
                            ui.label(t.replace_with);
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut app.replace_pattern)
                                    .id(egui::Id::new("replace_with_input"))
                                    .desired_width(250.0)
                                    .hint_text("Replacement…"),
                            );
                            if app.search_focus_replace {
                                response.request_focus();
                                app.search_focus_replace = false;
                            }
                            if response.has_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                app.replace_current_match();
                            }
                        });
                    }

                    ui.add_space(4.0);

                    let mut options_changed = false;
                    options_changed |= ui
                        .checkbox(&mut app.search_options.backward, t.opt_backward)
                        .changed();
                    options_changed |= ui
                        .checkbox(
                            &mut app.search_options.whole_word,
                            t.opt_whole_word,
                        )
                        .changed();
                    options_changed |= ui
                        .checkbox(&mut app.search_options.case_sensitive, t.opt_match_case)
                        .changed();
                    options_changed |= ui
                        .checkbox(&mut app.search_options.wrap_around, t.opt_wrap)
                        .changed();

                    ui.add_space(4.0);
                    ui.label(t.search_mode);
                    ui.horizontal(|ui| {
                        if ui
                            .radio(!app.search_options.use_regex, t.mode_normal)
                            .clicked()
                        {
                            app.search_options.use_regex = false;
                            options_changed = true;
                        }
                        if ui
                            .radio(app.search_options.use_regex, t.mode_regex)
                            .clicked()
                        {
                            app.search_options.use_regex = true;
                            options_changed = true;
                        }
                    });

                    if options_changed {
                        app.refresh_search_results(true);
                        let n = app.search_engine.results().len();
                        app.search_status_message = if n == 0 {
                            i18n::msg_no_matches(&lang)
                        } else {
                            i18n::msg_found_matches(&lang, n)
                        };
                        if n > 0 {
                            app.show_search_results = true;
                        }
                    }

                    if !app.search_status_message.is_empty() {
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(&app.search_status_message)
                                .italics()
                                .color(egui::Color32::from_gray(100)),
                        );
                    }
                });

                ui.separator();

                ui.vertical(|ui| {
                    // All action buttons share one fixed width for a tidy column.
                    const BTN_W: f32 = 210.0;
                    const BTN_H: f32 = 24.0;
                    ui.set_min_width(BTN_W);
                    let btn = |ui: &mut egui::Ui, text: &str| {
                        ui.add_sized([BTN_W, BTN_H], egui::Button::new(text))
                    };

                    if btn(ui, t.find_next).clicked() {
                        app.find_next_in_dialog();
                    }
                    if btn(ui, t.find_prev).clicked() {
                        app.find_prev_in_dialog();
                    }
                    if btn(ui, t.find_count).clicked() {
                        app.search_count_current();
                    }
                    if btn(ui, t.find_all_current).clicked() {
                        app.find_all_in_current_document();
                    }
                    if btn(ui, t.find_all_open).clicked() {
                        app.find_all_in_open_documents();
                    }
                    if btn(ui, t.clear_result).clicked() {
                        app.clear_search_status();
                    }
                    ui.separator();
                    if app.search_dialog_tab == SearchDialogTab::Replace {
                        if btn(ui, t.replace_one).clicked() {
                            app.replace_current_match();
                        }
                        if btn(ui, t.replace_all).clicked() {
                            let n = app.search_engine.results().len();
                            app.replace_all_matches();
                            app.search_status_message = i18n::msg_replaced(&lang, n);
                        }
                    }
                    if btn(ui, t.btn_close).clicked() {
                        request_close = true;
                    }
                });
            });
        });
    if request_close {
        open = false;
    }
    app.show_search = open;
}

/// Directory search window (Ctrl+Shift+F).
pub fn show_cross_file_search(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_cross_file_search {
        return;
    }

    let t = app.tr();
    let lang = app.config.ui.ui_language.clone();
    let mut open = app.show_cross_file_search;
    let mut request_close = false;
    let lbl_search = if UiLanguage::parse(&lang) == UiLanguage::Zh {
        "搜索："
    } else {
        "Search:"
    };
    let lbl_dir = if UiLanguage::parse(&lang) == UiLanguage::Zh {
        "目录："
    } else {
        "Directory:"
    };
    let lbl_filter = if UiLanguage::parse(&lang) == UiLanguage::Zh {
        "过滤："
    } else {
        "Filter:"
    };
    let lbl_browse = if UiLanguage::parse(&lang) == UiLanguage::Zh {
        "浏览..."
    } else {
        "Browse..."
    };
    let lbl_filter_hint = if UiLanguage::parse(&lang) == UiLanguage::Zh {
        "（例如 *.rs, *.txt）"
    } else {
        "(e.g. *.rs, *.txt)"
    };
    let lbl_search_btn = if UiLanguage::parse(&lang) == UiLanguage::Zh {
        "搜索"
    } else {
        "Search"
    };
    egui::Window::new(t.find_in_files)
        .id(egui::Id::new("find_in_files_dialog"))
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .default_height(400.0)
        .order(egui::Order::Foreground)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(lbl_search);
                ui.add(
                    egui::TextEdit::singleline(&mut app.search_pattern)
                        .id(egui::Id::new("dir_find_input"))
                        .desired_width(280.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label(lbl_dir);
                ui.text_edit_singleline(&mut app.cross_file_directory);
                if ui.button(lbl_browse).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        app.cross_file_directory = path.to_string_lossy().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label(lbl_filter);
                ui.text_edit_singleline(&mut app.cross_file_filter);
                ui.label(lbl_filter_hint);
            });

            ui.horizontal(|ui| {
                if ui.button(lbl_search_btn).clicked() {
                    app.perform_cross_file_search();
                }
                if ui.button(t.btn_close).clicked() {
                    request_close = true;
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (path, line, text) in &app.cross_file_results {
                    let label = format!("{}:{}: {}", path.display(), line + 1, text.trim());
                    if ui.button(&label).clicked()
                        && app.tab_manager.open_file(&path.clone()).is_ok()
                    {
                        let tab = app.tab_manager.active_mut();
                        tab.cursor.line = *line;
                        tab.cursor.col = 0;
                    }
                }
            });
        });
    if request_close {
        open = false;
    }
    app.show_cross_file_search = open;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_search_panel_compiles() {
        assert!(true);
    }
}
