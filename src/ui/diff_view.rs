use eframe::egui;
use eframe::epaint::Color32;

use crate::app::RustpadApp;
use crate::diff::{DiffResult, DiffRow, DiffTag};
use crate::ui::compare_session::CompareSession;

const DIFF_MAP_W: f32 = 22.0;
const PANE_GAP: f32 = 4.0;
const GUTTER_W: f32 = 48.0;
const CENTER_W: f32 = 64.0;

struct ViewportScroll {
    session_id: u64,
    scroll_w: f32,
    pane_h: f32,
    row_h: f32,
    scroll_y: f32,
    total_rows: usize,
}

struct LineCell<'a> {
    left: bool,
    line_index: usize,
    width: f32,
    row_h: f32,
    font_size: f32,
    bg: Color32,
    text: Option<&'a str>,
    spans: &'a [(usize, usize)],
}

fn insert_bg() -> Color32 {
    Color32::from_rgb(214, 245, 214)
}
fn delete_bg() -> Color32 {
    Color32::from_rgb(250, 220, 220)
}
fn replace_bg() -> Color32 {
    Color32::from_rgb(250, 244, 205)
}
fn gap_bg() -> Color32 {
    Color32::from_gray(232)
}
fn inline_hl() -> Color32 {
    Color32::from_rgb(247, 214, 124)
}

fn row_height_for(font_size: f32, ui: &egui::Ui) -> f32 {
    ui.spacing().interact_size.y.max(font_size + 6.0)
}

/// `area_height` is the total height reserved for the compare editor block.
pub fn show_text_content(
    app: &RustpadApp,
    session: &mut CompareSession,
    ui: &mut egui::Ui,
    area_height: f32,
) {
    if session.left_text.is_empty() && session.right_text.is_empty() {
        let t = app.tr();
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), area_height.max(80.0)),
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| ui.label(t.diff_pick_files_hint),
        );
        return;
    }

    if session.text_edit_pending_recompute {
        session.recompute_text();
        session.text_edit_pending_recompute = false;
    }

    let diff_result = session.text_result.clone();
    let font_size = session.font_size;
    let row_h = row_height_for(font_size, ui);

    if let Some(line) = session.text_scroll_to_line.take() {
        session.apply_scroll_to_line(line, row_h);
    }

    let editors_h = area_height.max(80.0);
    let full_w = ui.available_width();

    ui.allocate_ui_with_layout(
        egui::vec2(full_w, editors_h),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            show_editor_row(ui, session, &diff_result, font_size, row_h, editors_h);
        },
    );
}

fn show_editor_row(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    diff_result: &Option<DiffResult>,
    font_size: f32,
    row_h: f32,
    pane_h: f32,
) {
    ui.horizontal(|ui| {
        ui.set_min_height(pane_h);
        ui.set_max_height(pane_h);

        if session.show_diff_map {
            if let Some(ref result) = diff_result {
                show_diff_map(ui, session, result, pane_h);
                ui.add_space(PANE_GAP);
            }
        }

        let scroll_w = ui.available_width().max(160.0);
        let scroll_y = session.cmp_sync_scroll_y;
        let (scroll_rect, _) =
            ui.allocate_exact_size(egui::vec2(scroll_w, pane_h), egui::Sense::hover());

        let scroll_output = ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(scroll_rect),
            |ui| {
                let session_id = session.id;
                if let Some(ref result) = diff_result {
                    let col_w = ((scroll_w - CENTER_W - 2.0 * GUTTER_W) / 2.0).max(80.0);
                    show_viewport_rows(
                        ui,
                        ViewportScroll {
                            session_id,
                            scroll_w,
                            pane_h,
                            row_h,
                            scroll_y,
                            total_rows: result.rows.len(),
                        },
                        |ui, i| {
                            let row = &result.rows[i];
                            let block_start = result.change_starts.contains(&i);
                            render_diff_row(
                                session,
                                ui,
                                row,
                                col_w,
                                block_start,
                                font_size,
                                row_h,
                            );
                        },
                    )
                } else {
                    let lines = line_count(&session.left_text)
                        .max(line_count(&session.right_text))
                        .max(1);
                    let col_w = ((scroll_w - PANE_GAP) / 2.0).max(80.0);
                    show_viewport_rows(
                        ui,
                        ViewportScroll {
                            session_id,
                            scroll_w,
                            pane_h,
                            row_h,
                            scroll_y,
                            total_rows: lines,
                        },
                        |ui, line| {
                            plain_line_row(ui, session, line, col_w, font_size, row_h);
                        },
                    )
                }
            },
        );

        session.cmp_sync_scroll_y = scroll_output.inner.state.offset.y;
    });
}

