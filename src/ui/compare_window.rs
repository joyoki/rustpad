//! notepad-- style unified compare window chrome (toolbar, path inputs, encoding bar).

use std::path::PathBuf;

use eframe::egui;

use crate::app::RustpadApp;
use crate::diff::{DiffAlgorithm, FolderDiffFilter, SyncDirection, SyncScope};
use crate::ui::binary_diff_view;
use crate::ui::compare_session::{CompareIntent, CompareMode, CompareSession, SidePreviewKind};
use crate::ui::diff_view;
use crate::ui::folder_diff_view;

const PATH_ROW_H: f32 = 28.0;
const FOOTER_TOP_GAP: f32 = 6.0;
const FOOTER_SEPARATOR_H: f32 = 2.0;
const ENCODING_ROW_H: f32 = 36.0;
const STATS_ROW_H: f32 = 26.0;
const TOOLBAR_ICON_W: f32 = 36.0;
/// Width reserved for the vertical separator between left/right columns.
const COLUMN_SEPARATOR_W: f32 = 10.0;

/// Half width for left/right compare columns (path, editor, status, encoding).
pub fn compare_half_width(total_width: f32) -> f32 {
    ((total_width - COLUMN_SEPARATOR_W) * 0.5).max(80.0)
}

fn show_two_columns(
    ui: &mut egui::Ui,
    mut draw: impl FnMut(&mut egui::Ui, f32, bool),
) {
    let half = compare_half_width(ui.available_width());
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(half, ui.available_height()),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| draw(ui, half, true),
        );
        ui.separator();
        ui.allocate_ui_with_layout(
            egui::vec2(half, ui.available_height()),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| draw(ui, half, false),
        );
    });
}

/// Fixed window title per notepad-- (对比文件 / 对比文件夹).
pub fn window_title(app: &RustpadApp, session: &CompareSession) -> String {
    let t = app.tr();
    match session.compare_intent {
        CompareIntent::File => t.cmp_window_title_file.to_string(),
        CompareIntent::Folder => t.cmp_window_title_folder.to_string(),
    }
}

/// Minimum height reserved for the compare window footer.
fn footer_min_height(text_compare: bool) -> f32 {
    let mut h = FOOTER_TOP_GAP + FOOTER_SEPARATOR_H + ENCODING_ROW_H;
    if text_compare {
        h += STATS_ROW_H;
    }
    h
}

pub fn show(app: &RustpadApp, session: &mut CompareSession, ctx: &egui::Context) {
    let t = app.tr();
    let sid = session.id;
    session.reset_drop_tracking();
    let files_dragged = ctx.input(|i| !i.raw.hovered_files.is_empty());
    let pointer = ctx.pointer_latest_pos();

    egui::TopBottomPanel::top(egui::Id::new(("cmp_toolbar", sid))).show(ctx, |ui| {
        show_notepad_toolbar(ui, session, t);
    });

    let text_compare = matches!(session.mode, CompareMode::Text);
    egui::TopBottomPanel::bottom(egui::Id::new(("cmp_footer", sid)))
        .min_height(footer_min_height(text_compare))
        .show(ctx, |ui| {
        // Gap + rule between editor and footer so encoding controls are not clipped.
        ui.add_space(FOOTER_TOP_GAP);
        ui.separator();
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ENCODING_ROW_H),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                show_encoding_columns(ui, session, t, ENCODING_ROW_H);
            },
        );
        if text_compare {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), STATS_ROW_H),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    diff_view::show_text_stats(ui, session, session.text_result.as_ref());
                },
            );
        }
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        if !session.status_message.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(180, 60, 60), &session.status_message);
            ui.separator();
        }
        if session.same_path_compare {
            ui.colored_label(
                egui::Color32::from_rgb(80, 120, 160),
                t.cmp_same_path_info,
            );
            ui.separator();
        }
        if !session.folder_sync_message.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(40, 120, 40), &session.folder_sync_message);
            ui.separator();
        }

        show_path_columns(ui, session, t, files_dragged);
        ui.separator();

        let editor_area = ui.available_rect_before_wrap();
        let editor_rect = editor_area;
        let mid_x = editor_rect.center().x;
        let left_rect = egui::Rect::from_min_max(
            editor_rect.min,
            egui::pos2(mid_x, editor_rect.max.y),
        );
        let right_rect = egui::Rect::from_min_max(
            egui::pos2(mid_x, editor_rect.top()),
            editor_rect.max,
        );
        session.register_drop_rect(true, left_rect);
        session.register_drop_rect(false, right_rect);

        if files_dragged {
            session.set_drop_target_from_pointer(pointer);
            paint_drop_halves(
                ui,
                left_rect,
                right_rect,
                session.drop_target_left,
                session.drop_target_right,
            );
        }

        match session.mode {
            CompareMode::Text => {
                let h = editor_rect.height().max(120.0);
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(editor_rect), |ui| {
                    ui.set_clip_rect(editor_rect);
                    diff_view::show_text_content(app, session, ui, h);
                });
            }
            CompareMode::Binary => binary_diff_view::show_binary_content(app, session, ui),
            CompareMode::Folder => folder_diff_view::show_folder_content(app, session, ui),
            CompareMode::None => show_column_previews(ui, session, t, editor_rect.height()),
        }
    });

    handle_drops(session, ctx);
}

