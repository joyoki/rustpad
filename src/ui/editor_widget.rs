use eframe::egui;
use eframe::epaint::Color32;

use crate::app::RustpadApp;
use crate::editor::indent::IndentEngine;
use crate::editor::Cursor;

/// Render the main editor widget with syntax highlighting, cursor, and selection.
/// NOTE: Currently not used - editor_view.rs is the active editor.
#[allow(dead_code)]
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
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
        let syntax_name = app.highlighter.detect_syntax(&filename).name.clone();
        let highlighted = app.highlighter.highlight_document_by_name(&text, &syntax_name);

        ui.horizontal(|ui| {
            if show_line_numbers {
                draw_line_numbers(ui, line_count, scroll_offset, line_height, font_size, gutter_width, available_height);
            }

            let editor_rect = egui::Rect::from_min_size(
                ui.cursor().left_top(),
                egui::vec2(available_width, available_height),
            );
            let response = ui.allocate_rect(editor_rect, egui::Sense::click_and_drag());

            // Focus management using response.id
            if app.editor_has_focus {
                response.request_focus();
            }

            if response.clicked() {
                response.request_focus();
                if let Some(click_pos) = response.interact_pointer_pos() {
                    let rel_y = click_pos.y - editor_rect.top() + scroll_offset;
                    let rel_x = click_pos.x - editor_rect.left();
                    let line = (rel_y / line_height) as usize;
                    let col = (rel_x / (font_size * 0.6)) as usize;
                    let safe_line = line.min(line_count.saturating_sub(1));
                    let line_text = app.tab_manager.active().buffer.line(safe_line).unwrap_or_default();
                    let col = col.min(line_text.chars().count());
                    app.tab_manager.active_mut().cursor = Cursor::new(safe_line, col);
                    app.tab_manager.active_mut().selection =
                        crate::editor::Selection::cursor(Cursor::new(safe_line, col));
                }
            }

            if ctx.memory(|m| m.focused().is_none()) {
                response.request_focus();
            }

            let has_focus = response.has_focus();
            app.editor_has_focus = has_focus;

            if response.hovered() {
                let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                if scroll_delta != 0.0 {
                    app.tab_manager.active_mut().scroll_offset =
                        (app.tab_manager.active().scroll_offset - scroll_delta).max(0.0);
                }
            }

            let painter = ui.painter();
            let start_pos = editor_rect.left_top();
            let current_scroll = app.tab_manager.active().scroll_offset;
            let first_visible_line = (current_scroll / line_height) as usize;
            let visible_lines = (available_height / line_height) as usize + 2;

            if highlight_current_line {
                let y = start_pos.y + (cursor_line as f32 * line_height) - current_scroll;
                if y >= -line_height && y <= available_height {
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(start_pos.x, y),
                            egui::vec2(available_width, line_height),
                        ),
                        0.0,
                        Color32::from_rgba_premultiplied(60, 80, 100, 40),
                    );
                }
            }

            let selection = app.tab_manager.active().selection;
            if !selection.is_empty() {
                let norm = selection.normalized();
                for i in first_visible_line..(first_visible_line + visible_lines).min(line_count) {
                    if i < norm.start.line || i > norm.end.line {
                        continue;
                    }
                    let y = start_pos.y + (i as f32 * line_height) - current_scroll;
                    let sel_start_col = if i == norm.start.line { norm.start.col } else { 0 };
                    let sel_end_col = if i == norm.end.line {
                        norm.end.col
                    } else {
                        app.tab_manager.active().buffer.line_len(i)
                    };
                    let x1 = start_pos.x + sel_start_col as f32 * font_size * 0.6;
                    let x2 = start_pos.x + sel_end_col as f32 * font_size * 0.6;
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(x1, y),
                            egui::pos2(x2, y + line_height),
                        ),
                        0.0,
                        Color32::from_rgba_premultiplied(50, 100, 200, 60),
                    );
                }
            }

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

            let cursor_y = start_pos.y + (cursor_line as f32 * line_height) - current_scroll;
            if cursor_y >= 0.0 && cursor_y <= available_height {
                let line_text = app.tab_manager.active().buffer.line(cursor_line).unwrap_or_default();
                let prefix: String = line_text.chars().take(cursor_col).collect();
                let cursor_x = start_pos.x + prefix.len() as f32 * font_size * 0.6;

                let time = ctx.input(|i| i.time);
                let visible = (time * 2.0) as i32 % 2 == 0;
                if visible || has_focus {
                    painter.line_segment(
                        [
                            egui::pos2(cursor_x, cursor_y),
                            egui::pos2(cursor_x, cursor_y + line_height),
                        ],
                        egui::Stroke::new(2.0, Color32::WHITE),
                    );
                }

                ctx.request_repaint_after(std::time::Duration::from_millis(500));
            }

            if has_focus {
                handle_text_input(app, ctx);
            }
        });
    });
}

fn draw_line_numbers(
    ui: &mut egui::Ui,
    line_count: usize,
    scroll_offset: f32,
    line_height: f32,
    font_size: f32,
    gutter_width: f32,
    available_height: f32,
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

    for i in first_visible_line..(first_visible_line + visible_lines).min(line_count) {
        let y = origin.y + (i as f32 * line_height) - scroll_offset;
        if y < -line_height || y > available_height {
            continue;
        }
        let line_num = format!("{:>4}", i + 1);
        let galley = painter.layout_no_wrap(
            line_num,
            egui::FontId::monospace(font_size * 0.85),
            Color32::from_gray(120),
        );
        painter.galley(egui::pos2(origin.x, y), galley, Color32::WHITE);
    }
}