/// Virtualized diff rows: each row gets an absolute `max_rect` so children cannot
/// expand to fill the whole viewport (the root cause of the single-line bug).
fn show_viewport_rows(
    ui: &mut egui::Ui,
    scroll: ViewportScroll,
    mut draw_row: impl FnMut(&mut egui::Ui, usize),
) -> egui::scroll_area::ScrollAreaOutput<()> {
    let ViewportScroll {
        session_id,
        scroll_w,
        pane_h,
        row_h,
        scroll_y,
        total_rows,
    } = scroll;
    egui::ScrollArea::vertical()
        .id_salt((session_id, "cmp_sync_scroll"))
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .vertical_scroll_offset(scroll_y)
        .max_height(pane_h)
        .max_width(scroll_w)
        .show_viewport(ui, move |ui, viewport| {
            ui.set_min_width(scroll_w);
            ui.set_min_height(total_rows as f32 * row_h);

            if total_rows == 0 {
                return;
            }

            let content_top = ui.max_rect().top();
            let first_row = (viewport.min.y / row_h).floor() as usize;
            let last_row = ((viewport.max.y / row_h).ceil() as usize + 1).min(total_rows);

            for i in first_row..last_row {
                let y = content_top + i as f32 * row_h;
                let row_rect = egui::Rect::from_min_size(
                    egui::pos2(ui.max_rect().left(), y),
                    egui::vec2(scroll_w, row_h),
                );
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(row_rect), |ui| {
                    ui.set_clip_rect(row_rect);
                    draw_row(ui, i);
                });
            }
        })
}

fn plain_line_row(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    line: usize,
    col_w: f32,
    font_size: f32,
    row_h: f32,
) {
    display_line_cell(
        session,
        ui,
        LineCell {
            left: true,
            line_index: line,
            width: col_w,
            row_h,
            font_size,
            bg: Color32::TRANSPARENT,
            text: None,
            spans: &[],
        },
    );
    ui.add_space(PANE_GAP);
    display_line_cell(
        session,
        ui,
        LineCell {
            left: false,
            line_index: line,
            width: col_w,
            row_h,
            font_size,
            bg: Color32::TRANSPARENT,
            text: None,
            spans: &[],
        },
    );
}

fn render_diff_row(
    session: &mut CompareSession,
    ui: &mut egui::Ui,
    row: &DiffRow,
    col_w: f32,
    is_block_start: bool,
    font_size: f32,
    row_h: f32,
) {
    let (left_bg, right_bg) = row_backgrounds(row);
    ui.horizontal(|ui| {
        ui.set_max_height(row_h);
        gutter(ui, row.left_line, font_size, row_h);
        if let Some(line) = row.left_line {
            display_line_cell(
                session,
                ui,
                LineCell {
                    left: true,
                    line_index: line,
                    width: col_w,
                    row_h,
                    font_size,
                    bg: left_bg,
                    text: row.left_text.as_deref(),
                    spans: &row.left_spans,
                },
            );
        } else {
            gap_cell(
                ui,
                col_w,
                row_h,
                left_bg,
                row.left_text.as_deref(),
                &row.left_spans,
                font_size,
            );
        }
        merge_buttons(ui, session, row, is_block_start, row_h);
        gutter(ui, row.right_line, font_size, row_h);
        if let Some(line) = row.right_line {
            display_line_cell(
                session,
                ui,
                LineCell {
                    left: false,
                    line_index: line,
                    width: col_w,
                    row_h,
                    font_size,
                    bg: right_bg,
                    text: row.right_text.as_deref(),
                    spans: &row.right_spans,
                },
            );
        } else {
            gap_cell(
                ui,
                col_w,
                row_h,
                right_bg,
                row.right_text.as_deref(),
                &row.right_spans,
                font_size,
            );
        }
    });
}

fn gap_cell(
    ui: &mut egui::Ui,
    width: f32,
    row_h: f32,
    bg: Color32,
    text: Option<&str>,
    spans: &[(usize, usize)],
    font_size: f32,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, row_h), egui::Sense::hover());
    if bg != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 0.0, bg);
    }
    if let Some(text) = text {
        paint_spanned_text(ui, rect, text, spans, font_size);
    }
}

fn merge_buttons(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    row: &DiffRow,
    is_block_start: bool,
    row_h: f32,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(CENTER_W, row_h),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_max_height(row_h);
            if is_block_start {
                if let Some(change_id) = row.change_id {
                    if ui
                        .small_button("▶")
                        .on_hover_text("Copy this change to the right")
                        .clicked()
                    {
                        session.merge_to_right(change_id);
                    }
                    if ui
                        .small_button("◀")
                        .on_hover_text("Copy this change to the left")
                        .clicked()
                    {
                        session.merge_to_left(change_id);
                    }
                }
            }
        },
    );
}