fn show_notepad_toolbar(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    t: &crate::i18n::Locale,
) {
    let can_nav = matches!(session.mode, CompareMode::Text | CompareMode::Binary)
        && session_has_changes(session);
    let can_undo = session.can_undo_edit();

    ui.horizontal(|ui| {
        if toolbar_toggle_btn(ui, "␣", t.cmp_blank, t.cmp_blank_tip, session.show_whitespace)
            .clicked()
        {
            session.show_whitespace = !session.show_whitespace;
            if session.show_whitespace {
                session.ignore_whitespace = false;
                session.strict_mode = false;
            }
            session.recompute_active();
            ui.ctx().request_repaint();
        }

        let rules_label = format!("⚙ {}", session.algorithm_label());
        ui.menu_button(egui::RichText::new(rules_label).size(13.0), |ui| {
            ui.label(t.cmp_rules);
            ui.separator();
            if ui
                .selectable_value(&mut session.algorithm, DiffAlgorithm::Myers, t.diff_algo_myers)
                .changed()
            {
                session.recompute_active();
            }
            if ui
                .selectable_value(
                    &mut session.algorithm,
                    DiffAlgorithm::Patience,
                    t.diff_algo_patience,
                )
                .changed()
            {
                session.recompute_active();
            }
            if ui
                .selectable_value(&mut session.algorithm, DiffAlgorithm::Lcs, t.diff_algo_lcs)
                .changed()
            {
                session.recompute_active();
            }
            ui.separator();
            if ui.checkbox(&mut session.ignore_whitespace, t.cmp_ignore_ws).changed() {
                session.strict_mode = false;
                session.recompute_active();
            }
            if ui.checkbox(&mut session.ignore_case, t.cmp_ignore).changed() {
                session.strict_mode = false;
                session.recompute_active();
            }
        })
        .response
        .on_hover_text(t.cmp_rules);

        if toolbar_toggle_btn(ui, "↵", t.cmp_break, t.cmp_break, session.word_wrap).clicked() {
            session.word_wrap = !session.word_wrap;
        }

        if toolbar_toggle_btn(
            ui,
            "⇔",
            t.cmp_expand,
            t.cmp_expand,
            session.expand_unchanged,
        )
        .clicked()
        {
            session.expand_unchanged = !session.expand_unchanged;
        }

        if toolbar_toggle_btn(ui, "!", t.cmp_strict, t.cmp_strict, session.strict_mode).clicked()
        {
            session.toggle_strict_mode();
            ui.ctx().request_repaint();
        }

        if toolbar_toggle_btn(
            ui,
            "⊘",
            t.cmp_ignore_ws,
            t.cmp_ignore_ws,
            session.ignore_whitespace && !session.strict_mode,
        )
        .clicked()
        {
            session.ignore_whitespace = !session.ignore_whitespace;
            session.strict_mode = false;
            session.recompute_active();
            ui.ctx().request_repaint();
        }

        ui.add_enabled_ui(can_undo, |ui| {
            if toolbar_icon_btn(ui, "↶", t.cmp_undo, t.cmp_undo).clicked() {
                session.undo_text_edit();
                ui.ctx().request_repaint();
            }
        });

        ui.add_enabled_ui(can_nav, |ui| {
            if toolbar_icon_btn(ui, "↑", t.cmp_prev, t.tip_diff_prev).clicked() {
                session.prev_change();
            }
            if toolbar_icon_btn(ui, "↓", t.cmp_next, t.tip_diff_next).clicked() {
                session.next_change();
            }
        });

        if toolbar_icon_btn(ui, "+", t.cmp_zoom_in, t.cmp_zoom_in).clicked() {
            session.font_size = (session.font_size + 1.0).min(24.0);
        }
        if toolbar_icon_btn(ui, "−", t.cmp_zoom_out, t.cmp_zoom_out).clicked() {
            session.font_size = (session.font_size - 1.0).max(9.0);
        }

        if toolbar_icon_btn(ui, "⌫", t.cmp_clear, t.cmp_clear).clicked() {
            session.clear_paths();
        }
        if toolbar_icon_btn(ui, "⇄", t.cmp_swap, t.cmp_swap).clicked() {
            session.swap_sides();
        }
        if toolbar_icon_btn(ui, "↻", t.cmp_refresh, t.tip_diff_refresh).clicked() {
            session.refresh_display();
        }

        if toolbar_toggle_btn(
            ui,
            "▥",
            t.cmp_diff_map,
            t.cmp_diff_map,
            session.show_diff_map,
        )
        .clicked()
        {
            session.toggle_diff_map();
        }

        if matches!(session.mode, CompareMode::Text) {
            ui.separator();
            if ui.button(t.diff_export).clicked() {
                session.pending_export = true;
            }
        }
    });
}

