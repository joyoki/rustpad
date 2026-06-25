use eframe::egui;

use crate::app::{RustpadApp, SearchDialogTab, SearchResultItem};
use crate::i18n::{self, UiLanguage};

/// Dockable "Search results" panel — accumulates every "Find All" session (Notepad++ style).
pub fn show_results_panel(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_search_results {
        return;
    }

    let mut close = false;
    let mut jump_target: Option<(usize, usize)> = None;
    let mut toggle_batch: Option<usize> = None;
    let mut toggle_doc: Option<(usize, String)> = None;
    let mut collapse_all = false;
    let mut expand_all = false;

    let t = app.tr();
    let lang = app.config.ui.ui_language.clone();
    let batch_count = app.search_result_batches.len();
    let total_matches = app.search_result_total_matches();

    egui::TopBottomPanel::bottom("search_results_panel")
        .resizable(true)
        .default_height(180.0)
        .min_height(80.0)
        .show(ctx, |ui| {
            let scroll_id = ui.id().with("search_results_scroll_y");
            let mut scroll_y = ui.data(|d| d.get_temp::<f32>(scroll_id)).unwrap_or(0.0);

            ui.horizontal(|ui| {
                ui.strong(t.search_results);
                ui.label(
                    egui::RichText::new(i18n::matches_count(&lang, total_matches))
                        .color(egui::Color32::from_gray(120)),
                );
                if batch_count > 1 {
                    let sessions_lbl = if UiLanguage::parse(&lang) == UiLanguage::Zh {
                        format!("{batch_count} 次搜索")
                    } else {
                        format!("{batch_count} searches")
                    };
                    ui.label(
                        egui::RichText::new(sessions_lbl).color(egui::Color32::from_gray(120)),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(format!("✖ {}", t.btn_close)).clicked() {
                        close = true;
                    }
                    if ui.button(t.btn_clear).clicked() {
                        app.search_result_batches.clear();
                        app.active_result_batch_id = None;
                        scroll_y = 0.0;
                    }
                });
            });
            ui.separator();

            if app.search_result_batches.is_empty() {
                ui.weak(t.no_search_results);
                ui.data_mut(|d| d.insert_temp(scroll_id, scroll_y));
                return;
            }

            let line_no_width = 64.0;
            let has_file_groups = app.search_result_batches.iter().any(|batch| {
                batch.items.first().is_some_and(|first| {
                    batch.items.iter().any(|i| i.doc != first.doc)
                })
            });

            if has_file_groups || batch_count > 1 {
                ui.horizontal(|ui| {
                    let (expand_lbl, collapse_lbl) =
                        if UiLanguage::parse(&lang) == UiLanguage::Zh {
                            ("全部展开", "全部折叠")
                        } else {
                            ("Expand All", "Collapse All")
                        };
                    if ui.small_button(expand_lbl).clicked() {
                        expand_all = true;
                    }
                    if ui.small_button(collapse_lbl).clicked() {
                        collapse_all = true;
                    }
                });
            }

            let scroll = egui::ScrollArea::vertical()
                .id_salt("search_results_list")
                .auto_shrink([false, false])
                .drag_to_scroll(false)
                .animated(false)
                .vertical_scroll_offset(scroll_y)
                .show(ui, |ui| {
                    let batch_ids: Vec<usize> =
                        app.search_result_batches.iter().map(|b| b.id).collect();
                    for batch_id in batch_ids {
                        let Some(batch) = app
                            .search_result_batches
                            .iter()
                            .find(|b| b.id == batch_id)
                        else {
                            continue;
                        };
                        let batch_icon = if batch.collapsed { "▶" } else { "▼" };
                        let batch_header = format!(
                            "{} \"{}\" — {} ({})",
                            batch_icon,
                            batch.pattern,
                            batch.scope_label,
                            batch.items.len()
                        );
                        let batch_response = ui
                            .add(
                                egui::Label::new(
                                    egui::RichText::new(batch_header)
                                        .strong()
                                        .color(egui::Color32::from_rgb(60, 100, 160)),
                                )
                                .sense(egui::Sense::click()),
                            )
                            .on_hover_text(
                                if UiLanguage::parse(&lang) == UiLanguage::Zh {
                                    "点击折叠/展开此搜索"
                                } else {
                                    "Click to expand/collapse this search"
                                },
                            );
                        if batch_response.clicked() {
                            toggle_batch = Some(batch_id);
                        }

                        if batch.collapsed {
                            continue;
                        }

                        let multi_doc = batch.items.first().is_some_and(|first| {
                            batch.items.iter().any(|i| i.doc != first.doc)
                        });

                        let mut doc_groups: Vec<(String, Vec<usize>)> = Vec::new();
                        for (item_idx, item) in batch.items.iter().enumerate() {
                            if let Some((doc, entries)) = doc_groups.last_mut() {
                                if *doc == item.doc {
                                    entries.push(item_idx);
                                    continue;
                                }
                            }
                            doc_groups.push((item.doc.clone(), vec![item_idx]));
                        }

                        ui.indent(ui.id().with(("search_batch", batch_id)), |ui| {
                            for (doc, entries) in doc_groups {
                                if multi_doc {
                                    let expanded =
                                        !batch.collapsed_docs.contains(&doc);
                                    let icon = if expanded { "▼" } else { "▶" };
                                    let header =
                                        format!("{} {} ({})", icon, doc, entries.len());
                                    let header_response = ui
                                        .add(
                                            egui::Label::new(
                                                egui::RichText::new(header).strong().color(
                                                    egui::Color32::from_rgb(80, 120, 200),
                                                ),
                                            )
                                            .sense(egui::Sense::click()),
                                        )
                                        .on_hover_text(
                                            if UiLanguage::parse(&lang) == UiLanguage::Zh {
                                                "点击折叠/展开"
                                            } else {
                                                "Click to expand/collapse"
                                            },
                                        );
                                    if header_response.clicked() {
                                        toggle_doc = Some((batch_id, doc.clone()));
                                    }
                                    if !expanded {
                                        continue;
                                    }
                                    ui.indent(
                                        ui.id().with(("search_doc", batch_id, &doc)),
                                        |ui| {
                                            for item_idx in entries {
                                                if let Some(item) =
                                                    batch.items.get(item_idx)
                                                {
                                                    paint_search_result_row(
                                                        ui,
                                                        app,
                                                        batch_id,
                                                        item_idx,
                                                        item,
                                                        line_no_width,
                                                        &lang,
                                                        &mut jump_target,
                                                    );
                                                }
                                            }
                                        },
                                    );
                                } else {
                                    for item_idx in entries {
                                        if let Some(item) = batch.items.get(item_idx) {
                                            paint_search_result_row(
                                                ui,
                                                app,
                                                batch_id,
                                                item_idx,
                                                item,
                                                line_no_width,
                                                &lang,
                                                &mut jump_target,
                                            );
                                        }
                                    }
                                }
                            }
                        });
                    }
                });

            scroll_y = scroll.state.offset.y;

            // Mouse wheels often emit raw_scroll_delta; egui ScrollArea only reads smooth delta.
            let pointer = ui.input(|i| i.pointer.hover_pos());
            if pointer.is_some_and(|p| scroll.inner_rect.contains(p)) {
                let raw = ui.input(|i| i.raw_scroll_delta.y);
                if raw != 0.0 {
                    let max_y = (scroll.content_size.y - scroll.inner_rect.height()).max(0.0);
                    scroll_y = (scroll_y - raw).clamp(0.0, max_y);
                    ui.ctx().input_mut(|i| i.raw_scroll_delta.y = 0.0);
                }
            }

            ui.data_mut(|d| d.insert_temp(scroll_id, scroll_y));

            // Keep wheel events in the results panel when the pointer is over it.
            if pointer.is_some_and(|p| scroll.inner_rect.contains(p)) {
                let has_wheel = ui.input(|i| {
                    i.smooth_scroll_delta.y != 0.0 || i.raw_scroll_delta.y != 0.0
                });
                if has_wheel {
                    crate::ui::scroll_bar::consume_editor_wheel(ctx, true);
                }
            }
        });

    if expand_all {
        for batch in &mut app.search_result_batches {
            batch.collapsed = false;
            batch.collapsed_docs.clear();
        }
    }
    if collapse_all {
        for batch in &mut app.search_result_batches {
            batch.collapsed = false;
            batch.collapsed_docs = batch
                .items
                .iter()
                .map(|i| i.doc.clone())
                .collect();
        }
    }
    if let Some(batch_id) = toggle_batch {
        if let Some(batch) = app
            .search_result_batches
            .iter_mut()
            .find(|b| b.id == batch_id)
        {
            batch.collapsed = !batch.collapsed;
        }
    }
    if let Some((batch_id, doc)) = toggle_doc {
        if let Some(batch) = app
            .search_result_batches
            .iter_mut()
            .find(|b| b.id == batch_id)
        {
            if batch.collapsed_docs.contains(&doc) {
                batch.collapsed_docs.remove(&doc);
            } else {
                batch.collapsed_docs.insert(doc);
            }
        }
    }

    if let Some((batch_id, item_idx)) = jump_target {
        app.jump_to_batch_item(batch_id, item_idx);
        ctx.request_repaint();
    }
    if close {
        app.show_search_results = false;
    }
}

