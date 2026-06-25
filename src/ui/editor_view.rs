use eframe::egui;
use eframe::epaint::Color32;

use crate::app::RustpadApp;
use crate::editor::context_actions::{self, mark_line_color};
use crate::editor::fold::FoldState;
use crate::editor::Cursor;
use crate::ui::scroll_bar;

const LINE_NUMBER_FONT_SIZE: f32 = 14.0;
const CONTENT_EXTENT_LINE_WIDTH: f32 = 2.0;
/// Gap between line-number gutter (orange extent line) and the editor pane.
const GUTTER_EDITOR_GAP: f32 = 2.0;

#[derive(Clone, Copy)]
struct EditorDisplayOpts {
    font_size: f32,
    display_blank: bool,
    show_tabs_as_spaces: bool,
    display_non_print: bool,
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

struct ColumnMapper<'a> {
    painter: &'a egui::Painter,
    opts: EditorDisplayOpts,
}

impl Copy for ColumnMapper<'_> {}

impl Clone for ColumnMapper<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> ColumnMapper<'a> {
    fn new(painter: &'a egui::Painter, opts: EditorDisplayOpts) -> Self {
        Self { painter, opts }
    }

    fn col_to_x(&self, base_x: f32, line_text: &str, col: usize) -> f32 {
        let prefix: String = line_text.chars().take(col).collect();
        let display = display_line_text(
            &prefix,
            self.opts.display_blank,
            self.opts.show_tabs_as_spaces,
            self.opts.display_non_print,
        );
        if display.is_empty() {
            return base_x;
        }
        let galley = self.painter.layout_no_wrap(
            display,
            egui::FontId::monospace(self.opts.font_size),
            Color32::TRANSPARENT,
        );
        base_x + galley.size().x
    }

    fn x_to_col(&self, base_x: f32, line_text: &str, rel_x: f32) -> usize {
        let line_len = line_text.chars().count();
        if line_len == 0 {
            return 0;
        }

        let target_x = base_x + rel_x.max(0.0);
        let mut lo = 0usize;
        let mut hi = line_len;

        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if self.col_to_x(base_x, line_text, mid) < target_x {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }

        if lo > 0 {
            let x_prev = self.col_to_x(base_x, line_text, lo - 1);
            let x_lo = self.col_to_x(base_x, line_text, lo);
            if target_x - x_prev < x_lo - target_x {
                return lo - 1;
            }
        }
        lo.min(line_len)
    }

    fn pointer_to_cursor(
        &self,
        pointer_pos: egui::Pos2,
        editor_rect: egui::Rect,
        line: usize,
        line_text: &str,
    ) -> Cursor {
        let rel_x = pointer_pos.x - editor_rect.left();
        let col = self.x_to_col(editor_rect.left(), line_text, rel_x);
        Cursor::new(line, col)
    }
}

struct LineMarkDrawCtx<'a> {
    mapper: ColumnMapper<'a>,
    buffer: &'a crate::editor::TextBuffer,
    line_height: f32,
}

struct ContentExtentCtx<'a> {
    painter: &'a egui::Painter,
    fold_state: &'a FoldState,
    origin_y: f32,
    viewport_height: f32,
    line_count: usize,
    line_height: f32,
    scroll_offset: f32,
}

struct SearchHighlightCtx<'a> {
    app: &'a RustpadApp,
    fold_state: &'a FoldState,
    theme: &'a crate::config::theme::EditorTheme,
    mapper: ColumnMapper<'a>,
    start_pos: egui::Pos2,
    first_visible_display: usize,
    visible_lines_on_screen: usize,
    line_height: f32,
    current_scroll: f32,
}