fn toolbar_icon_btn(ui: &mut egui::Ui, icon: &str, label: &str, tip: &str) -> egui::Response {
    toolbar_toggle_btn(ui, icon, label, tip, false)
}

fn toolbar_toggle_btn(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    tip: &str,
    selected: bool,
) -> egui::Response {
    const TOOLBAR_BTN_H: f32 = 40.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(TOOLBAR_ICON_W, TOOLBAR_BTN_H),
        egui::Sense::click(),
    );
    if ui.is_rect_visible(rect) {
        if selected {
            ui.painter().rect_filled(rect, 4.0, ui.visuals().selection.bg_fill);
        } else if response.hovered() {
            ui.painter().rect_filled(rect, 4.0, ui.visuals().widgets.inactive.bg_fill);
        }
        let text_color = ui.visuals().text_color();
        ui.painter().text(
            rect.center() - egui::vec2(0.0, 7.0),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(15.0),
            text_color,
        );
        ui.painter().text(
            rect.center() + egui::vec2(0.0, 10.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(10.0),
            text_color,
        );
    }
    response.on_hover_text(tip)
}

fn show_path_columns(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    t: &crate::i18n::Locale,
    files_dragged: bool,
) {
    show_two_columns(ui, |ui, half, left| {
        path_side(ui, session, left, half, t, files_dragged);
    });
}

fn show_column_previews(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    t: &crate::i18n::Locale,
    height: f32,
) {
    ui.horizontal(|ui| {
        let w = (ui.available_width() - 4.0) / 2.0;
        side_pane(ui, session, true, w, height, t, session.has_side_preview(true));
        ui.add_space(4.0);
        side_pane(
            ui,
            session,
            false,
            w,
            height,
            t,
            session.has_side_preview(false),
        );
    });
}

