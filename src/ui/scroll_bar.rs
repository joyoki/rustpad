//! Narrow quick-scroll strip on the right edge of the editor (Notepad++ style).

use eframe::egui;
use eframe::epaint::{Color32, Shape, Stroke};

use crate::app::RustpadApp;
use crate::editor::fold::FoldState;

pub const QUICK_SCROLL_WIDTH: f32 = 16.0;
/// Height of each line bar in the document silhouette strip.
const SILHOUETTE_LINE_HEIGHT: f32 = 2.0;
/// Minimum height of the viewport band in the silhouette.
const SILHOUETTE_MIN_VIEW_HEIGHT: f32 = 24.0;
const ARROW_BTN_HEIGHT: f32 = 14.0;
/// Gap between the scroll track and the arrow buttons (boundary buffer).
const TRACK_END_PADDING: f32 = 3.0;
/// Extra inset so the bottom arrow stays inside the paintable area (macOS rounded corners).
#[cfg(target_os = "macos")]
pub const BAR_BOTTOM_SAFE_INSET: f32 = 6.0;
#[cfg(not(target_os = "macos"))]
pub const BAR_BOTTOM_SAFE_INSET: f32 = 0.0;
/// Minimum thumb height so the handle is easy to grab.
const MIN_THUMB_HEIGHT: f32 = 48.0;
/// Thumb is at least this fraction of the track height.
const MIN_THUMB_TRACK_RATIO: f32 = 0.15;

struct ScrollTrackLayout {
    track_rect: egui::Rect,
    up_btn_rect: egui::Rect,
    down_btn_rect: egui::Rect,
    thumb_height: f32,
    thumb_travel: f32,
}

/// Maximum scroll offset in pixels for the current document.
pub fn max_scroll_offset(
    visible_line_total: usize,
    line_height: f32,
    viewport_height: f32,
) -> f32 {
    let content_height = visible_line_total as f32 * line_height;
    (content_height - viewport_height).max(0.0)
}

fn effective_bar_rect(bar_rect: egui::Rect, clip: egui::Rect) -> egui::Rect {
    bar_rect.intersect(clip)
}

fn compute_track_layout(
    bar_rect: egui::Rect,
    clip: egui::Rect,
    content_height: f32,
    viewport_height: f32,
) -> ScrollTrackLayout {
    let up_btn_rect = egui::Rect::from_min_size(
        bar_rect.left_top(),
        egui::vec2(bar_rect.width(), ARROW_BTN_HEIGHT),
    );
    // Keep the bottom arrow fully inside the paintable clip (macOS rounded window corners).
    let down_bottom = (bar_rect.bottom() - BAR_BOTTOM_SAFE_INSET).min(clip.bottom() - 1.0);
    let down_top = (down_bottom - ARROW_BTN_HEIGHT).max(up_btn_rect.bottom());
    let down_btn_rect = egui::Rect::from_min_max(
        egui::pos2(bar_rect.left(), down_top),
        egui::pos2(bar_rect.right(), down_bottom),
    );
    let track_top = up_btn_rect.bottom() + TRACK_END_PADDING;
    let track_bottom = down_btn_rect.top() - TRACK_END_PADDING;
    let track_height = (track_bottom - track_top).max(0.0);
    let track_rect = egui::Rect::from_min_max(
        egui::pos2(bar_rect.left(), track_top),
        egui::pos2(bar_rect.right(), track_bottom),
    );

    let thumb_height = if content_height > viewport_height && track_height > 0.0 {
        let proportional = track_height * (viewport_height / content_height);
        let min_by_ratio = track_height * MIN_THUMB_TRACK_RATIO;
        proportional
            .max(MIN_THUMB_HEIGHT)
            .max(min_by_ratio)
            .min(track_height)
    } else {
        track_height
    };
    let thumb_travel = (track_height - thumb_height).max(0.0);

    ScrollTrackLayout {
        track_rect,
        up_btn_rect,
        down_btn_rect,
        thumb_height,
        thumb_travel,
    }
}