/// Render the main editor view.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    // Hide the editor only when a diff is actually being shown. If diff mode is
    // toggled on but no result is ready yet, keep rendering the editor so the
    // central area never goes blank.
    if app.show_diff_view && app.diff_result.is_some() {
        return;
    }

    let mut central_frame = egui::Frame::central_panel(&ctx.style());
    central_frame.inner_margin.top = 0.0;

    egui::CentralPanel::default()
        .frame(central_frame)
        .show(ctx, |ui| {
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
        let gutter_editor_gap = if show_line_numbers {
            GUTTER_EDITOR_GAP
        } else {
            0.0
        };
        let panel_rect = ui.available_rect_before_wrap();
        let layout_top = ui.max_rect().top();
        let content_bottom = panel_rect.bottom() - scroll_bar::BAR_BOTTOM_SAFE_INSET;
        let row_height = (content_bottom - layout_top).max(0.0);
        let line_num_width = if show_line_numbers {
            gutter_width + gutter_editor_gap
        } else {
            0.0
        };

        let document_lines: Vec<&str> = text.lines().collect();
        let minimap_enabled = app.minimap.enabled;
        let silhouette_content_width = if minimap_enabled {
            scroll_bar::compute_silhouette_content_width(&document_lines, app.minimap.width)
        } else {
            0.0
        };
        let strip_width = scroll_bar::quick_scroll_strip_width(
            minimap_enabled,
            silhouette_content_width,
        );

        let fold_rect = egui::Rect::from_min_size(
            egui::pos2(panel_rect.left(), layout_top),
            egui::vec2(FOLD_GUTTER_WIDTH, row_height),
        );
        let line_rect = egui::Rect::from_min_size(
            egui::pos2(fold_rect.right(), layout_top),
            egui::vec2(line_num_width, row_height),
        );
        let strip_rect = egui::Rect::from_min_max(
            egui::pos2(panel_rect.right() - strip_width, layout_top),
            egui::pos2(panel_rect.right(), content_bottom),
        );
        let scroll_rect = egui::Rect::from_min_max(
            strip_rect.left_top(),
            egui::pos2(strip_rect.left() + scroll_bar::QUICK_SCROLL_WIDTH, strip_rect.bottom()),
        );
        let silhouette_rect = if minimap_enabled {
            egui::Rect::from_min_max(
                scroll_rect.right_top(),
                strip_rect.right_bottom(),
            )
        } else {
            egui::Rect::NOTHING
        };
        let editor_rect = egui::Rect::from_min_max(
            egui::pos2(line_rect.right(), layout_top),
            egui::pos2(strip_rect.left(), content_bottom),
        );
        let available_width = editor_rect.width().max(0.0);
        let available_height = row_height;

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

        let editor_theme = app.theme_manager.current_theme().clone();
        let bg_color = editor_theme.background_color();
        let current_line_bg = editor_theme.current_line_bg_color();
        let selection_bg = editor_theme.selection_bg_color();
        let cursor_color = editor_theme.cursor_color();
        let gutter_bg = crate::config::theme::EditorTheme::to_color32(editor_theme.gutter_bg);
        let gutter_fg = crate::config::theme::EditorTheme::to_color32(editor_theme.line_number_fg);

        if let Some(start_line) = draw_fold_gutter(
                ui,
                fold_rect,
                line_count,
                scroll_offset,
                line_height,
                font_size,
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
                    line_rect,
                    line_count,
                    scroll_offset,
                    line_height,
                    gutter_width,
                    gutter_bg,
                    gutter_fg,
                    &line_marks,
                    &app.tab_manager.active().editor_extras.fold_state,
                );
            }

            let fold_state = app.tab_manager.active().editor_extras.fold_state.clone();
            let response = ui.allocate_rect(editor_rect, egui::Sense::click_and_drag());
            let painter = ui.painter();
            let column_mapper = ColumnMapper::new(
                painter,
                EditorDisplayOpts {
                    font_size,
                    display_blank,
                    show_tabs_as_spaces,
                    display_non_print,
                },
            );

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
                    let click_cursor = column_mapper.pointer_to_cursor(
                        pos,
                        editor_rect,
                        click_line,
                        line_text,
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

            // Focus management: restore focus from last frame (never steal from dialogs
            // or the toolbar font-size field).
            if app.editor_has_focus && !modal_blocks_focus && !app.toolbar_font_size_editing {
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
                    let cursor = column_mapper.pointer_to_cursor(
                        pointer_pos,
                        editor_rect,
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
                    let cursor = column_mapper.pointer_to_cursor(
                        pointer_pos,
                        editor_rect,
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
                            &fold_state,
                        );
                        let line_text = app
                            .tab_manager
                            .active()
                            .buffer
                            .line(click_line)
                            .unwrap_or_default();
                        let cursor = column_mapper.pointer_to_cursor(
                            pointer_pos,
                            editor_rect,
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

            // Auto-focus on first frame (not while a dialog or toolbar font field is open).
            if !modal_blocks_focus
                && !app.toolbar_font_size_editing
                && ctx.memory(|m| m.focused().is_none())
            {
                response.request_focus();
            }

            // Store focus state for next frame (toolbar font field owns the keyboard).
            let has_focus = response.has_focus();
            if app.toolbar_font_size_editing {
                app.editor_has_focus = false;
            } else {
                app.editor_has_focus = has_focus;
            }

            // Handle scroll (wheel); do not fight with auto-scroll-to-cursor below.
            let visible_line_total = fold_state.visible_line_count(line_count);
            let max_scroll =
                scroll_bar::max_scroll_offset(visible_line_total, line_height, available_height);

            if response.hovered() {
                let scroll_delta = ui.input(|i| {
                    if i.smooth_scroll_delta.y != 0.0 {
                        i.smooth_scroll_delta.y
                    } else {
                        i.raw_scroll_delta.y
                    }
                });
                if scroll_delta != 0.0 {
                    let tab = app.tab_manager.active_mut();
                    scroll_bar::apply_wheel_scroll(
                        &mut tab.scroll_offset,
                        scroll_delta,
                        max_scroll,
                    );
                    tab.last_auto_scroll_cursor = tab.cursor;
                    scroll_bar::consume_editor_wheel(ctx, true);
                } else if response.hovered() {
                    scroll_bar::consume_editor_wheel(ctx, false);
                }
            }

            let start_pos = editor_rect.left_top();
            let current_scroll = app.tab_manager.active().scroll_offset;
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
                    let x1 = column_mapper.col_to_x(start_pos.x, line_text, sel_start_col);
                    let x2 = column_mapper.col_to_x(start_pos.x, line_text, sel_end_col);
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

            // Search match backgrounds (under text, Notepad++ style).
            if crate::ui::search_highlight::should_paint_editor_highlights(
                app.show_search,
                app.show_search_results,
                app.search_pattern.is_empty(),
                app.search_engine.results().len(),
            ) {
                draw_search_match_highlights(&SearchHighlightCtx {
                    app,
                    fold_state: &fold_state,
                    theme: &editor_theme,
                    mapper: column_mapper,
                    start_pos,
                    first_visible_display,
                    visible_lines_on_screen,
                    line_height,
                    current_scroll,
                });
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
                    &LineMarkDrawCtx {
                        mapper: column_mapper,
                        buffer: &app.tab_manager.active().buffer,
                        line_height,
                    },
                    text_marks,
                    i,
                    start_pos.x,
                    y,
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
                    let cursor_x =
                        column_mapper.col_to_x(start_pos.x, line_text, cursor_col);

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

            // Auto-scroll only when the cursor moves (typing, clicks, arrows) — not every
            // frame, otherwise wheel scrolling jitters as the view snaps back to the cursor.
            let cursor_for_scroll = app.tab_manager.active().cursor;
            if app.tab_manager.active().last_auto_scroll_cursor != cursor_for_scroll {
                let tab = app.tab_manager.active_mut();
                ensure_cursor_visible(
                    &mut tab.scroll_offset,
                    cursor_for_scroll.line,
                    line_height,
                    available_height,
                    &fold_state,
                );
                tab.last_auto_scroll_cursor = cursor_for_scroll;
            }

            scroll_bar::show_quick_scroll_bar(
                ui,
                app,
                scroll_rect,
                line_height,
                visible_line_total,
                &fold_state,
                gutter_bg,
            );

            if minimap_enabled {
                scroll_bar::paint_document_silhouette(
                    ui,
                    ctx,
                    app,
                    silhouette_rect,
                    &document_lines,
                    app.tab_manager.active().scroll_offset,
                    visible_line_total,
                    line_height,
                    app.minimap.width,
                    gutter_bg,
                );
            }
    });

    crate::ui::editor_context_menu::show(app, ctx);
}

fn line_y_in_viewport(y: f32, viewport_top: f32, viewport_height: f32, line_height: f32) -> bool {
    let rel = y - viewport_top;
    rel >= -line_height && rel <= viewport_height
}

/// Foreground/syntax text color is unchanged; this only fills behind the glyphs.
fn draw_line_mark_backgrounds(
    ctx: &LineMarkDrawCtx<'_>,
    text_marks: &[crate::editor::context_actions::TextMark],
    line: usize,
    start_x: f32,
    y: f32,
) {
    let painter = ctx.mapper.painter;
    let line_height = ctx.line_height;
    let buffer = ctx.buffer;
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
        let x1 = ctx.mapper.col_to_x(start_x, line_text, sel_start_col);
        let x2 = ctx.mapper.col_to_x(start_x, line_text, sel_end_col);
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
    *scroll_offset = scroll_offset.round();
}

#[allow(clippy::too_many_arguments)]
fn draw_fold_gutter(
    ui: &mut egui::Ui,
    gutter_rect: egui::Rect,
    line_count: usize,
    scroll_offset: f32,
    line_height: f32,
    font_size: f32,
    gutter_bg: Color32,
    fold_state: &FoldState,
) -> Option<usize> {
    ui.allocate_rect(gutter_rect, egui::Sense::hover());

    let origin = gutter_rect.left_top();
    let available_height = gutter_rect.height();

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
fn draw_content_extent_line(x: f32, ctx: &ContentExtentCtx<'_>) {
    let Some((y_start, y_end)) = content_extent_y_range(
        ctx.origin_y,
        ctx.viewport_height,
        ctx.line_count,
        ctx.line_height,
        ctx.scroll_offset,
        ctx.fold_state,
    ) else {
        return;
    };
    let orange = Color32::from_rgb(255, 140, 0);
    ctx.painter.rect_filled(
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
    gutter_rect: egui::Rect,
    line_count: usize,
    scroll_offset: f32,
    line_height: f32,
    gutter_width: f32,
    gutter_bg: Color32,
    gutter_fg: Color32,
    line_marks: &std::collections::HashMap<usize, u8>,
    fold_state: &FoldState,
) {
    let visible_line_total = fold_state.visible_line_count(line_count);
    let first_visible_display = (scroll_offset / line_height) as usize;
    let available_height = gutter_rect.height();
    let visible_lines_on_screen = (available_height / line_height) as usize + 2;

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
        let num_x = origin.x + gutter_width - galley.size().x - 2.0 - GUTTER_EDITOR_GAP;
        painter.galley(egui::pos2(num_x, y), galley, Color32::TRANSPARENT);
    }

    // Orange content-extent line (Notepad++ style): from line 1 to last line of content.
    draw_content_extent_line(
        origin.x + gutter_width - GUTTER_EDITOR_GAP,
        &ContentExtentCtx {
            painter,
            fold_state,
            origin_y: origin.y,
            viewport_height: available_height,
            line_count,
            line_height,
            scroll_offset,
        },
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

fn draw_search_match_highlights(ctx: &SearchHighlightCtx<'_>) {
    let match_bg = ctx.theme.search_highlight_bg_color();
    let current_bg = ctx.theme.search_current_highlight_bg_color();
    let current_idx = ctx.app.search_engine.current_index();
    let results: Vec<crate::search::SearchMatch> = ctx.app.search_engine.results().to_vec();
    let painter = ctx.mapper.painter;

    for (mi, m) in results.iter().enumerate() {
        let (s_line, s_col) = ctx
            .app
            .tab_manager
            .active()
            .buffer
            .line_col_for_char_pos(m.start);
        let (e_line, e_col) = ctx
            .app
            .tab_manager
            .active()
            .buffer
            .line_col_for_char_pos(m.end);
        let color = if Some(mi) == current_idx {
            current_bg
        } else {
            match_bg
        };

        for line_idx in s_line..=e_line {
            if ctx.fold_state.is_line_hidden(line_idx) {
                continue;
            }
            let Some(vis) = ctx.fold_state.visible_line_index(line_idx) else {
                continue;
            };
            if vis < ctx.first_visible_display
                || vis >= ctx.first_visible_display + ctx.visible_lines_on_screen
            {
                continue;
            }
            let y = FoldState::visible_line_y(
                ctx.start_pos.y,
                vis,
                ctx.line_height,
                ctx.current_scroll,
            );
            let line_text = ctx
                .app
                .tab_manager
                .active()
                .buffer
                .line(line_idx)
                .unwrap_or_default();
            let col_start = if line_idx == s_line { s_col } else { 0 };
            let col_end = if line_idx == e_line {
                e_col
            } else {
                ctx.app.tab_manager.active().buffer.line_len(line_idx)
            };
            let x1 = ctx
                .mapper
                .col_to_x(ctx.start_pos.x, line_text, col_start);
            let x2 = ctx.mapper.col_to_x(ctx.start_pos.x, line_text, col_end);
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(x1, y),
                    egui::pos2(x2.max(x1 + 1.0), y + ctx.line_height),
                ),
                0.0,
                color,
            );
        }
    }
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