fn path_side(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    left: bool,
    width: f32,
    t: &crate::i18n::Locale,
    files_dragged: bool,
) {
    let (row_rect, row_response) = ui.allocate_exact_size(
        egui::vec2(width, PATH_ROW_H),
        egui::Sense::hover(),
    );
    session.register_drop_rect(left, row_rect);
    let hovered = row_response.hovered()
        || (files_dragged && pointer_on_rect(ui.ctx(), row_rect));
    if hovered && files_dragged {
        if left {
            session.drop_target_left = true;
        } else {
            session.drop_target_right = true;
        }
        ui.painter().rect_stroke(
            row_rect.expand(1.0),
            4.0,
            egui::Stroke::new(2.0, ui.visuals().selection.bg_fill),
        );
    }

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(row_rect), |ui| {
        ui.horizontal(|ui| {
            let action_w = path_action_width(session);
            let text_w = (ui.available_width() - action_w).max(40.0);
            let text = if left {
                &mut session.left_path_text
            } else {
                &mut session.right_path_text
            };
            let hint = if session.compare_intent == CompareIntent::Folder {
                if left {
                    t.cmp_left_folder_hint
                } else {
                    t.cmp_right_folder_hint
                }
            } else if left {
                t.cmp_left_file_hint
            } else {
                t.cmp_right_file_hint
            };
            let response = ui.add(
                egui::TextEdit::singleline(text)
                    .desired_width(text_w)
                    .hint_text(hint),
            );
            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                session.refresh_display();
            }

            ui.allocate_ui_with_layout(
                egui::vec2(action_w, PATH_ROW_H),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    match session.compare_intent {
                        CompareIntent::Folder => {
                            if ui.small_button("📁").on_hover_text(t.cmp_pick_folder).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    if left {
                                        session.left_path_text = path.display().to_string();
                                    } else {
                                        session.right_path_text = path.display().to_string();
                                    }
                                    session.refresh_display();
                                }
                            }
                        }
                        CompareIntent::File => {
                            if ui.small_button("📄").on_hover_text(t.cmp_pick_file).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    if left {
                                        session.left_path_text = path.display().to_string();
                                    } else {
                                        session.right_path_text = path.display().to_string();
                                    }
                                    session.refresh_display();
                                }
                            }
                            if matches!(session.mode, CompareMode::Text) {
                                let dirty = if left {
                                    session.left_dirty
                                } else {
                                    session.right_dirty
                                };
                                let save_tip = if left {
                                    t.diff_save_left
                                } else {
                                    t.diff_save_right
                                };
                                ui.add_enabled_ui(dirty, |ui| {
                                    if ui
                                        .small_button("💾")
                                        .on_hover_text(save_tip)
                                        .clicked()
                                    {
                                        if left {
                                            session.pending_save_left = true;
                                        } else {
                                            session.pending_save_right = true;
                                        }
                                    }
                                });
                            }
                        }
                    }
                },
            );
        });
    });
}

fn path_action_width(session: &CompareSession) -> f32 {
    match session.compare_intent {
        CompareIntent::Folder => 40.0,
        CompareIntent::File => {
            if matches!(session.mode, CompareMode::Text) {
                76.0
            } else {
                40.0
            }
        }
    }
}

fn show_encoding_columns(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    t: &crate::i18n::Locale,
    row_h: f32,
) {
    let half = compare_half_width(ui.available_width());
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), row_h),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            encoding_column(ui, session, true, half, row_h, t);
            ui.separator();
            encoding_column(ui, session, false, half, row_h, t);
        },
    );
}