/// Apply mouse-wheel delta to a scroll offset, clamped to document bounds.
pub fn apply_wheel_scroll(
    scroll_offset: &mut f32,
    delta_y: f32,
    max_scroll: f32,
) {
    if delta_y == 0.0 {
        return;
    }
    *scroll_offset = (*scroll_offset - delta_y).clamp(0.0, max_scroll);
    *scroll_offset = scroll_offset.round();
}

/// Prevent trackpad horizontal deltas and consumed vertical scroll from bubbling
/// to parent panels (a common source of left-right layout jitter while scrolling).
pub fn consume_editor_wheel(ctx: &egui::Context, consumed_vertical: bool) {
    ctx.input_mut(|i| {
        i.smooth_scroll_delta.x = 0.0;
        i.raw_scroll_delta.x = 0.0;
        if consumed_vertical {
            i.smooth_scroll_delta.y = 0.0;
            i.raw_scroll_delta.y = 0.0;
        }
    });
}

fn wheel_delta_y(ui: &egui::Ui) -> f32 {
    ui.input(|i| {
        if i.smooth_scroll_delta.y != 0.0 {
            i.smooth_scroll_delta.y
        } else {
            i.raw_scroll_delta.y
        }
    })
}

fn thumb_top_from_scroll(
    scroll_offset: f32,
    track_top: f32,
    thumb_travel: f32,
    max_scroll: f32,
) -> f32 {
    if max_scroll <= 0.0 || thumb_travel <= 0.0 {
        return track_top;
    }
    track_top + (scroll_offset / max_scroll).clamp(0.0, 1.0) * thumb_travel
}

fn silhouette_bar_width(line: &str, cap: f32) -> f32 {
    (line.len() as f32 * 0.5).min(cap)
}

/// Width of the document silhouette content (gray bars), capped by `max_width`.
pub fn compute_silhouette_content_width(lines: &[&str], max_width: f32) -> f32 {
    lines
        .iter()
        .map(|line| silhouette_bar_width(line, max_width))
        .fold(0.0_f32, f32::max)
        .max(1.0)
}

/// Total width of the quick-scroll strip (scrollbar + optional silhouette).
pub fn quick_scroll_strip_width(minimap_enabled: bool, silhouette_content_width: f32) -> f32 {
    if minimap_enabled {
        QUICK_SCROLL_WIDTH + silhouette_content_width
    } else {
        QUICK_SCROLL_WIDTH
    }
}

/// Viewport indicator `(top, height)` inside a vertical strip (minimap / scrollbar).
pub fn viewport_indicator_in_strip(
    strip_height: f32,
    scroll_offset: f32,
    visible_line_total: usize,
    line_height: f32,
    editor_viewport_height: f32,
    min_indicator_height: f32,
) -> (f32, f32) {
    let content_height = visible_line_total as f32 * line_height;
    let max_scroll = max_scroll_offset(visible_line_total, line_height, editor_viewport_height);
    if strip_height <= 0.0 || content_height <= editor_viewport_height {
        return (0.0, strip_height.max(0.0));
    }
    let indicator_height = (strip_height * (editor_viewport_height / content_height))
        .max(min_indicator_height)
        .min(strip_height);
    let indicator_travel = strip_height - indicator_height;
    let top = thumb_top_from_scroll(scroll_offset, 0.0, indicator_travel, max_scroll);
    (top, indicator_height)
}

fn scroll_from_thumb_top(
    thumb_top: f32,
    track_top: f32,
    thumb_travel: f32,
    max_scroll: f32,
) -> f32 {
    if max_scroll <= 0.0 || thumb_travel <= 0.0 {
        return 0.0;
    }
    let rel = ((thumb_top - track_top) / thumb_travel).clamp(0.0, 1.0);
    rel * max_scroll
}