fn row_backgrounds(row: &DiffRow) -> (Color32, Color32) {
    match row.tag {
        DiffTag::Equal => (Color32::TRANSPARENT, Color32::TRANSPARENT),
        DiffTag::Delete => (delete_bg(), gap_bg()),
        DiffTag::Insert => (gap_bg(), insert_bg()),
        DiffTag::Replace => (replace_bg(), replace_bg()),
    }
}

fn gutter(ui: &mut egui::Ui, line: Option<usize>, font_size: f32, row_h: f32) {
    let text = line.map(|n| format!("{:>4}", n + 1)).unwrap_or_default();
    ui.allocate_ui_with_layout(
        egui::vec2(GUTTER_W, row_h),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            ui.set_max_height(row_h);
            ui.label(
                egui::RichText::new(text)
                    .monospace()
                    .size(font_size - 1.0)
                    .color(Color32::from_gray(120)),
            );
        },
    );
}

fn display_line_cell(session: &mut CompareSession, ui: &mut egui::Ui, cell: LineCell<'_>) {
    let LineCell {
        left,
        line_index,
        width,
        row_h,
        font_size,
        bg,
        text,
        spans,
    } = cell;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, row_h), egui::Sense::click());
    if bg != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 0.0, bg);
    }

    let display = text.or_else(|| {
        let source = if left {
            session.left_text.as_str()
        } else {
            session.right_text.as_str()
        };
        source.lines().nth(line_index)
    });

    if let Some(text) = display {
        paint_spanned_text(ui, rect, text, spans, font_size);
    }

    let editing = session.cmp_editing == Some((left, line_index));
    if editing {
        show_inline_editor(session, ui, rect, left, line_index, font_size);
    } else if response.clicked() {
        session.cmp_editing = Some((left, line_index));
    }
}

fn show_inline_editor(
    session: &mut CompareSession,
    ui: &mut egui::Ui,
    rect: egui::Rect,
    left: bool,
    line_index: usize,
    font_size: f32,
) {
    let before_left = session.left_text.clone();
    let before_right = session.right_text.clone();
    let source = if left {
        &session.left_text
    } else {
        &session.right_text
    };
    let Some(line_text) = source.lines().nth(line_index) else {
        session.cmp_editing = None;
        return;
    };
    let mut line_buf = line_text.to_string();

    let edit_rect = rect.shrink2(egui::vec2(1.0, 0.0));
    let response = ui
        .allocate_new_ui(egui::UiBuilder::new().max_rect(edit_rect), |ui| {
            ui.add_sized(
                edit_rect.size(),
                egui::TextEdit::singleline(&mut line_buf)
                    .id(egui::Id::new((session.id, left, line_index, "diff_edit")))
                    .font(egui::FontId::monospace(font_size))
                    .frame(true)
                    .margin(egui::vec2(2.0, 0.0)),
            )
        })
        .inner;
    if !response.has_focus() {
        response.request_focus();
    }
    if response.changed() {
        session.push_edit_undo_snapshot(&before_left, &before_right);
        set_text_line(
            if left {
                &mut session.left_text
            } else {
                &mut session.right_text
            },
            line_index,
            &line_buf,
        );
        if left {
            session.left_dirty = true;
        } else {
            session.right_dirty = true;
        }
        session.text_edit_pending_recompute = true;
    }
    if response.lost_focus() {
        session.cmp_editing = None;
    }
}

fn set_text_line(text: &mut String, line_index: usize, new_line: &str) {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    if line_index >= lines.len() {
        lines.resize(line_index + 1, String::new());
    }
    lines[line_index] = new_line.to_string();
    *text = lines.join("\n");
}

fn paint_spanned_text(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    text: &str,
    spans: &[(usize, usize)],
    font_size: f32,
) {
    let text_color = ui.style().visuals.text_color();
    let job = build_job(text, spans, text_color, font_size);
    let galley = ui.fonts(|f| f.layout_job(job));
    let pos = egui::pos2(rect.min.x + 4.0, rect.min.y + 1.0);
    ui.painter().galley(pos, galley, text_color);
}

fn build_job(
    text: &str,
    spans: &[(usize, usize)],
    text_color: Color32,
    font_size: f32,
) -> egui::text::LayoutJob {
    use egui::text::{LayoutJob, TextFormat};
    let mut job = LayoutJob::default();
    let font = egui::FontId::monospace(font_size);
    let plain = TextFormat {
        font_id: font.clone(),
        color: text_color,
        ..Default::default()
    };
    let hl = TextFormat {
        font_id: font,
        color: text_color,
        background: inline_hl(),
        ..Default::default()
    };
    if spans.is_empty() {
        job.append(text, 0.0, plain);
        return job;
    }
    let chars: Vec<char> = text.chars().collect();
    let mut idx = 0usize;
    for &(start, end) in spans {
        let start = start.min(chars.len());
        let end = end.min(chars.len());
        if start > idx {
            let seg: String = chars[idx..start].iter().collect();
            job.append(&seg, 0.0, plain.clone());
        }
        if end > start {
            let seg: String = chars[start..end].iter().collect();
            job.append(&seg, 0.0, hl.clone());
        }
        idx = end.max(idx);
    }
    if idx < chars.len() {
        let seg: String = chars[idx..].iter().collect();
        job.append(&seg, 0.0, plain);
    }
    job
}