fn encoding_column(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    left: bool,
    half: f32,
    row_h: f32,
    t: &crate::i18n::Locale,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(half, row_h),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            if left {
                encoding_side(
                    ui,
                    &mut session.left_encoding,
                    &mut session.left_save_encoding,
                    t.cmp_left_enc,
                    t.cmp_save_enc,
                    half,
                    row_h,
                );
            } else {
                encoding_side(
                    ui,
                    &mut session.right_encoding,
                    &mut session.right_save_encoding,
                    t.cmp_right_enc,
                    t.cmp_save_enc,
                    half,
                    row_h,
                );
            }
        },
    );
}

fn encoding_side(
    ui: &mut egui::Ui,
    open_enc: &mut String,
    save_enc: &mut String,
    open_label: &str,
    save_label: &str,
    width: f32,
    row_h: f32,
) {
    let combo_w = ((width - 8.0) * 0.38).clamp(72.0, 120.0);
    ui.scope(|ui| {
        ui.set_min_size(egui::vec2(width, row_h));
        ui.set_max_size(egui::vec2(width, row_h));
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_min_height(row_h);
            ui.set_max_height(row_h);
            ui.label(open_label);
            egui::ComboBox::from_id_salt(open_label)
                .selected_text(open_enc.as_str())
                .width(combo_w)
                .show_ui(ui, |ui| {
                    for enc in ["UTF-8", "GBK", "UTF-16 LE", "Latin-1"] {
                        ui.selectable_value(open_enc, enc.to_string(), enc);
                    }
                });
            ui.separator();
            ui.label(save_label);
            egui::ComboBox::from_id_salt(save_label)
                .selected_text(save_enc.as_str())
                .width(combo_w)
                .show_ui(ui, |ui| {
                    for enc in ["UTF-8", "GBK", "UTF-16 LE", "Latin-1"] {
                        ui.selectable_value(save_enc, enc.to_string(), enc);
                    }
                });
        });
    });
}

fn side_pane(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    left: bool,
    width: f32,
    height: f32,
    t: &crate::i18n::Locale,
    allow_preview: bool,
) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    session.register_drop_rect(left, rect);
    let files_dragged = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    let hovered = response.hovered() || (files_dragged && pointer_on_rect(ui.ctx(), rect));
    if hovered && files_dragged {
        if left {
            session.drop_target_left = true;
        } else {
            session.drop_target_right = true;
        }
    }

    let show_content = allow_preview && session.has_side_preview(left);
    if show_content {
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
            preview_pane(ui, session, left, width, height);
        });
    } else {
        let title = if left { t.cmp_left_pane } else { t.cmp_right_pane };
        let stroke = if hovered && files_dragged {
            egui::Stroke::new(2.0, ui.visuals().selection.bg_fill)
        } else {
            ui.visuals().widgets.noninteractive.bg_stroke
        };
        ui.painter().rect_stroke(rect, 4.0, stroke);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{title}\n{}", t.cmp_drag_hint),
            egui::FontId::proportional(14.0),
            ui.visuals().weak_text_color(),
        );
    }
}

fn pointer_on_rect(ctx: &egui::Context, rect: egui::Rect) -> bool {
    ctx.pointer_latest_pos()
        .is_some_and(|p| rect.contains(p))
}

fn preview_pane(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    left: bool,
    width: f32,
    height: f32,
) {
    let path = if left {
        session.left_path.display().to_string()
    } else {
        session.right_path.display().to_string()
    };
    let kind = if left {
        session.left_preview_kind
    } else {
        session.right_preview_kind
    };
    let font_size = session.font_size;

    ui.allocate_ui_with_layout(
        egui::vec2(width, height),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.label(
                egui::RichText::new(path)
                    .strong()
                    .size(font_size),
            );
            ui.separator();
            let inner_h = ui.available_height();
            match kind {
                SidePreviewKind::Text => {
                    editable_text_pane(ui, session, left, inner_h, font_size);
                }
                SidePreviewKind::Binary => {
                    let bytes = if left {
                        &session.left_binary_bytes
                    } else {
                        &session.right_binary_bytes
                    };
                    binary_preview(ui, bytes, inner_h, session.font_size);
                }
                SidePreviewKind::Folder => {
                    let entries = if left {
                        &session.left_folder_entries
                    } else {
                        &session.right_folder_entries
                    };
                    folder_preview(ui, entries, inner_h);
                }
                SidePreviewKind::None => {}
            }
        },
    );
}

