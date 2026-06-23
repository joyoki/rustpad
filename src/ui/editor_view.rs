use eframe::egui;
use eframe::epaint::Color32;

use crate::app::RustpadApp;
use crate::editor::context_actions::{self, mark_line_color};
use crate::editor::fold::FoldState;
use crate::editor::Cursor;

const LINE_NUMBER_FONT_SIZE: f32 = 14.0;
const CONTENT_EXTENT_LINE_WIDTH: f32 = 2.0;

/// Render the main editor view.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    // Hide the editor only when a diff is actually being shown. If diff mode is
    // toggled on but no result is ready yet, keep rendering the editor so the
    // central area never goes blank.
    if app.show_diff_view && app.diff_result.is_some() {
        return;
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        let font_size = app.config.editor.font_size;
        let line_height = font_size * 1.4;
        let show_line_numbers = app.config.editor.show_line_numbers;
        let highlight_current_line = app.config.editor.highlight_current_line;

        let text = app.tab_manager.active().buffer.text();
        let line_count = app.tab_manager.active().buffer.line_count();
        let scroll_offset = app.tab_manager.active().scroll_offset;
        let cursor_line = app.tab_manager.active().cursor.line;
        let cursor_col = app.tab_manager.active().cursor.col;
        let file_path = app.tab_manager.active().file_path.clone();
        let syntax_override = app.tab_manager.active().syntax_override.clone();

        let gutter_width = if show_line_numbers {
            let digits = format!("{}", line_count).len();
            (digits as f32 * LINE_NUMBER_FONT_SIZE * 0.65) + 8.0
        } else {
            0.0
        };
        const FOLD_GUTTER_WIDTH: f32 = 16.0;
        let total_gutter_width = gutter_width + FOLD_GUTTER_WIDTH;

        let available_width = ui.available_width() - total_gutter_width;
        let available_height = ui.available_height();

        let filename = file_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled.txt".to_string());
        let syntax_name = app.highlighter.syntax_name_for_file(
            &filename,
            syntax_override.as_deref(),
        );
        let highlighted = app.highlighter.highlight_document_by_name(&text, &syntax_name);

        app.tab_manager
            .active_mut()
            .editor_extras
            .fold_state
            .detect_folds(&text);

        let display_blank = app.config.editor.display_blank_chars;
        let display_non_print = app.config.editor.display_non_print_chars;
        let show_tabs_as_spaces = app.config.editor.show_tabs_as_spaces;
        let line_marks = app.tab_manager.active().editor_extras.line_marks.clone();

        let editor_theme = app.theme_manager.current_theme();
        let bg_color = editor_theme.background_color();
        let current_line_bg = editor_theme.current_line_bg_color();
        let selection_bg = editor_theme.selection_bg_color();
        let cursor_color = editor_theme.cursor_color();
        let gutter_bg = crate::config::theme::EditorTheme::to_color32(editor_theme.gutter_bg);
        let gutter_fg = crate::config::theme::EditorTheme::to_color32(editor_theme.line_number_fg);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            if let Some(start_line) = draw_fold_gutter(
                ui,
                line_count,
                scroll_offset,
                line_height,
                font_size,
                FOLD_GUTTER_WIDTH,
                available_height,
                gutter_bg,
                &app.tab_manager.active().editor_extras.fold_state,
            ) {
                app.tab_manager
                    .active_mut()
                    .editor_extras
                    .fold_state
                    .toggle_fold(start_line);
                ctx.request_repaint();
            }

            if show_line_numbers {
                draw_line_numbers(
                    ui,
                    line_count,
                    scroll_offset,
                    line_height,
                    gutter_width,
                    available_height,
                    gutter_bg,
                    gutter_fg,
                    &line_marks,
                    &app.tab_manager.active().editor_extras.fold_state,
                );
            }

            let fold_state = app.tab_manager.active().editor_extras.fold_state.clone();
            let editor_rect = egui::Rect::from_min_size(
                ui.cursor().left_top(),
                egui::vec2(available_width, available_height),
            );
            let response = ui.allocate_rect(editor_rect, egui::Sense::click_and_drag());
            let painter = ui.painter();

            // Right-click: open context menu, snapshot selection, move cursor to click.
            if response.secondary_clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    app.context_menu_selection = Some(app.tab_manager.active().selection);
                    let click_line = line_from_pointer_y(
                        pos,
                        editor_rect,
                        scroll_offset,
                        line_height,
                        line_count,
                        &fold_state,
                    );
                    let line_text = app
                        .tab_manager
                        .active()
                        .buffer
                        .line(click_line)
                        .unwrap_or_default();
                    let click_cursor = pointer_to_cursor(
                        painter,
                        pos,
                        editor_rect,
                        font_size,
                        click_line,
                        line_text,
                        display_blank,
                        show_tabs_as_spaces,
                        display_non_print,
                    );
                    app.tab_manager.active_mut().cursor = click_cursor;
                    if app.tab_manager.active().selection.is_empty() {
                        app.tab_manager.active_mut().selection =
                            crate::editor::Selection::cursor(click_cursor);
                    }
                    app.show_context_menu = true;
                    app.context_menu_pos = pos;
                }
            }

            let modal_blocks_focus = app.blocks_editor_focus();

            // Focus management: restore focus from last frame (never steal from dialogs).
            if app.editor_has_focus && !modal_blocks_focus {
                response.request_focus();
            }

            // Mouse: click, drag-select, double-click word select (skip while menu open).
            if !app.show_context_menu {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                let (shift, alt) = ui.input(|i| (i.modifiers.shift, i.modifiers.alt));
                if response.drag_started() {
                    response.request_focus();
                    let click_line = line_from_pointer_y(
                        pointer_pos,
                        editor_rect,
                        scroll_offset,
                        line_height,
                        line_count,
                        &fold_state,
                    );
                    let line_text = app
                        .tab_manager
                        .active()
                        .buffer
                        .line(click_line)
                        .unwrap_or_default();
                    let cursor = pointer_to_cursor(
                        painter,
                        pointer_pos,
                        editor_rect,
                        font_size,
                        click_line,
                        line_text,
                        display_blank,
                        show_tabs_as_spaces,
                        display_non_print,
                    );
                    if shift {
                        let anchor = {
                            let tab = app.tab_manager.active();
                            if tab.selection.is_empty() {
                                tab.cursor
                            } else {
                                tab.selection.start
                            }
                        };
                        app.tab_manager.active_mut().selection =
                            crate::editor::Selection::new(anchor, cursor);
                    } else {
                        app.tab_manager.active_mut().selection =
                            crate::editor::Selection::cursor(cursor);
                    }
                    app.tab_manager.active_mut().column_selection = alt;
                    app.tab_manager.active_mut().cursor = cursor;
                } else if response.dragged() {
                    let click_line = line_from_pointer_y(
                        pointer_pos,
                        editor_rect,
                        scroll_offset,
                        line_height,
                        line_count,
                        &fold_state,
                    );
                    let line_text = app
                        .tab_manager
                        .active()
                        .buffer
                        .line(click_line)
                        .unwrap_or_default();
                    let cursor = pointer_to_cursor(
                        painter,
                        pointer_pos,
                        editor_rect,
                        font_size,
                        click_line,
                        line_text,
                        display_blank,
                        show_tabs_as_spaces,
                        display_non_print,
                    );
                    let anchor = app.tab_manager.active().selection.start;
                    app.tab_manager.active_mut().selection =
                        crate::editor::Selection::new(anchor, cursor);
                    app.tab_manager.active_mut().cursor = cursor;
                } else if response.clicked() {
                    response.request_focus();
                    if response.double_clicked() {
                        select_word_at_cursor(app);
                    } else {
                        let click_line = line_from_pointer_y(
                            pointer_pos,
                            editor_rect,
                            scroll_offset,
                            line_height,
                            line_count,
                            &fold_state,
                        );
                        let line_text = app
                            .tab_manager
                            .active()
                            .buffer
                            .line(click_line)
                            .unwrap_or_default();
                        let cursor = pointer_to_cursor(
                            painter,
                            pointer_pos,
                            editor_rect,
                            font_size,
                            click_line,
                            line_text,
                            display_blank,
                            show_tabs_as_spaces,
                            display_non_print,
                        );
                        if shift {
                            let anchor = {
                                let tab = app.tab_manager.active();
                                if tab.selection.is_empty() {
                                    tab.cursor
                                } else {
                                    tab.selection.start
                                }
                            };
                            app.tab_manager.active_mut().selection =
                                crate::editor::Selection::new(anchor, cursor);
                            if alt {
                                app.tab_manager.active_mut().column_selection = true;
                            }
                        } else {
                            app.tab_manager.active_mut().selection =
                                crate::editor::Selection::cursor(cursor);
                            app.tab_manager.active_mut().column_selection = false;
                        }
                        app.tab_manager.active_mut().cursor = cursor;
                    }
                }
                }
            }

            // Auto-focus on first frame (not while a dialog is open).
            if !modal_blocks_focus && ctx.memory(|m| m.focused().is_none()) {
                response.request_focus();
            }

            // Store focus state for next frame
            let has_focus = response.has_focus();
            app.editor_has_focus = has_focus;

            // Handle scroll
            if response.hovered() {
                let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                if scroll_delta != 0.0 {
                    app.tab_manager.active_mut().scroll_offset =
                        (app.tab_manager.active().scroll_offset - scroll_delta).max(0.0);
                }
            }

            let start_pos = editor_rect.left_top();
            let current_scroll = app.tab_manager.active().scroll_offset;
            let visible_line_total = fold_state.visible_line_count(line_count);
            let first_visible_display = (current_scroll / line_height) as usize;
            let visible_lines_on_screen = (available_height / line_height) as usize + 2;

            // Draw editor background
            painter.rect_filled(editor_rect, 0.0, bg_color);

            // Draw current line highlight
            if highlight_current_line {
                if let Some(vis) = fold_state.visible_line_index(cursor_line) {
                    let y = FoldState::visible_line_y(
                        start_pos.y,
                        vis,
                        line_height,
                        current_scroll,
                    );
                    if line_y_in_viewport(y, start_pos.y, available_height, line_height) {
                        painter.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(start_pos.x, y),
                                egui::vec2(available_width, line_height),
                            ),
                            0.0,
                            current_line_bg,
                        );
                    }
                }
            }

            // Draw selection highlight
            let selection = app.tab_manager.active().selection;
            let column_mode = app.tab_manager.active().column_selection;
            if !selection.is_empty() {
                let norm = selection.normalized();
                let (col_start, col_end) = if column_mode {
                    crate::editor::column_selection::column_col_range(&norm)
                } else {
                    (0, 0)
                };
                for i in 0..line_count {
                    if fold_state.is_line_hidden(i) || i < norm.start.line || i > norm.end.line {
                        continue;
                    }
                    let Some(vis) = fold_state.visible_line_index(i) else {
                        continue;
                    };
                    if vis < first_visible_display
                        || vis >= first_visible_display + visible_lines_on_screen
                    {
                        continue;
                    }
                    let y = FoldState::visible_line_y(
                        start_pos.y,
                        vis,
                        line_height,
                        current_scroll,
                    );
                    let sel_start_col = if column_mode {
                        col_start
                    } else if i == norm.start.line {
                        norm.start.col
                    } else {
                        0
                    };
                    let sel_end_col = if column_mode {
                        col_end
                    } else if i == norm.end.line {
                        norm.end.col
                    } else {
                        app.tab_manager.active().buffer.line_len(i)
                    };
                    let line_text = app
                        .tab_manager
                        .active()
                        .buffer
                        .line(i)
                        .unwrap_or_default();
                    let x1 = col_to_x(
                        painter,
                        start_pos.x,
                        line_text,
                        sel_start_col,
                        font_size,
                        display_blank,
                        show_tabs_as_spaces,
                        display_non_print,
                    );
                    let x2 = col_to_x(
                        painter,
                        start_pos.x,
                        line_text,
                        sel_end_col,
                        font_size,
                        display_blank,
                        show_tabs_as_spaces,
                        display_non_print,
                    );
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(x1, y),
                            egui::pos2(x2.max(x1 + 1.0), y + line_height),
                        ),
                        0.0,
                        selection_bg,
                    );
                }
            }

            // Highlight every search match while the Find dialog is open so the
            // user gets clear feedback that search is working (Notepad-- style).
            if app.show_search && !app.search_engine.results().is_empty() {
                let match_bg = Color32::from_rgba_unmultiplied(255, 213, 0, 90);
                let current_idx = app.search_engine.current_index();
                let results: Vec<crate::search::SearchMatch> =
                    app.search_engine.results().to_vec();
                for (mi, m) in results.iter().enumerate() {
                    let (s_line, s_col) =
                        app.tab_manager.active().buffer.line_col_for_char_pos(m.start);
                    let (e_line, e_col) =
                        app.tab_manager.active().buffer.line_col_for_char_pos(m.end);
                    for line_idx in s_line..=e_line {
                        if fold_state.is_line_hidden(line_idx) {
                            continue;
                        }
                        let Some(vis) = fold_state.visible_line_index(line_idx) else {
                            continue;
                        };
                        if vis < first_visible_display
                            || vis >= first_visible_display + visible_lines_on_screen
                        {
                            continue;
                        }
                        let y = FoldState::visible_line_y(
                            start_pos.y,
                            vis,
                            line_height,
                            current_scroll,
                        );
                        let line_text = app
                            .tab_manager
                            .active()
                            .buffer
                            .line(line_idx)
                            .unwrap_or_default();
                        let col_start = if line_idx == s_line { s_col } else { 0 };
                        let col_end = if line_idx == e_line {
                            e_col
                        } else {
                            app.tab_manager.active().buffer.line_len(line_idx)
                        };
                        let x1 = col_to_x(
                            painter,
                            start_pos.x,
                            line_text,
                            col_start,
                            font_size,
                            display_blank,
                            show_tabs_as_spaces,
                            display_non_print,
                        );
                        let x2 = col_to_x(
                            painter,
                            start_pos.x,
                            line_text,
                            col_end,
                            font_size,
                            display_blank,
                            show_tabs_as_spaces,
                            display_non_print,
                        );
                        let color = if Some(mi) == current_idx {
                            Color32::from_rgba_unmultiplied(255, 165, 0, 140)
                        } else {
                            match_bg
                        };
                        painter.rect_filled(
                            egui::Rect::from_min_max(
                                egui::pos2(x1, y),
                                egui::pos2(x2.max(x1 + 1.0), y + line_height),
                            ),
                            0.0,
                            color,
                        );
                    }
                }
            }

            // Draw text lines with syntax highlighting (color marks = background only).
            let text_marks = &app.tab_manager.active().editor_extras.text_marks;
            for vis in first_visible_display
                ..(first_visible_display + visible_lines_on_screen).min(visible_line_total)
            {
                let Some(i) = fold_state.buffer_line_at_visible_index(vis, line_count) else {
                    continue;
                };
                let y = FoldState::visible_line_y(start_pos.y, vis, line_height, current_scroll);
                if !line_y_in_viewport(y, start_pos.y, available_height, line_height) {
                    continue;
                }

                draw_line_mark_backgrounds(
                    painter,
                    text_marks,
                    &app.tab_manager.active().buffer,
                    i,
                    start_pos.x,
                    y,
                    line_height,
                    font_size,
                    display_blank,
                    show_tabs_as_spaces,
                    display_non_print,
                );

                if let Some(line_spans) = highlighted.get(i) {
                    let mut x = start_pos.x;
                    for (style, text_segment) in line_spans {
                        let mut display_seg = text_segment.clone();
                        if display_blank || show_tabs_as_spaces {
                            display_seg = context_actions::visualize_whitespace(
                                &display_seg,
                                display_blank,
                                show_tabs_as_spaces,
                            );
                        }
                        if display_non_print {
                            display_seg = context_actions::visualize_non_prints(&display_seg);
                        }
                        let color = Color32::from_rgb(
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                        );
                        let galley = painter.layout_no_wrap(
                            display_seg,
                            egui::FontId::monospace(font_size),
                            color,
                        );
                        let galley_size = galley.size();
                        painter.galley(egui::pos2(x, y), galley, Color32::WHITE);
                        x += galley_size.x;
                    }
                }
            }

            // Draw cursor (with blinking)
            if let Some(vis) = fold_state.visible_line_index(cursor_line) {
                let cursor_y =
                    FoldState::visible_line_y(start_pos.y, vis, line_height, current_scroll);
                if line_y_in_viewport(cursor_y, start_pos.y, available_height, line_height) {
                    let line_text = app
                        .tab_manager
                        .active()
                        .buffer
                        .line(cursor_line)
                        .unwrap_or_default();
                    let cursor_x = col_to_x(
                        painter,
                        start_pos.x,
                        line_text,
                        cursor_col,
                        font_size,
                        display_blank,
                        show_tabs_as_spaces,
                        display_non_print,
                    );

                    let time = ctx.input(|i| i.time);
                    let visible = (time * 2.0) as i32 % 2 == 0;
                    if visible || has_focus {
                        painter.line_segment(
                            [
                                egui::pos2(cursor_x, cursor_y),
                                egui::pos2(cursor_x, cursor_y + line_height),
                            ],
                            egui::Stroke::new(2.0, cursor_color),
                        );
                    }

                    ctx.request_repaint_after(std::time::Duration::from_millis(500));
                }
            }

            // Handle keyboard input only when this editor area owns egui focus.
            let editor_owns_focus = ctx.memory(|m| m.focused()) == Some(response.id);
            if has_focus && editor_owns_focus {
                handle_keyboard_input(app, ctx);
            }

            // Keep cursor within visible area
            let cursor_line_for_scroll = app.tab_manager.active().cursor.line;
            ensure_cursor_visible(
                &mut app.tab_manager.active_mut().scroll_offset,
                cursor_line_for_scroll,
                line_height,
                available_height,
                &fold_state,
            );
        });
    });

    crate::ui::editor_context_menu::show(app, ctx);
}

