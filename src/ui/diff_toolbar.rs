use eframe::egui;

use crate::app::RustpadApp;

/// Show the diff toolbar (visible only while comparing).
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_diff_view {
        return;
    }

    egui::TopBottomPanel::top("diff_toolbar").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Open Left…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    app.diff_open_left(path);
                }
            }
            if ui.button("Open Right…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    app.diff_open_right(path);
                }
            }
            if ui.button("⇄ Swap").clicked() {
                app.diff_swap();
            }

            ui.separator();

            // Change navigation (F7 / F8).
            let has_changes = app
                .diff_result
                .as_ref()
                .map(|r| r.change_count() > 0)
                .unwrap_or(false);
            ui.add_enabled_ui(has_changes, |ui| {
                if ui
                    .button("⬆ Prev")
                    .on_hover_text("Previous difference (F7)")
                    .clicked()
                {
                    app.diff_prev_change();
                }
                if ui
                    .button("⬇ Next")
                    .on_hover_text("Next difference (F8)")
                    .clicked()
                {
                    app.diff_next_change();
                }
            });

            ui.separator();

            let mut changed = false;
            changed |= ui
                .checkbox(&mut app.diff_ignore_whitespace, "Ignore Whitespace")
                .changed();
            changed |= ui
                .checkbox(&mut app.diff_ignore_case, "Ignore Case")
                .changed();
            if changed {
                app.recompute_diff();
            }

            ui.separator();

            ui.add_enabled_ui(app.diff_left_dirty, |ui| {
                if ui.button("💾 Save Left").clicked() {
                    app.diff_save_left();
                }
            });
            ui.add_enabled_ui(app.diff_right_dirty, |ui| {
                if ui.button("💾 Save Right").clicked() {
                    app.diff_save_right();
                }
            });

            if ui.button("Export Report…").clicked() {
                app.export_diff_report();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✖ Close Compare").clicked() {
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