fn editable_text_pane(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    left: bool,
    height: f32,
    font_size: f32,
) {
    let before_left = session.left_text.clone();
    let before_right = session.right_text.clone();
    let text = if left {
        &mut session.left_text
    } else {
        &mut session.right_text
    };
    let response = egui::ScrollArea::vertical()
        .max_height(height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(text)
                    .id(egui::Id::new((session.id, left, "preview_edit")))
                    .font(egui::FontId::monospace(font_size))
                    .desired_width(ui.available_width())
                    .lock_focus(true),
            )
        })
        .inner;
    if response.changed() {
        session.push_edit_undo_snapshot(&before_left, &before_right);
        if left {
            session.left_dirty = true;
        } else {
            session.right_dirty = true;
        }
        session.text_edit_pending_recompute = true;
    }
}

fn binary_preview(ui: &mut egui::Ui, bytes: &[u8], height: f32, font_size: f32) {
    use crate::diff::binary_diff::{format_ascii_byte, format_hex_byte};
    const BYTES_PER_ROW: usize = 16;
    let rows = bytes.len().div_ceil(BYTES_PER_ROW);
    let row_h = font_size + 4.0;
    egui::ScrollArea::vertical()
        .max_height(height)
        .auto_shrink([false, false])
        .show_rows(ui, row_h, rows, |ui, range| {
            for row in range {
                let offset = row * BYTES_PER_ROW;
                let chunk = &bytes[offset..offset + BYTES_PER_ROW.min(bytes.len() - offset)];
                let hex: String = chunk
                    .iter()
                    .map(|b| format_hex_byte(Some(*b)))
                    .collect::<Vec<_>>()
                    .join(" ");
                let ascii: String = chunk
                    .iter()
                    .map(|b| format_ascii_byte(Some(*b)))
                    .collect();
                ui.label(
                    egui::RichText::new(format!("{offset:08X}  {hex:<47}  {ascii}"))
                        .monospace()
                        .size(font_size),
                );
            }
        });
}

fn folder_preview(ui: &mut egui::Ui, entries: &[String], height: f32) {
    egui::ScrollArea::vertical()
        .max_height(height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for name in entries {
                ui.label(name);
            }
        });
}

fn session_has_changes(session: &CompareSession) -> bool {
    match session.mode {
        CompareMode::Text => session
            .text_result
            .as_ref()
            .map(|r| r.change_count() > 0)
            .unwrap_or(false),
        CompareMode::Binary => session
            .binary_result
            .as_ref()
            .map(|r| r.diff_count() > 0)
            .unwrap_or(false),
        _ => false,
    }
}

fn paint_drop_halves(
    ui: &egui::Ui,
    left_rect: egui::Rect,
    right_rect: egui::Rect,
    highlight_left: bool,
    highlight_right: bool,
) {
    let stroke = |active: bool| {
        if active {
            egui::Stroke::new(2.5, ui.visuals().selection.bg_fill)
        } else {
            egui::Stroke::new(1.0, ui.visuals().selection.bg_fill.gamma_multiply(0.35))
        }
    };
    ui.painter().rect_stroke(left_rect.shrink(2.0), 4.0, stroke(highlight_left));
    ui.painter().rect_stroke(right_rect.shrink(2.0), 4.0, stroke(highlight_right));
}