fn paint_search_result_row(
    ui: &mut egui::Ui,
    app: &RustpadApp,
    batch_id: usize,
    item_idx: usize,
    item: &SearchResultItem,
    line_no_width: f32,
    lang: &str,
    jump_target: &mut Option<(usize, usize)>,
) {
    let is_current = result_item_is_current(app, batch_id, item);
    let match_len = item.end.saturating_sub(item.start);
    let theme = app.theme_manager.current_theme().clone();
    let row_width = ui.available_width();
    let row_height = ui.spacing().interact_size.y;
    let (row_rect, row_response) =
        ui.allocate_exact_size(egui::vec2(row_width, row_height), egui::Sense::click());

    if ui.is_rect_visible(row_rect) {
        let painter = ui.painter().with_clip_rect(ui.clip_rect());
        if is_current {
            painter.rect_filled(
                row_rect,
                0.0,
                theme.search_highlight_bg_color().gamma_multiply(0.35),
            );
        } else if row_response.hovered() {
            painter.rect_filled(
                row_rect,
                0.0,
                ui.visuals().widgets.hovered.weak_bg_fill,
            );
        }

        let line_label = i18n::line_label(lang, item.line + 1);
        let line_font = egui::TextStyle::Monospace.resolve(ui.style());
        let line_galley = ui.fonts(|f| {
            f.layout_no_wrap(
                line_label,
                line_font,
                egui::Color32::from_rgb(120, 140, 160),
            )
        });
        let line_y = row_rect.center().y - line_galley.size().y * 0.5;
        painter.galley(
            egui::pos2(row_rect.min.x, line_y),
            line_galley,
            egui::Color32::from_rgb(120, 140, 160),
        );

        let preview_rect = egui::Rect::from_min_max(
            egui::pos2(row_rect.min.x + line_no_width, row_rect.min.y),
            row_rect.max,
        );
        crate::ui::search_highlight::paint_result_preview_painter(
            ui,
            preview_rect,
            &item.preview,
            item.col,
            match_len,
            is_current,
            &theme,
        );
    }

    if row_response.double_clicked() || row_response.clicked() {
        *jump_target = Some((batch_id, item_idx));
    }
}

fn result_item_is_current(
    app: &RustpadApp,
    batch_id: usize,
    item: &SearchResultItem,
) -> bool {
    if app.active_result_batch_id != Some(batch_id) {
        return false;
    }
    if item.tab != app.tab_manager.active_index() {
        return false;
    }
    let Some(ci) = app.search_engine.current_index() else {
        return false;
    };
    app.search_engine
        .results()
        .get(ci)
        .is_some_and(|m| m.start == item.start && m.line == item.line)
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
                        let theme = app.theme_manager.current_theme();
                        let status_color = if app.search_engine.results().is_empty() {
                            egui::Color32::from_gray(100)
                        } else {
                            theme.search_highlight_bg_color()
                        };
                        let status_text = egui::RichText::new(&app.search_status_message)
                            .italics()
                            .background_color(status_color)
                            .color(egui::Color32::BLACK);
                        ui.label(status_text);
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