fn line_y_in_viewport(y: f32, viewport_top: f32, viewport_height: f32, line_height: f32) -> bool {
    let rel = y - viewport_top;
    rel >= -line_height && rel <= viewport_height
}

/// Foreground/syntax text color is unchanged; this only fills behind the glyphs.
fn draw_line_mark_backgrounds(
    painter: &egui::Painter,
    text_marks: &[crate::editor::context_actions::TextMark],
    buffer: &crate::editor::TextBuffer,
    line: usize,
    start_x: f32,
    y: f32,
    line_height: f32,
    font_size: f32,
    display_blank: bool,
    show_tabs_as_spaces: bool,
    display_non_print: bool,
) {
    for mark in text_marks {
        let norm = mark.selection.normalized();
        if norm.is_empty() || line < norm.start.line || line > norm.end.line {
            continue;
        }
        let sel_start_col = if line == norm.start.line {
            norm.start.col
        } else {
            0
        };
        let sel_end_col = if line == norm.end.line {
            norm.end.col
        } else {
            buffer.line_len(line)
        };
        if sel_start_col >= sel_end_col {
            continue;
        }
        let line_text = buffer.line(line).unwrap_or_default();
        let x1 = col_to_x(
            painter,
            start_x,
            line_text,
            sel_start_col,
            font_size,
            display_blank,
            show_tabs_as_spaces,
            display_non_print,
        );
        let x2 = col_to_x(
            painter,
            start_x,
            line_text,
            sel_end_col,
            font_size,
            display_blank,
            show_tabs_as_spaces,
            display_non_print,
        );
        let (r, g, b) = mark_line_color(mark.color);
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(x1, y),
                egui::pos2(x2.max(x1 + 1.0), y + line_height),
            ),
            0.0,
            Color32::from_rgb(r, g, b),
        );
    }
}

