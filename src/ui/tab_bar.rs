use eframe::egui;

use crate::app::RustpadApp;

const TAB_SCROLL_STEP: f32 = 160.0;
const TAB_NAV_BTN_WIDTH: f32 = 22.0;
const TAB_NEW_BTN_WIDTH: f32 = 24.0;

/// Render the tab bar showing all open tabs.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    let t = app.tr();
    egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
        let scroll_id = ui.id().with("tab_bar_h_scroll");
        let mut scroll_x = ui.data(|d| d.get_temp::<f32>(scroll_id)).unwrap_or(0.0);

        let tab_count = app.tab_manager.tab_count();
        let active_idx = app.tab_manager.active_index();
        let mut tab_to_close: Option<usize> = None;
        let mut tab_to_activate: Option<usize> = None;
        let mut tab_context_action: Option<crate::ui::tab_context_menu::TabMenuAction> = None;
        let mut active_tab_response: Option<egui::Response> = None;

        let btn_height = ui.spacing().interact_size.y;
        let right_controls_width = TAB_NAV_BTN_WIDTH * 2.0
            + TAB_NEW_BTN_WIDTH
            + ui.spacing().item_spacing.x * 2.0;

        ui.horizontal(|ui| {
            let scroll_width = (ui.available_width() - right_controls_width).max(80.0);

            let scroll = egui::ScrollArea::horizontal()
                .id_salt("tab_strip")
                .max_width(scroll_width)
                .horizontal_scroll_offset(scroll_x)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for i in 0..tab_count {
                            let title = app.tab_manager.tabs()[i].display_title();
                            let is_active = i == active_idx;

                            let response = ui.selectable_label(is_active, &title);
                            if response.clicked() {
                                tab_to_activate = Some(i);
                            }
                            response.context_menu(|ui| {
                                if let Some(action) =
                                    crate::ui::tab_context_menu::show(ui, app, i, tab_count)
                                {
                                    tab_context_action = Some(action);
                                }
                            });

                            if is_active || response.hovered() {
                                let close_btn = ui.small_button("×");
                                if close_btn.clicked() {
                                    tab_to_close = Some(i);
                                }
                            }

                            if is_active {
                                active_tab_response = Some(response);
                            }

                            if i < tab_count - 1 {
                                ui.separator();
                            }
                        }
                    });
                });

            scroll_x = scroll.state.offset.x;
            let max_scroll =
                (scroll.content_size.x - scroll.inner_rect.width()).max(0.0);

            // Keep the active tab visible horizontally without `scroll_to_me`, which also
            // sets a vertical scroll target and breaks other panels (e.g. search results).
            if let Some(response) = active_tab_response {
                let tab_center = response.rect.center().x;
                let view_center = scroll.inner_rect.center().x;
                let delta = tab_center - view_center;
                if delta.abs() > 0.5 {
                    scroll_x = (scroll_x + delta).clamp(0.0, max_scroll);
                }
            }

            let can_scroll_left = scroll_x > 0.5;
            let can_scroll_right = scroll_x < max_scroll - 0.5;

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Rightmost: scroll arrows grouped (Notepad++ style).
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    if ui
                        .add_enabled(
                            can_scroll_right,
                            egui::Button::new(egui::RichText::new("▶").size(11.0))
                                .min_size(egui::vec2(TAB_NAV_BTN_WIDTH, btn_height)),
                        )
                        .on_hover_text(t.tip_tab_scroll_right)
                        .clicked()
                    {
                        scroll_x = (scroll_x + TAB_SCROLL_STEP).min(max_scroll);
                    }
                    if ui
                        .add_enabled(
                            can_scroll_left,
                            egui::Button::new(egui::RichText::new("◀").size(11.0))
                                .min_size(egui::vec2(TAB_NAV_BTN_WIDTH, btn_height)),
                        )
                        .on_hover_text(t.tip_tab_scroll_left)
                        .clicked()
                    {
                        scroll_x = (scroll_x - TAB_SCROLL_STEP).max(0.0);
                    }
                });

                if ui
                    .add_sized(
                        [TAB_NEW_BTN_WIDTH, btn_height],
                        egui::Button::new("+"),
                    )
                    .on_hover_text(t.tip_new)
                    .clicked()
                {
                    app.tab_manager.new_tab();
                }
            });
        });

        ui.data_mut(|d| d.insert_temp(scroll_id, scroll_x));

        if let Some(idx) = tab_to_activate {
            if idx != app.tab_manager.active_index() {
                app.tab_manager.set_active(idx);
                app.highlighter.clear_cache();
                app.highlighter.invalidate_all();
            }
        }
        if let Some(idx) = tab_to_close {
            app.close_tab_at(idx);
        }
        if let Some(action) = tab_context_action {
            crate::ui::tab_context_menu::dispatch(app, action);
        }
    });
}
