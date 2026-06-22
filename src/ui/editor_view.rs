use eframe::egui;
use eframe::epaint::Color32;

use crate::app::RustpadApp;
use crate::editor::Cursor;

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
            (digits as f32 * font_size * 0.6) + 16.0
        } else {
            0.0
        };

        let available_width = ui.available_width() - gutter_width;
        let available_height = ui.available_height();

        let filename = file_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled.txt".to_string());
        let syntax_name = app
            .highlighter
            .syntax_name_for_file(&filename, syntax_override.as_deref());
        let highlighted = app
            .highlighter
            .highlight_document_by_name(&text, &syntax_name);

        let editor_theme = app.theme_manager.current_theme();
        let bg_color = editor_theme.background_color();
        let current_line_bg = editor_theme.current_line_bg_color();
        let selection_bg = editor_theme.selection_bg_color();
        let cursor_color = editor_theme.cursor_color();
        let gutter_bg = crate::config::theme::EditorTheme::to_color32(editor_theme.gutter_bg);
        let gutter_fg = crate::config::theme::EditorTheme::to_color32(editor_theme.line_number_fg);

        ui.horizontal(|ui| {
            if show_line_numbers {
                draw_line_numbers(
                    ui,
                    line_count,
                    scroll_offset,
                    line_height,
                    font_size,
                    gutter_width,
                    available_height,
                    gutter_bg,
                    gutter_fg,
                );
            }

            let editor_rect = egui::Rect::from_min_size(
                ui.cursor().left_top(),
                egui::vec2(available_width, available_height),
            );
            let response = ui.allocate_rect(editor_rect, egui::Sense::click_and_drag());

            let modal_blocks_focus = app.blocks_editor_focus();

            // Focus management: restore focus from last frame (never steal from dialogs).
            if app.editor_has_focus && !modal_blocks_focus {
                response.request_focus();
            }

            let painter = ui.painter();

            // Mouse: click, drag-select, double-click word select
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let shift = ui.input(|i| i.modifiers.shift);
                if response.drag_started() {
                    response.request_focus();
                    let click_line = line_from_pointer_y(
                        pointer_pos,
                        editor_rect,
                        scroll_offset,
                        line_height,
                        line_count,
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
                    app.tab_manager.active_mut().cursor = cursor;
                } else if response.dragged() {
                    let click_line = line_from_pointer_y(
                        pointer_pos,
                        editor_rect,
                        scroll_offset,
                        line_height,
                        line_count,
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
                        app.tab_manager.active_mut().cursor = cursor;
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
            let first_visible_line = (current_scroll / line_height) as usize;
            let visible_lines = (available_height / line_height) as usize + 2;

            // Draw editor background
            painter.rect_filled(editor_rect, 0.0, bg_color);

            // Draw current line highlight
            if highlight_current_line {
                let y = start_pos.y + (cursor_line as f32 * line_height) - current_scroll;
                if y >= -line_height && y <= available_height {
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

            // Draw selection highlight
            let selection = app.tab_manager.active().selection;
            if !selection.is_empty() {
                let norm = selection.normalized();
                for i in first_visible_line..(first_visible_line + visible_lines).min(line_count) {
                    if i < norm.start.line || i > norm.end.line {
                        continue;
                    }
                    let y = start_pos.y + (i as f32 * line_height) - current_scroll;
                    let sel_start_col = if i == norm.start.line {
                        norm.start.col
                    } else {
                        0
                    };
                    let sel_end_col = if i == norm.end.line {
                        norm.end.col
                    } else {
                        app.tab_manager.active().buffer.line_len(i)
                    };
                    let line_text = app.tab_manager.active().buffer.line(i).unwrap_or_default();
                    let x1 = col_to_x(painter, start_pos.x, line_text, sel_start_col, font_size);
                    let x2 = col_to_x(painter, start_pos.x, line_text, sel_end_col, font_size);
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
                let results: Vec<crate::search::SearchMatch> = app.search_engine.results().to_vec();
                for (mi, m) in results.iter().enumerate() {
                    let (s_line, s_col) = app
                        .tab_manager
                        .active()
                        .buffer
                        .line_col_for_char_pos(m.start);
                    let (e_line, e_col) =
                        app.tab_manager.active().buffer.line_col_for_char_pos(m.end);
                    for line_idx in s_line..=e_line {
                        if line_idx < first_visible_line
                            || line_idx >= (first_visible_line + visible_lines).min(line_count)
                        {
                            continue;
                        }
                        let y = start_pos.y + (line_idx as f32 * line_height) - current_scroll;
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
                        let x1 = col_to_x(painter, start_pos.x, line_text, col_start, font_size);
                        let x2 = col_to_x(painter, start_pos.x, line_text, col_end, font_size);
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

            // Draw text lines with syntax highlighting
            for i in first_visible_line..(first_visible_line + visible_lines).min(line_count) {
                let y = start_pos.y + (i as f32 * line_height) - current_scroll;
                if y < -line_height || y > available_height {
                    continue;
                }

                if let Some(line_spans) = highlighted.get(i) {
                    let mut x = start_pos.x;
                    for (style, text_segment) in line_spans {
                        let color = Color32::from_rgb(
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                        );
                        let galley = painter.layout_no_wrap(
                            text_segment.clone(),
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
            let cursor_y = start_pos.y + (cursor_line as f32 * line_height) - current_scroll;
            if cursor_y >= 0.0 && cursor_y <= available_height {
                let line_text = app
                    .tab_manager
                    .active()
                    .buffer
                    .line(cursor_line)
                    .unwrap_or_default();
                let cursor_x = col_to_x(painter, start_pos.x, line_text, cursor_col, font_size);

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
            );
        });
    });
}

/// Scroll the editor so the cursor line stays visible.
fn ensure_cursor_visible(
    scroll_offset: &mut f32,
    cursor_line: usize,
    line_height: f32,
    viewport_height: f32,
) {
    let cursor_top = cursor_line as f32 * line_height;
    let cursor_bottom = cursor_top + line_height;

    if cursor_top < *scroll_offset {
        *scroll_offset = cursor_top;
    } else if cursor_bottom > *scroll_offset + viewport_height {
        *scroll_offset = (cursor_bottom - viewport_height).max(0.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line_numbers(
    ui: &mut egui::Ui,
    line_count: usize,
    scroll_offset: f32,
    line_height: f32,
    font_size: f32,
    gutter_width: f32,
    available_height: f32,
    gutter_bg: Color32,
    gutter_fg: Color32,
) {
    let first_visible_line = (scroll_offset / line_height) as usize;
    let visible_lines = (available_height / line_height) as usize + 2;

    let gutter_rect = egui::Rect::from_min_size(
        ui.cursor().left_top(),
        egui::vec2(gutter_width, available_height),
    );
    ui.allocate_rect(gutter_rect, egui::Sense::hover());

    let painter = ui.painter();
    let origin = gutter_rect.left_top();

    // Draw gutter background
    painter.rect_filled(gutter_rect, 0.0, gutter_bg);

    for i in first_visible_line..(first_visible_line + visible_lines).min(line_count) {
        let y = origin.y + (i as f32 * line_height) - scroll_offset;
        if y < -line_height || y > available_height {
            continue;
        }
        let line_num = format!("{:>4}", i + 1);
        let galley = painter.layout_no_wrap(
            line_num,
            egui::FontId::monospace(font_size * 0.85),
            gutter_fg,
        );
        painter.galley(egui::pos2(origin.x, y), galley, Color32::TRANSPARENT);
    }
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
    let mut text_input: Vec<String> = Vec::new();

    ctx.input(|i| {
        shift = i.modifiers.shift;
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
            let anchor = if tab.selection.is_empty() {
                tab.cursor
            } else {
                tab.selection.start
            };
            tab.selection = crate::editor::Selection::new(anchor, new_cursor);
        } else {
            tab.selection = crate::editor::Selection::cursor(new_cursor);
        }
        tab.cursor = new_cursor;
    };

    if enter_pressed {
        app.delete_selection();
        let indent_engine =
            crate::editor::indent::IndentEngine::new(crate::editor::indent::IndentConfig {
                use_spaces: app.config.editor.tab_size > 0,
                tab_size: app.config.editor.tab_size,
                auto_indent: app.config.editor.auto_indent,
                smart_indent: true,
            });

        let prev_line = app
            .tab_manager
            .active()
            .buffer
            .line(app.tab_manager.active().cursor.line)
            .unwrap_or_default();
        let (indent, _) = indent_engine.newline_indent(prev_line);

        let line = app.tab_manager.active().cursor.line;
        let col = app.tab_manager.active().cursor.col;
        let pos = app
            .tab_manager
            .active()
            .buffer
            .char_pos_for_line_col(line, col);
        let newline = "\n".to_string() + &indent;
        app.tab_manager
            .active_mut()
            .buffer
            .insert_str(pos, &newline);
        app.tab_manager.active_mut().cursor.line += 1;
        app.tab_manager.active_mut().cursor.col = indent.chars().count();
        app.tab_manager.active_mut().selection =
            crate::editor::Selection::cursor(app.tab_manager.active().cursor);
        app.highlighter.invalidate_all();
        return;
    }

    if backspace_pressed {
        if app.delete_selection() {
            return;
        }
        let line = app.tab_manager.active().cursor.line;
        let col = app.tab_manager.active().cursor.col;
        if col > 0 {
            let pos = app
                .tab_manager
                .active()
                .buffer
                .char_pos_for_line_col(line, col);
            app.tab_manager
                .active_mut()
                .buffer
                .delete_range(pos - 1, pos);
            app.tab_manager.active_mut().cursor.col -= 1;
            app.highlighter.invalidate_all();
        } else if line > 0 {
            let prev_line_len = app.tab_manager.active().buffer.line_len(line - 1);
            let pos = app
                .tab_manager
                .active()
                .buffer
                .char_pos_for_line_col(line, 0);
            if pos > 0 {
                app.tab_manager
                    .active_mut()
                    .buffer
                    .delete_range(pos - 1, pos);
                app.tab_manager.active_mut().cursor.line -= 1;
                app.tab_manager.active_mut().cursor.col = prev_line_len;
                app.highlighter.invalidate_all();
            }
        }
        return;
    }

    if delete_pressed {
        if app.delete_selection() {
            return;
        }
        let line = app.tab_manager.active().cursor.line;
        let col = app.tab_manager.active().cursor.col;
        let pos = app
            .tab_manager
            .active()
            .buffer
            .char_pos_for_line_col(line, col);
        if pos < app.tab_manager.active().buffer.len() {
            app.tab_manager
                .active_mut()
                .buffer
                .delete_range(pos, pos + 1);
            app.highlighter.invalidate_all();
        }
        return;
    }

    if tab_pressed {
        app.delete_selection();
        let tab_size = app.config.editor.tab_size;
        let line = app.tab_manager.active().cursor.line;
        let col = app.tab_manager.active().cursor.col;
        let pos = app
            .tab_manager
            .active()
            .buffer
            .char_pos_for_line_col(line, col);
        let spaces = " ".repeat(tab_size);
        app.tab_manager.active_mut().buffer.insert_str(pos, &spaces);
        app.tab_manager.active_mut().cursor.col += tab_size;
        app.highlighter.invalidate_all();
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
            Cursor::new(
                prev_line,
                app.tab_manager.active().buffer.line_len(prev_line),
            )
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
        app.delete_selection();
        for text in text_input {
            let line = app.tab_manager.active().cursor.line;
            let col = app.tab_manager.active().cursor.col;
            let pos = app
                .tab_manager
                .active()
                .buffer
                .char_pos_for_line_col(line, col);
            app.tab_manager.active_mut().buffer.insert_str(pos, &text);
            app.tab_manager.active_mut().cursor.col += text.chars().count();
            app.highlighter.invalidate_all();
        }
        app.tab_manager.active_mut().selection =
            crate::editor::Selection::cursor(app.tab_manager.active().cursor);
    }
}

fn col_to_x(
    painter: &egui::Painter,
    base_x: f32,
    line_text: &str,
    col: usize,
    font_size: f32,
) -> f32 {
    let prefix: String = line_text.chars().take(col).collect();
    if prefix.is_empty() {
        return base_x;
    }
    let galley = painter.layout_no_wrap(
        prefix,
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
        if col_to_x(painter, base_x, line_text, mid, font_size) < target_x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    // Snap to the nearest column boundary.
    if lo > 0 {
        let x_prev = col_to_x(painter, base_x, line_text, lo - 1, font_size);
        let x_lo = col_to_x(painter, base_x, line_text, lo, font_size);
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
) -> usize {
    let rel_y = pointer_pos.y - editor_rect.top() + scroll_offset;
    ((rel_y / line_height) as usize).min(line_count.saturating_sub(1))
}

fn pointer_to_cursor(
    painter: &egui::Painter,
    pointer_pos: egui::Pos2,
    editor_rect: egui::Rect,
    font_size: f32,
    line: usize,
    line_text: &str,
) -> Cursor {
    let base_x = editor_rect.left();
    let rel_x = pointer_pos.x - base_x;
    let col = x_to_col(painter, base_x, line_text, rel_x, font_size);
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