fn paint_arrow(painter: &egui::Painter, rect: egui::Rect, up: bool, color: Color32) {
    let c = rect.center();
    let half_h = rect.height() * 0.22;
    let half_w = rect.width() * 0.28;
    let points = if up {
        vec![
            egui::pos2(c.x, c.y - half_h),
            egui::pos2(c.x - half_w, c.y + half_h),
            egui::pos2(c.x + half_w, c.y + half_h),
        ]
    } else {
        vec![
            egui::pos2(c.x, c.y + half_h),
            egui::pos2(c.x - half_w, c.y - half_h),
            egui::pos2(c.x + half_w, c.y - half_h),
        ]
    };
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

fn paint_arrow_button(
    painter: &egui::Painter,
    rect: egui::Rect,
    up: bool,
    btn_bg: Color32,
    border: Color32,
    arrow_color: Color32,
) {
    painter.rect_filled(rect, 0.0, btn_bg);
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, border));
    paint_arrow(painter, rect, up, arrow_color);
}

/// Scroll so the cursor line is visible (same logic as the main editor).
pub fn scroll_to_cursor(
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
pub fn show_quick_scroll_bar(
    ui: &mut egui::Ui,
    app: &mut RustpadApp,
    bar_rect: egui::Rect,
    line_height: f32,
    visible_line_total: usize,
    fold_state: &FoldState,
    gutter_bg: Color32,
) {
    let t = app.tr();
    let ctx = ui.ctx().clone();
    let clip = ui.clip_rect();
    let bar_rect = effective_bar_rect(bar_rect, clip);
    let viewport_height = bar_rect.height();
    let max_scroll = max_scroll_offset(visible_line_total, line_height, viewport_height);
    let content_height = visible_line_total as f32 * line_height;

    let layout = compute_track_layout(bar_rect, clip, content_height, viewport_height);
    let track = layout.track_rect;
    let track_top = track.top();
    let track_bottom = track.bottom();

    let track_response = ui.allocate_rect(bar_rect, egui::Sense::click_and_drag());

    let scroll_offset = app.tab_manager.active().scroll_offset;
    let thumb_top =
        thumb_top_from_scroll(scroll_offset, track_top, layout.thumb_travel, max_scroll);
    let thumb_rect = egui::Rect::from_min_size(
        egui::pos2(track.left(), thumb_top),
        egui::vec2(track.width(), layout.thumb_height),
    );

    let tab_id = app.tab_manager.active().id;
    let grab_id = ui.id().with(("quick_scroll_grab", tab_id));
    let thumb_id = ui.id().with(("quick_scroll_thumb", tab_id));

    let thumb_response = ui.interact(thumb_rect, thumb_id, egui::Sense::click_and_drag());
    let up_response = ui.interact(layout.up_btn_rect, ui.id().with(("scroll_up", tab_id)), egui::Sense::click());
    let down_response = ui.interact(
        layout.down_btn_rect,
        ui.id().with(("scroll_down", tab_id)),
        egui::Sense::click(),
    );

    let painter = ui.painter();
    let btn_bg = ctx.style().visuals.widgets.inactive.bg_fill;
    let buffer_bg = ctx.style().visuals.panel_fill;
    let track_bg = gutter_bg;
    let border_color = ctx.style().visuals.widgets.noninteractive.bg_stroke.color;
    let arrow_color = ctx.style().visuals.strong_text_color();

    painter.rect_filled(bar_rect, 0.0, buffer_bg);
    painter.rect_filled(track, 0.0, track_bg);
    // Boundary buffer zones between arrow buttons and track.
    if TRACK_END_PADDING > 0.0 {
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(bar_rect.left(), layout.up_btn_rect.bottom()),
                egui::pos2(bar_rect.right(), track_top),
            ),
            0.0,
            buffer_bg,
        );
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(bar_rect.left(), track_bottom),
                egui::pos2(bar_rect.right(), layout.down_btn_rect.top()),
            ),
            0.0,
            buffer_bg,
        );
    }

    let thumb_color = if thumb_response.hovered() || thumb_response.dragged() {
        Color32::from_gray(120)
    } else {
        Color32::from_gray(150)
    };
    painter.rect_filled(thumb_rect, 1.0, thumb_color);
    // Draw arrow buttons on top so they are never covered by the thumb.
    paint_arrow_button(
        painter,
        layout.up_btn_rect,
        true,
        btn_bg,
        border_color,
        arrow_color,
    );
    paint_arrow_button(
        painter,
        layout.down_btn_rect,
        false,
        btn_bg,
        border_color,
        arrow_color,
    );

    let can_scroll = max_scroll > 0.0 && layout.thumb_travel > 0.0;

    if up_response.clicked() {
        let tab = app.tab_manager.active_mut();
        tab.scroll_offset = (tab.scroll_offset - line_height).max(0.0).round();
        tab.last_auto_scroll_cursor = tab.cursor;
        ctx.request_repaint();
    }
    if down_response.clicked() {
        let tab = app.tab_manager.active_mut();
        tab.scroll_offset = (tab.scroll_offset + line_height).min(max_scroll).round();
        tab.last_auto_scroll_cursor = tab.cursor;
        ctx.request_repaint();
    }

    if can_scroll {
        if thumb_response.drag_started() {
            if let Some(pos) = thumb_response.interact_pointer_pos() {
                let grab = pos.y - thumb_top;
                ctx.data_mut(|d| d.insert_temp(grab_id, grab));
            }
        }

        if thumb_response.dragged() {
            if let Some(pos) = thumb_response.interact_pointer_pos() {
                let grab = ctx.data_mut(|d| *d.get_temp_mut_or_default::<f32>(grab_id));
                // Clamp thumb inside track only — never overlaps arrow buttons.
                let new_thumb_top =
                    (pos.y - grab).clamp(track_top, track_top + layout.thumb_travel);
                let tab = app.tab_manager.active_mut();
                tab.scroll_offset = scroll_from_thumb_top(
                    new_thumb_top,
                    track_top,
                    layout.thumb_travel,
                    max_scroll,
                );
                tab.last_auto_scroll_cursor = tab.cursor;
                ctx.request_repaint();
            }
        }

        if thumb_response.drag_stopped() {
            ctx.data_mut(|d| d.remove::<f32>(grab_id));
            let tab = app.tab_manager.active_mut();
            tab.scroll_offset = tab.scroll_offset.round();
        }

        // Click on track (outside thumb): jump within track bounds.
        if track_response.clicked()
            && !thumb_response.clicked()
            && !up_response.clicked()
            && !down_response.clicked()
        {
            if let Some(pos) = track_response.interact_pointer_pos() {
                if pos.y >= track_top && pos.y <= track_bottom {
                    let center_y = pos.y.clamp(
                        track_top + layout.thumb_height * 0.5,
                        track_bottom - layout.thumb_height * 0.5,
                    );
                    let new_thumb_top = center_y - layout.thumb_height * 0.5;
                    let tab = app.tab_manager.active_mut();
                    tab.scroll_offset = scroll_from_thumb_top(
                        new_thumb_top,
                        track_top,
                        layout.thumb_travel,
                        max_scroll,
                    )
                    .round();
                    tab.last_auto_scroll_cursor = tab.cursor;
                }
            }
        }
    }

    if thumb_response.hovered() || thumb_response.dragged() {
        ctx.set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }
    up_response.on_hover_text(t.scroll_up);
    down_response.on_hover_text(t.scroll_down);

    if track_response.hovered() || thumb_response.hovered() {
        let delta = wheel_delta_y(ui);
        if delta != 0.0 {
            let tab = app.tab_manager.active_mut();
            apply_wheel_scroll(&mut tab.scroll_offset, delta, max_scroll);
            tab.last_auto_scroll_cursor = tab.cursor;
            consume_editor_wheel(&ctx, true);
        } else {
            consume_editor_wheel(&ctx, false);
        }
    }

    track_response.context_menu(|ui| {
        if ui.button(t.scroll_to_here).clicked() {
            let tab = app.tab_manager.active_mut();
            let cursor_line = tab.cursor.line;
            scroll_to_cursor(
                &mut tab.scroll_offset,
                cursor_line,
                line_height,
                viewport_height,
                fold_state,
            );
            tab.last_auto_scroll_cursor = tab.cursor;
            ui.close_menu();
        }
        ui.separator();
        if ui.button(t.scroll_top).clicked() {
            let tab = app.tab_manager.active_mut();
            tab.scroll_offset = 0.0;
            tab.last_auto_scroll_cursor = tab.cursor;
            ui.close_menu();
        }
        if ui.button(t.scroll_bottom).clicked() {
            let tab = app.tab_manager.active_mut();
            tab.scroll_offset = max_scroll;
            tab.last_auto_scroll_cursor = tab.cursor;
            ui.close_menu();
        }
        ui.separator();
        if ui.button(t.scroll_page_up).clicked() {
            let tab = app.tab_manager.active_mut();
            tab.scroll_offset = (tab.scroll_offset - viewport_height).max(0.0);
            tab.last_auto_scroll_cursor = tab.cursor;
            ui.close_menu();
        }
        if ui.button(t.scroll_page_down).clicked() {
            let tab = app.tab_manager.active_mut();
            tab.scroll_offset = (tab.scroll_offset + viewport_height).min(max_scroll);
            tab.last_auto_scroll_cursor = tab.cursor;
            ui.close_menu();
        }
        ui.separator();
        if ui.button(t.scroll_up).clicked() {
            let tab = app.tab_manager.active_mut();
            tab.scroll_offset = (tab.scroll_offset - line_height).max(0.0);
            tab.last_auto_scroll_cursor = tab.cursor;
            ui.close_menu();
        }
        if ui.button(t.scroll_down).clicked() {
            let tab = app.tab_manager.active_mut();
            tab.scroll_offset = (tab.scroll_offset + line_height).min(max_scroll);
            tab.last_auto_scroll_cursor = tab.cursor;
            ui.close_menu();
        }
    });

    track_response.on_hover_text(t.scroll_to_here);
}