fn handle_text_input(app: &mut RustpadApp, ctx: &egui::Context) {
    ctx.input(|i| {
        if i.key_pressed(egui::Key::Enter) {
            let indent_engine = IndentEngine::new(crate::editor::indent::IndentConfig {
                use_spaces: app.config.editor.tab_size > 0,
                tab_size: app.config.editor.tab_size,
                auto_indent: app.config.editor.auto_indent,
                smart_indent: true,
            });

            let prev_line = app.tab_manager.active().buffer.line(app.tab_manager.active().cursor.line).unwrap_or_default();
            let (indent, _) = indent_engine.newline_indent(prev_line);

            let line = app.tab_manager.active().cursor.line;
            let col = app.tab_manager.active().cursor.col;
            let pos = app.tab_manager.active().buffer.char_pos_for_line_col(line, col);
            let newline = "\n".to_string() + &indent;
            app.tab_manager.active_mut().buffer.insert_str(pos, &newline);
            app.tab_manager.active_mut().cursor.line += 1;
            app.tab_manager.active_mut().cursor.col = indent.chars().count();
            app.highlighter.invalidate_all();
            return;
        }

        if i.key_pressed(egui::Key::Backspace) {
            let line = app.tab_manager.active().cursor.line;
            let col = app.tab_manager.active().cursor.col;
            if col > 0 {
                let pos = app.tab_manager.active().buffer.char_pos_for_line_col(line, col);
                app.tab_manager.active_mut().buffer.delete_range(pos - 1, pos);
                app.tab_manager.active_mut().cursor.col -= 1;
                app.highlighter.invalidate_all();
            } else if line > 0 {
                let prev_line_len = app.tab_manager.active().buffer.line_len(line - 1);
                let pos = app.tab_manager.active().buffer.char_pos_for_line_col(line, 0);
                if pos > 0 {
                    app.tab_manager.active_mut().buffer.delete_range(pos - 1, pos);
                    app.tab_manager.active_mut().cursor.line -= 1;
                    app.tab_manager.active_mut().cursor.col = prev_line_len;
                    app.highlighter.invalidate_all();
                }
            }
            return;
        }

        if i.key_pressed(egui::Key::Delete) {
            let line = app.tab_manager.active().cursor.line;
            let col = app.tab_manager.active().cursor.col;
            let pos = app.tab_manager.active().buffer.char_pos_for_line_col(line, col);
            if pos < app.tab_manager.active().buffer.len() {
                app.tab_manager.active_mut().buffer.delete_range(pos, pos + 1);
                app.highlighter.invalidate_all();
            }
            return;
        }

        if i.key_pressed(egui::Key::Tab) {
            let tab_size = app.config.editor.tab_size;
            let line = app.tab_manager.active().cursor.line;
            let col = app.tab_manager.active().cursor.col;
            let pos = app.tab_manager.active().buffer.char_pos_for_line_col(line, col);
            let spaces = " ".repeat(tab_size);
            app.tab_manager.active_mut().buffer.insert_str(pos, &spaces);
            app.tab_manager.active_mut().cursor.col += tab_size;
            app.highlighter.invalidate_all();
            return;
        }

        if i.key_pressed(egui::Key::ArrowUp) {
            if app.tab_manager.active().cursor.line > 0 {
                app.tab_manager.active_mut().cursor.line -= 1;
                let max_col = app.tab_manager.active().buffer.line_len(app.tab_manager.active().cursor.line);
                app.tab_manager.active_mut().cursor.col = app.tab_manager.active().cursor.col.min(max_col);
            }
            return;
        }
        if i.key_pressed(egui::Key::ArrowDown) {
            let line_count = app.tab_manager.active().line_count();
            if line_count > 0 && app.tab_manager.active().cursor.line < line_count - 1 {
                app.tab_manager.active_mut().cursor.line += 1;
                let max_col = app.tab_manager.active().buffer.line_len(app.tab_manager.active().cursor.line);
                app.tab_manager.active_mut().cursor.col = app.tab_manager.active().cursor.col.min(max_col);
            }
            return;
        }
        if i.key_pressed(egui::Key::ArrowLeft) {
            if app.tab_manager.active().cursor.col > 0 {
                app.tab_manager.active_mut().cursor.col -= 1;
            } else if app.tab_manager.active().cursor.line > 0 {
                app.tab_manager.active_mut().cursor.line -= 1;
                app.tab_manager.active_mut().cursor.col = app.tab_manager.active().buffer.line_len(app.tab_manager.active().cursor.line);
            }
            return;
        }
        if i.key_pressed(egui::Key::ArrowRight) {
            let max_col = app.tab_manager.active().buffer.line_len(app.tab_manager.active().cursor.line);
            let line_count = app.tab_manager.active().line_count();
            if app.tab_manager.active().cursor.col < max_col {
                app.tab_manager.active_mut().cursor.col += 1;
            } else if line_count > 0 && app.tab_manager.active().cursor.line < line_count - 1 {
                app.tab_manager.active_mut().cursor.line += 1;
                app.tab_manager.active_mut().cursor.col = 0;
            }
            return;
        }

        for event in &i.events {
            if let egui::Event::Text(text) = event {
                if text.chars().all(|c| !c.is_control()) {
                    let line = app.tab_manager.active().cursor.line;
                    let col = app.tab_manager.active().cursor.col;
                    let pos = app.tab_manager.active().buffer.char_pos_for_line_col(line, col);
                    app.tab_manager.active_mut().buffer.insert_str(pos, text);
                    app.tab_manager.active_mut().cursor.col += text.chars().count();
                    app.highlighter.invalidate_all();
                }
            }
        }
    });
}