fn line_count(text: &str) -> usize {
    if text.is_empty() {
        1
    } else {
        text.matches('\n').count() + 1
    }
}

struct MapRow<'a> {
    row: &'a DiffRow,
    original_index: usize,
}

fn show_diff_map(
    ui: &mut egui::Ui,
    session: &mut CompareSession,
    result: &DiffResult,
    height: f32,
) {
    let rows = diff_map_rows(result, session.expand_unchanged);
    let n = rows.len().max(1);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(DIFF_MAP_W, height), egui::Sense::click());
    let row_h = (height / n as f32).max(1.0);

    for (i, entry) in rows.iter().enumerate() {
        let color = match entry.row.tag {
            DiffTag::Equal => Color32::from_gray(235),
            DiffTag::Delete => delete_bg(),
            DiffTag::Insert => insert_bg(),
            DiffTag::Replace => replace_bg(),
        };
        let y = rect.min.y + i as f32 * row_h;
        let r = egui::Rect::from_min_size(egui::pos2(rect.min.x, y), egui::vec2(DIFF_MAP_W, row_h));
        ui.painter().rect_filled(r, 0.0, color);
    }

    ui.painter()
        .rect_stroke(rect, 0.0, ui.visuals().widgets.noninteractive.bg_stroke);

    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let idx = ((pos.y - rect.min.y) / row_h).floor() as usize;
            if let Some(entry) = rows.get(idx) {
                if let Some(line) = entry.row.left_line.or(entry.row.right_line) {
                    let line_height = session.font_size + 5.0;
                    session.apply_scroll_to_line(line, line_height);
                }
                session.text_current_change =
                    change_index_for_row(result, entry.original_index);
            }
        }
    }
}

fn change_index_for_row(result: &DiffResult, row_index: usize) -> usize {
    for (i, &start) in result.change_starts.iter().enumerate().rev() {
        if row_index >= start {
            return i;
        }
    }
    0
}

fn diff_map_rows<'a>(result: &'a DiffResult, expand_unchanged: bool) -> Vec<MapRow<'a>> {
    if expand_unchanged {
        return result
            .rows
            .iter()
            .enumerate()
            .map(|(i, row)| MapRow {
                row,
                original_index: i,
            })
            .collect();
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < result.rows.len() {
        if result.rows[i].tag == DiffTag::Equal {
            let start = i;
            while i < result.rows.len() && result.rows[i].tag == DiffTag::Equal {
                i += 1;
            }
            let run = i - start;
            if run <= 3 {
                for idx in start..i {
                    out.push(MapRow {
                        row: &result.rows[idx],
                        original_index: idx,
                    });
                }
            } else {
                out.push(MapRow {
                    row: &result.rows[start],
                    original_index: start,
                });
                out.push(MapRow {
                    row: &result.rows[i - 1],
                    original_index: i - 1,
                });
            }
        } else {
            out.push(MapRow {
                row: &result.rows[i],
                original_index: i,
            });
            i += 1;
        }
    }
    out
}

/// Diff statistics strip in the compare window footer (single line, below encoding).
pub fn show_text_stats(ui: &mut egui::Ui, session: &CompareSession, diff_result: Option<&DiffResult>) {
    show_stats(ui, session, diff_result);
}

fn show_stats(ui: &mut egui::Ui, session: &CompareSession, diff_result: Option<&DiffResult>) {
    let Some(result) = diff_result else {
        ui.label("No diff result");
        return;
    };

    let s = &result.stats;
    let mut line = format!(
        "Changes: {} | Equal: {} | Inserted: {} | Deleted: {} | Replaced: {}",
        result.change_count(),
        s.equal,
        s.insertions,
        s.deletions,
        s.replacements,
    );
    if result.change_count() > 0 {
        line.push_str(&format!(
            " | At change {}/{}",
            session.text_current_change + 1,
            result.change_count()
        ));
    }
    line.push_str(&format!(" | Algo: {}", session.algorithm_label()));
    if session.strict_mode {
        line.push_str(" | Strict");
    }
    if session.ignore_whitespace {
        line.push_str(" | IgnWS");
    }
    if session.ignore_case {
        line.push_str(" | IgnCase");
    }
    if session.show_whitespace {
        line.push_str(" | WS");
    }
    if session.show_diff_map {
        line.push_str(" | Map");
    }

    ui.label(egui::RichText::new(line).size(12.0));
}