fn resolve_drop_side(
    session: &CompareSession,
    pointer: Option<egui::Pos2>,
    index: usize,
    count: usize,
) -> bool {
    if let Some(pointer) = pointer {
        if let Some(to_right) = session.drop_side_at_pointer(pointer) {
            return to_right;
        }
    }
    if session.drop_target_right && !session.drop_target_left {
        return true;
    }
    if session.drop_target_left && !session.drop_target_right {
        return false;
    }
    if count > 1 {
        return index % 2 == 1;
    }
    if let (Some(left), Some(right)) = (session.left_drop_rect, session.right_drop_rect) {
        if let Some(pointer) = pointer {
            return pointer.x >= (left.right() + right.left()) * 0.5;
        }
    }
    false
}

fn handle_drops(session: &mut CompareSession, ctx: &egui::Context) {
    let paths: Vec<PathBuf> = ctx.input(|i| {
        i.raw
            .dropped_files
            .iter()
            .filter_map(|f| f.path.clone())
            .collect()
    });
    if paths.is_empty() {
        return;
    }

    let pointer = ctx.pointer_latest_pos();
    let count = paths.len();

    for (index, path) in paths.into_iter().enumerate() {
        let to_right = resolve_drop_side(session, pointer, index, count);
        session.apply_dropped_path(path, to_right);
    }

    ctx.input_mut(|i| i.raw.dropped_files.clear());
    ctx.request_repaint();
}

/// Folder toolbar extras rendered inside folder content when in folder mode.
pub fn show_folder_extras(app: &RustpadApp, session: &mut CompareSession, ui: &mut egui::Ui) {
    let t = app.tr();
    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_id_salt(("cmp_fdiff_mode", session.id))
            .selected_text(match session.folder_options.mode {
                crate::diff::FolderCompareMode::Quick => t.fdiff_mode_quick,
                crate::diff::FolderCompareMode::Deep => t.fdiff_mode_deep,
            })
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(
                        &mut session.folder_options.mode,
                        crate::diff::FolderCompareMode::Quick,
                        t.fdiff_mode_quick,
                    )
                    .clicked()
                {
                    session.run_folder_compare();
                }
                if ui
                    .selectable_value(
                        &mut session.folder_options.mode,
                        crate::diff::FolderCompareMode::Deep,
                        t.fdiff_mode_deep,
                    )
                    .clicked()
                {
                    session.run_folder_compare();
                }
            });

        egui::ComboBox::from_id_salt(("cmp_fdiff_filter", session.id))
            .selected_text(match session.folder_filter {
                FolderDiffFilter::All => t.fdiff_filter_all,
                FolderDiffFilter::Diff => t.fdiff_filter_diff,
                FolderDiffFilter::DifferentOnly => t.fdiff_filter_diff_only,
                FolderDiffFilter::UniqueOnly => t.fdiff_filter_unique,
                FolderDiffFilter::Identical => t.fdiff_filter_identical,
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut session.folder_filter, FolderDiffFilter::All, t.fdiff_filter_all);
                ui.selectable_value(&mut session.folder_filter, FolderDiffFilter::Diff, t.fdiff_filter_diff);
                ui.selectable_value(
                    &mut session.folder_filter,
                    FolderDiffFilter::DifferentOnly,
                    t.fdiff_filter_diff_only,
                );
                ui.selectable_value(
                    &mut session.folder_filter,
                    FolderDiffFilter::UniqueOnly,
                    t.fdiff_filter_unique,
                );
                ui.selectable_value(
                    &mut session.folder_filter,
                    FolderDiffFilter::Identical,
                    t.fdiff_filter_identical,
                );
            });

        ui.menu_button(t.fdiff_batch_sync, |ui| {
            if ui.button(t.fdiff_mirror_to_right).clicked() {
                session.folder_batch_sync(SyncDirection::ToRight, SyncScope::MirrorLeftToRight);
                ui.close_menu();
            }
            if ui.button(t.fdiff_mirror_to_left).clicked() {
                session.folder_batch_sync(SyncDirection::ToLeft, SyncScope::MirrorRightToLeft);
                ui.close_menu();
            }
        });
    });
    ui.separator();
}