/// Scroll the editor so the cursor line stays visible.
fn ensure_cursor_visible(
    scroll_offset: &mut f32,
    cursor_line: usize,
    line_height: f32,
    viewport_height: f32,
    fold_state: &FoldState,
) {
    let Some(visible_index) = fold_state.visible_line_index(cursor_line) else {
        return;
    };
    let cursor_top = visible_index as f32 * line_height;
    let cursor_bottom = cursor_top + line_height;

    if cursor_top < *scroll_offset {
        *scroll_offset = cursor_top;
    } else if cursor_bottom > *scroll_offset + viewport_height {
        *scroll_offset = (cursor_bottom - viewport_height).max(0.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_fold_gutter(
    ui: &mut egui::Ui,
    line_count: usize,
    scroll_offset: f32,
    line_height: f32,
    font_size: f32,
    fold_gutter_width: f32,
    available_height: f32,
    gutter_bg: Color32,
    fold_state: &FoldState,
) -> Option<usize> {
    let gutter_rect = egui::Rect::from_min_size(
        ui.cursor().left_top(),
        egui::vec2(fold_gutter_width, available_height),
    );
    ui.allocate_rect(gutter_rect, egui::Sense::hover());

    let origin = gutter_rect.left_top();

    let icon_yellow = Color32::from_rgb(255, 224, 120);
    let icon_border = Color32::from_rgb(170, 150, 70);
    let scope_color = Color32::from_rgb(190, 190, 190);
    let icon_size = (font_size * 0.72).clamp(9.0, 13.0);

    let visible_line_total = fold_state.visible_line_count(line_count);
    let first_visible_display = (scroll_offset / line_height) as usize;
    let visible_lines_on_screen = (available_height / line_height) as usize + 2;

    let mut clicked_start: Option<usize> = None;

    struct FoldIconPaint {
        rect: egui::Rect,
        folded: bool,
        scope_end_y: Option<f32>,
    }
    struct FoldGuidePaint {
        x: f32,
        y: f32,
    }

    let mut icons: Vec<FoldIconPaint> = Vec::new();
    let mut guides: Vec<FoldGuidePaint> = Vec::new();

    for vis in first_visible_display
        ..(first_visible_display + visible_lines_on_screen).min(visible_line_total)
    {
        let Some(buffer_line) = fold_state.buffer_line_at_visible_index(vis, line_count) else {
            continue;
        };
        let y = FoldState::visible_line_y(origin.y, vis, line_height, scroll_offset);
        if !line_y_in_viewport(y, origin.y, available_height, line_height) {
            continue;
        }

        if let Some(range) = fold_state.range_at_start(buffer_line) {
            let x = origin.x + 1.0 + range.level as f32 * 3.0;
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(x + icon_size * 0.5, y + line_height * 0.5),
                egui::vec2(icon_size, icon_size),
            );
            let icon_response = ui.allocate_rect(icon_rect, egui::Sense::click());
            if icon_response.clicked() {
                clicked_start = Some(range.start_line);
            }
            let scope_end_y = if !range.folded {
                fold_state.visible_line_index(range.end_line).map(|end_vis| {
                    FoldState::visible_line_y(origin.y, end_vis, line_height, scroll_offset)
                        + line_height * 0.5
                })
            } else {
                None
            };
            icons.push(FoldIconPaint {
                rect: icon_rect,
                folded: range.folded,
                scope_end_y,
            });
        }

        for range in fold_state.expanded_ranges_containing(buffer_line) {
            if buffer_line <= range.start_line || buffer_line >= range.end_line {
                continue;
            }
            let x = origin.x + 1.0 + icon_size * 0.5 + range.level as f32 * 3.0;
            guides.push(FoldGuidePaint { x, y });
        }
    }

    let painter = ui.painter();
    painter.rect_filled(gutter_rect, 0.0, gutter_bg);

    for guide in guides {
        painter.line_segment(
            [
                egui::pos2(guide.x, guide.y),
                egui::pos2(guide.x, guide.y + line_height),
            ],
            egui::Stroke::new(1.0, scope_color),
        );
    }

    for icon in icons {
        painter.rect_filled(icon.rect, 1.0, icon_yellow);
        painter.rect_stroke(icon.rect, 1.0, egui::Stroke::new(1.0, icon_border));
        let symbol = if icon.folded { "+" } else { "−" };
        let galley = painter.layout_no_wrap(
            symbol.to_string(),
            egui::FontId::monospace(font_size * 0.62),
            Color32::BLACK,
        );
        let text_pos = icon.rect.center() - galley.size() * 0.5;
        painter.galley(text_pos, galley, Color32::BLACK);
        if let Some(end_y) = icon.scope_end_y {
            let line_x = icon.rect.center().x;
            painter.line_segment(
                [
                    egui::pos2(line_x, icon.rect.max.y),
                    egui::pos2(line_x, end_y),
                ],
                egui::Stroke::new(1.0, scope_color),
            );
        }
    }

    clicked_start
}

/// Y range of the orange content-extent line, clipped to the visible gutter viewport.
fn content_extent_y_range(
    origin_y: f32,
    viewport_height: f32,
    line_count: usize,
    line_height: f32,
    scroll_offset: f32,
    fold_state: &FoldState,
) -> Option<(f32, f32)> {
    let visible_total = fold_state.visible_line_count(line_count);
    if visible_total == 0 {
        return None;
    }
    let content_top =
        FoldState::visible_line_y(origin_y, 0, line_height, scroll_offset);
    let last_vis = visible_total.saturating_sub(1);
    let content_bottom = FoldState::visible_line_y(origin_y, last_vis, line_height, scroll_offset)
        + line_height;
    let y_start = content_top.max(origin_y);
    let y_end = content_bottom.min(origin_y + viewport_height);
    if y_end > y_start {
        Some((y_start, y_end))
    } else {
        None
    }
}

/// Notepad++-style orange vertical line: height follows document content.
fn draw_content_extent_line(
    painter: &egui::Painter,
    x: f32,
    origin_y: f32,
    viewport_height: f32,
    line_count: usize,
    line_height: f32,
    scroll_offset: f32,
    fold_state: &FoldState,
) {
    let Some((y_start, y_end)) = content_extent_y_range(
        origin_y,
        viewport_height,
        line_count,
        line_height,
        scroll_offset,
        fold_state,
    ) else {
        return;
    };
    let orange = Color32::from_rgb(255, 140, 0);
    painter.rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(x - CONTENT_EXTENT_LINE_WIDTH, y_start),
            egui::pos2(x, y_end),
        ),
        0.0,
        orange,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_line_numbers(
    ui: &mut egui::Ui,
    line_count: usize,
    scroll_offset: f32,
    line_height: f32,
    gutter_width: f32,
    available_height: f32,
    gutter_bg: Color32,
    gutter_fg: Color32,
    line_marks: &std::collections::HashMap<usize, u8>,
    fold_state: &FoldState,
) {
    let visible_line_total = fold_state.visible_line_count(line_count);
    let first_visible_display = (scroll_offset / line_height) as usize;
    let visible_lines_on_screen = (available_height / line_height) as usize + 2;

    let gutter_rect = egui::Rect::from_min_size(
        ui.cursor().left_top(),
        egui::vec2(gutter_width, available_height),
    );
    ui.allocate_rect(gutter_rect, egui::Sense::hover());

    let painter = ui.painter();
    let origin = gutter_rect.left_top();

    // Draw gutter background
    painter.rect_filled(gutter_rect, 0.0, gutter_bg);

    for vis in first_visible_display
        ..(first_visible_display + visible_lines_on_screen).min(visible_line_total)
    {
        let Some(i) = fold_state.buffer_line_at_visible_index(vis, line_count) else {
            continue;
        };
        let y = FoldState::visible_line_y(origin.y, vis, line_height, scroll_offset);
        if !line_y_in_viewport(y, origin.y, available_height, line_height) {
            continue;
        }
        if let Some(&color_idx) = line_marks.get(&i) {
            let (r, g, b) = mark_line_color(color_idx);
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(origin.x, y),
                    egui::pos2(origin.x + 4.0, y + line_height),
                ),
                0.0,
                Color32::from_rgb(r, g, b),
            );
        }
        let digits = (i + 1).to_string().len().max(line_count.to_string().len());
        let line_num = format!("{:>width$}", i + 1, width = digits);
        let galley = painter.layout_no_wrap(
            line_num,
            egui::FontId::monospace(LINE_NUMBER_FONT_SIZE),
            gutter_fg,
        );
        let num_x = origin.x + gutter_width - galley.size().x - 2.0;
        painter.galley(egui::pos2(num_x, y), galley, Color32::TRANSPARENT);
    }

    // Orange content-extent line (Notepad++ style): from line 1 to last line of content.
    draw_content_extent_line(
        painter,
        origin.x + gutter_width,
        origin.y,
        available_height,
        line_count,
        line_height,
        scroll_offset,
        fold_state,
    );
}

