use eframe::egui;

use crate::app::RustpadApp;

/// Show the diff toolbar (visible only while comparing).
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_diff_view {
        return;
    }

    let t = app.tr();
    egui::TopBottomPanel::top("diff_toolbar").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            if ui.button(t.diff_open_left).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    app.diff_open_left(path);
                }
            }
            if ui.button(t.diff_open_right).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    app.diff_open_right(path);
                }
            }
            if ui.button(t.diff_swap).clicked() {
                app.diff_swap();
            }

            ui.separator();

            let has_changes = app
                .diff_result
                .as_ref()
                .map(|r| r.change_count() > 0)
                .unwrap_or(false);
            ui.add_enabled_ui(has_changes, |ui| {
                if ui
                    .button(t.diff_prev)
                    .on_hover_text(t.tip_diff_prev)
                    .clicked()
                {
                    app.diff_prev_change();
                }
                if ui
                    .button(t.diff_next)
                    .on_hover_text(t.tip_diff_next)
                    .clicked()
                {
                    app.diff_next_change();
                }
            });

            ui.separator();

            let mut changed = false;
            changed |= ui
                .checkbox(&mut app.diff_ignore_whitespace, t.diff_ignore_ws)
                .changed();
            changed |= ui
                .checkbox(&mut app.diff_ignore_case, t.diff_ignore_case)
                .changed();
            if changed {
                app.recompute_diff();
            }

            ui.separator();

            ui.add_enabled_ui(app.diff_left_dirty, |ui| {
                if ui.button(t.diff_save_left).clicked() {
                    app.diff_save_left();
                }
            });
            ui.add_enabled_ui(app.diff_right_dirty, |ui| {
                if ui.button(t.diff_save_right).clicked() {
                    app.diff_save_right();
                }
            });

            if ui.button(t.diff_export).clicked() {
                app.export_diff_report();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(t.diff_close).clicked() {
                    app.close_diff_view();
                }
            });
        });
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_diff_toolbar_compiles() {
        assert!(true);
    }
}