/// Document silhouette + viewport band, drawn flush to the right of the quick scrollbar.
#[allow(clippy::too_many_arguments)]
pub fn paint_document_silhouette(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    app: &mut RustpadApp,
    rect: egui::Rect,
    lines: &[&str],
    scroll_offset: f32,
    visible_line_total: usize,
    editor_line_height: f32,
    max_bar_width: f32,
    bg: Color32,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let available_height = rect.height();
    let total_lines = lines.len().max(1);
    let content_width = compute_silhouette_content_width(lines, max_bar_width);
    let scale = available_height / (total_lines as f32 * SILHOUETTE_LINE_HEIGHT);

    let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
    let painter = ui.painter();
    let start = rect.left_top();

    painter.rect_filled(rect, 0.0, bg);

    for (i, line) in lines.iter().enumerate() {
        let y = start.y + i as f32 * SILHOUETTE_LINE_HEIGHT * scale;
        let width = silhouette_bar_width(line, max_bar_width);
        let color = if line.trim().is_empty() {
            Color32::TRANSPARENT
        } else {
            Color32::from_gray(120)
        };
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(start.x, y),
                egui::vec2(width, SILHOUETTE_LINE_HEIGHT * scale.max(0.5)),
            ),
            0.0,
            color,
        );
    }

    let (view_top, view_height) = viewport_indicator_in_strip(
        available_height,
        scroll_offset,
        visible_line_total,
        editor_line_height,
        available_height,
        SILHOUETTE_MIN_VIEW_HEIGHT,
    );
    let view_rect = egui::Rect::from_min_size(
        egui::pos2(start.x, start.y + view_top),
        egui::vec2(content_width, view_height),
    );
    painter.rect_filled(
        view_rect,
        0.0,
        Color32::from_rgba_premultiplied(100, 150, 255, 55),
    );
    painter.rect_stroke(
        view_rect,
        0.0,
        Stroke::new(1.0, Color32::from_rgb(100, 150, 255)),
    );

    if response.clicked() || response.dragged() {
        if let Some(click_pos) = response.interact_pointer_pos() {
            let rel_y = (click_pos.y - start.y).clamp(0.0, available_height);
            let max_scroll = max_scroll_offset(
                visible_line_total,
                editor_line_height,
                available_height,
            );
            let (_, indicator_height) = viewport_indicator_in_strip(
                available_height,
                scroll_offset,
                visible_line_total,
                editor_line_height,
                available_height,
                SILHOUETTE_MIN_VIEW_HEIGHT,
            );
            let indicator_travel = (available_height - indicator_height).max(0.0);
            let ratio = if indicator_travel > 0.0 {
                ((rel_y - indicator_height * 0.5) / indicator_travel).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let tab = app.tab_manager.active_mut();
            tab.scroll_offset = (ratio * max_scroll).round();
            tab.last_auto_scroll_cursor = tab.cursor;

            let line = ((rel_y / available_height) * total_lines as f32) as usize;
            tab.cursor.line = line.min(total_lines.saturating_sub(1));
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_scroll_is_zero_for_short_documents() {
        assert_eq!(max_scroll_offset(5, 20.0, 400.0), 0.0);
    }

    #[test]
    fn max_scroll_positive_for_long_documents() {
        assert_eq!(max_scroll_offset(100, 20.0, 400.0), 1600.0);
    }

    #[test]
    fn wheel_scroll_clamps_and_snaps_to_pixels() {
        let mut offset = 10.4;
        apply_wheel_scroll(&mut offset, -50.0, 100.0);
        assert_eq!(offset, 60.0);
        apply_wheel_scroll(&mut offset, 200.0, 100.0);
        assert_eq!(offset, 0.0);
    }

    #[test]
    fn thumb_and_scroll_mapping_round_trip() {
        let track_top = 20.0;
        let travel = 300.0;
        let max = 1200.0;
        let top = thumb_top_from_scroll(600.0, track_top, travel, max);
        assert!((scroll_from_thumb_top(top, track_top, travel, max) - 600.0).abs() < 0.01);
    }

    #[test]
    fn viewport_indicator_follows_scroll_ratio() {
        let (top, height) = viewport_indicator_in_strip(400.0, 0.0, 200, 20.0, 400.0, 24.0);
        assert_eq!(top, 0.0);
        assert!(height > 0.0);
        let (bottom_top, _) =
            viewport_indicator_in_strip(400.0, 2800.0, 200, 20.0, 400.0, 24.0);
        assert!(bottom_top > top);
    }

    #[test]
    fn track_layout_reserves_arrow_buttons_and_padding() {
        let bar = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(16.0, 400.0));
        let layout = compute_track_layout(bar, bar, 8000.0, 400.0);
        assert!(layout.up_btn_rect.bottom() + TRACK_END_PADDING <= layout.track_rect.top() + 0.01);
        assert!(layout.track_rect.bottom() + TRACK_END_PADDING <= layout.down_btn_rect.top() + 0.01);
        assert!(layout.down_btn_rect.bottom() <= bar.bottom() + 0.01);
        assert!(layout.thumb_height >= MIN_THUMB_HEIGHT);
    }

    #[test]
    fn silhouette_content_width_matches_longest_line() {
        let lines = ["short", "a much longer line of text"];
        let w = compute_silhouette_content_width(
            &lines.map(|s| s as &str),
            200.0,
        );
        let expected = ("a much longer line of text".len() as f32 * 0.5).min(200.0);
        assert!(w > ("short".len() as f32 * 0.5));
        assert_eq!(w, expected);
    }

    #[test]
    fn quick_scroll_strip_width_includes_silhouette_when_enabled() {
        assert_eq!(quick_scroll_strip_width(false, 40.0), QUICK_SCROLL_WIDTH);
        assert_eq!(quick_scroll_strip_width(true, 40.0), QUICK_SCROLL_WIDTH + 40.0);
    }

    #[test]
    fn effective_bar_insets_bottom_on_macos() {
        let bar = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(16.0, 400.0));
        let clip = bar;
        let effective = effective_bar_rect(bar, clip);
        if BAR_BOTTOM_SAFE_INSET > 0.0 {
            assert!(effective.height() <= bar.height());
            let layout = compute_track_layout(effective, clip, 8000.0, effective.height());
            assert!(layout.down_btn_rect.bottom() <= clip.bottom());
            assert!(layout.down_btn_rect.height() >= ARROW_BTN_HEIGHT - 0.01);
        }
    }
}