/// Handle all keyboard input for the editor.
fn handle_keyboard_input(app: &mut RustpadApp, ctx: &egui::Context) {
    let mut enter_pressed = false;
    let mut backspace_pressed = false;
    let mut delete_pressed = false;
    let mut tab_pressed = false;
    let mut home_pressed = false;
    let mut end_pressed = false;
    let mut arrow_up = false;
    let mut arrow_down = false;
    let mut arrow_left = false;
    let mut arrow_right = false;
    let mut shift = false;
    let mut alt = false;
    let mut text_input: Vec<String> = Vec::new();

    ctx.input(|i| {
        shift = i.modifiers.shift;
        alt = i.modifiers.alt;
        enter_pressed = i.key_pressed(egui::Key::Enter);
        backspace_pressed = i.key_pressed(egui::Key::Backspace);
        delete_pressed = i.key_pressed(egui::Key::Delete);
        tab_pressed = i.key_pressed(egui::Key::Tab) && !i.modifiers.ctrl && !i.modifiers.command;
        home_pressed = i.key_pressed(egui::Key::Home);
        end_pressed = i.key_pressed(egui::Key::End);
        arrow_up = i.key_pressed(egui::Key::ArrowUp);
        arrow_down = i.key_pressed(egui::Key::ArrowDown);
        arrow_left = i.key_pressed(egui::Key::ArrowLeft);
        arrow_right = i.key_pressed(egui::Key::ArrowRight);

        for event in &i.events {
            if let egui::Event::Text(text) = event {
                if text.chars().all(|c| !c.is_control()) {
                    text_input.push(text.clone());
                }
            }
        }
    });

    if !enter_pressed
        && !backspace_pressed
        && !delete_pressed
        && !tab_pressed
        && !home_pressed
        && !end_pressed
        && !arrow_up
        && !arrow_down
        && !arrow_left
        && !arrow_right
        && text_input.is_empty()
    {
        return;
    }

    let move_cursor_with_selection = |app: &mut RustpadApp, new_cursor: Cursor, extend: bool| {
        let tab = app.tab_manager.active_mut();
        if extend {
            tab.column_selection = alt;
            let anchor = if tab.selection.is_empty() {
                tab.cursor
            } else {
                tab.selection.start
            };
            tab.selection = crate::editor::Selection::new(anchor, new_cursor);
        } else {
            tab.column_selection = false;
            tab.selection = crate::editor::Selection::cursor(new_cursor);
        }
        tab.cursor = new_cursor;
    };

    if enter_pressed {
        let indent_engine = crate::editor::indent::IndentEngine::new(
            crate::editor::indent::IndentConfig {
                use_spaces: app.config.editor.tab_size > 0,
                tab_size: app.config.editor.tab_size,
                auto_indent: app.config.editor.auto_indent,
                smart_indent: true,
            },
        );

        let prev_line = app
            .tab_manager
            .active()
            .buffer
            .line(app.tab_manager.active().cursor.line)
            .unwrap_or_default();
        let (indent, _) = indent_engine.newline_indent(prev_line);
        let newline = "\n".to_string() + &indent;
        app.insert_at_cursor(&newline);
        app.tab_manager.active_mut().selection = crate::editor::Selection::cursor(
            app.tab_manager.active().cursor,
        );
        return;
    }

    if backspace_pressed {
        app.delete_char_before_cursor();
        return;
    }

    if delete_pressed {
        app.delete_char_at_cursor();
        return;
    }

    if tab_pressed {
        let tab_size = app.config.editor.tab_size;
        let spaces = " ".repeat(tab_size);
        app.insert_at_cursor(&spaces);
        return;
    }

    if home_pressed {
        let line = app.tab_manager.active().cursor.line;
        let new_cursor = Cursor::new(line, 0);
        move_cursor_with_selection(app, new_cursor, shift);
        return;
    }

    if end_pressed {
        let line = app.tab_manager.active().cursor.line;
        let max_col = app.tab_manager.active().buffer.line_len(line);
        let new_cursor = Cursor::new(line, max_col);
        move_cursor_with_selection(app, new_cursor, shift);
        return;
    }

    if arrow_up {
        let line = app.tab_manager.active().cursor.line;
        let col = app.tab_manager.active().cursor.col;
        if line > 0 {
            let new_line = line - 1;
            let max_col = app.tab_manager.active().buffer.line_len(new_line);
            let new_cursor = Cursor::new(new_line, col.min(max_col));
            move_cursor_with_selection(app, new_cursor, shift);
        }
        return;
    }
    if arrow_down {
        let line = app.tab_manager.active().cursor.line;
        let col = app.tab_manager.active().cursor.col;
        let line_count = app.tab_manager.active().line_count();
        if line_count > 0 && line < line_count - 1 {
            let new_line = line + 1;
            let max_col = app.tab_manager.active().buffer.line_len(new_line);
            let new_cursor = Cursor::new(new_line, col.min(max_col));
            move_cursor_with_selection(app, new_cursor, shift);
        }
        return;
    }
    if arrow_left {
        let line = app.tab_manager.active().cursor.line;
        let col = app.tab_manager.active().cursor.col;
        let new_cursor = if col > 0 {
            Cursor::new(line, col - 1)
        } else if line > 0 {
            let prev_line = line - 1;
            Cursor::new(prev_line, app.tab_manager.active().buffer.line_len(prev_line))
        } else {
            return;
        };
        move_cursor_with_selection(app, new_cursor, shift);
        return;
    }
    if arrow_right {
        let line = app.tab_manager.active().cursor.line;
        let col = app.tab_manager.active().cursor.col;
        let max_col = app.tab_manager.active().buffer.line_len(line);
        let line_count = app.tab_manager.active().line_count();
        let new_cursor = if col < max_col {
            Cursor::new(line, col + 1)
        } else if line_count > 0 && line < line_count - 1 {
            Cursor::new(line + 1, 0)
        } else {
            return;
        };
        move_cursor_with_selection(app, new_cursor, shift);
        return;
    }

    if !text_input.is_empty() {
        for text in text_input {
            app.insert_at_cursor(&text);
        }
        app.tab_manager.active_mut().selection = crate::editor::Selection::cursor(
            app.tab_manager.active().cursor,
        );
    }
}

fn display_line_text(
    line_text: &str,
    display_blank: bool,
    show_tabs_as_spaces: bool,
    display_non_print: bool,
) -> String {
    let mut display = line_text.to_string();
    if display_blank || show_tabs_as_spaces {
        display = context_actions::visualize_whitespace(
            &display,
            display_blank,
            show_tabs_as_spaces,
        );
    }
    if display_non_print {
        display = context_actions::visualize_non_prints(&display);
    }
    display
}

fn col_to_x(
    painter: &egui::Painter,
    base_x: f32,
    line_text: &str,
    col: usize,
    font_size: f32,
    display_blank: bool,
    show_tabs_as_spaces: bool,
    display_non_print: bool,
) -> f32 {
    let prefix: String = line_text.chars().take(col).collect();
    let display = display_line_text(
        &prefix,
        display_blank,
        show_tabs_as_spaces,
        display_non_print,
    );
    if display.is_empty() {
        return base_x;
    }
    let galley = painter.layout_no_wrap(
        display,
        egui::FontId::monospace(font_size),
        Color32::TRANSPARENT,
    );
    base_x + galley.size().x
}

fn x_to_col(
    painter: &egui::Painter,
    base_x: f32,
    line_text: &str,
    rel_x: f32,
    font_size: f32,
    display_blank: bool,
    show_tabs_as_spaces: bool,
    display_non_print: bool,
) -> usize {
    let line_len = line_text.chars().count();
    if line_len == 0 {
        return 0;
    }

    let target_x = base_x + rel_x.max(0.0);
    let mut lo = 0usize;
    let mut hi = line_len;

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if col_to_x(
            painter,
            base_x,
            line_text,
            mid,
            font_size,
            display_blank,
            show_tabs_as_spaces,
            display_non_print,
        ) < target_x
        {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    if lo > 0 {
        let x_prev = col_to_x(
            painter,
            base_x,
            line_text,
            lo - 1,
            font_size,
            display_blank,
            show_tabs_as_spaces,
            display_non_print,
        );
        let x_lo = col_to_x(
            painter,
            base_x,
            line_text,
            lo,
            font_size,
            display_blank,
            show_tabs_as_spaces,
            display_non_print,
        );
        if target_x - x_prev < x_lo - target_x {
            return lo - 1;
        }
    }
    lo.min(line_len)
}

fn line_from_pointer_y(
    pointer_pos: egui::Pos2,
    editor_rect: egui::Rect,
    scroll_offset: f32,
    line_height: f32,
    line_count: usize,
    fold_state: &FoldState,
) -> usize {
    let rel_y = pointer_pos.y - editor_rect.top() + scroll_offset;
    let visible_index = (rel_y / line_height).max(0.0) as usize;
    fold_state
        .buffer_line_at_visible_index(visible_index, line_count)
        .unwrap_or(line_count.saturating_sub(1))
}

fn pointer_to_cursor(
    painter: &egui::Painter,
    pointer_pos: egui::Pos2,
    editor_rect: egui::Rect,
    font_size: f32,
    line: usize,
    line_text: &str,
    display_blank: bool,
    show_tabs_as_spaces: bool,
    display_non_print: bool,
) -> Cursor {
    let base_x = editor_rect.left();
    let rel_x = pointer_pos.x - base_x;
    let col = x_to_col(
        painter,
        base_x,
        line_text,
        rel_x,
        font_size,
        display_blank,
        show_tabs_as_spaces,
        display_non_print,
    );
    Cursor::new(line, col)
}

fn select_word_at_cursor(app: &mut RustpadApp) {
    let tab = app.tab_manager.active();
    let line = tab.cursor.line;
    let col = tab.cursor.col;
    let line_text = tab.buffer.line(line).unwrap_or_default();
    let chars: Vec<char> = line_text.chars().collect();
    if chars.is_empty() {
        return;
    }

    let col = col.min(chars.len().saturating_sub(1));
    let is_word = |c: char| c.is_alphanumeric() || c == '_';

    let mut start = col;
    while start > 0 && is_word(chars[start - 1]) {
        start -= 1;
    }
    let mut end = col;
    while end < chars.len() && is_word(chars[end]) {
        end += 1;
    }
    if start == end && col < chars.len() {
        end = (col + 1).min(chars.len());
        start = col;
    }

    let tab = app.tab_manager.active_mut();
    tab.selection = crate::editor::Selection::new(Cursor::new(line, start), Cursor::new(line, end));
    tab.cursor = Cursor::new(line, end);
}
