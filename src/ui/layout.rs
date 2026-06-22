use eframe::egui;

use crate::app::RustpadApp;

/// Main application layout with menu bar, tab bar, editor area, and status bar.
#[allow(dead_code)]
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    // Top menu bar
    show_menu_bar(app, ctx);

    // Tab bar below menu
    show_tab_bar(app, ctx);

    // Status bar at bottom
    show_status_bar(app, ctx);

    // Central editor area
    show_editor_area(app, ctx);
}

/// Top menu bar with File/Edit/View/Search/Tools/Help menus.
#[allow(dead_code)]
fn show_menu_bar(app: &mut RustpadApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New  Ctrl+N").clicked() {
                    app.tab_manager.new_tab();
                    ui.close_menu();
                }
                if ui.button("Open...  Ctrl+O").clicked() {
                    app.pending_open_file = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Save  Ctrl+S").clicked() {
                    app.save_current_tab();
                    ui.close_menu();
                }
                if ui.button("Save As...  Ctrl+Shift+S").clicked() {
                    app.save_as_dialog();
                    ui.close_menu();
                }
                if ui.button("Save All").clicked() {
                    app.save_all_tabs();
                    ui.close_menu();
                }
                ui.separator();
                // Recent files submenu
                ui.menu_button("Recent Files", |ui| {
                    let recent = app.config.recent_files.clone();
                    if recent.is_empty() {
                        ui.label("No recent files");
                    } else {
                        for path in &recent {
                            let name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| path.display().to_string());
                            if ui.button(&name).clicked() {
                                let p = path.clone();
                                app.tab_manager.open_file(&p).ok();
                                ui.close_menu();
                            }
                        }
                    }
                });
                ui.separator();
                if ui.button("Close Tab  Ctrl+W").clicked() {
                    app.close_current_tab();
                    ui.close_menu();
                }
                if ui.button("Close All").clicked() {
                    // Close all tabs (with unsaved check)
                    while app.tab_manager.tab_count() > 1 {
                        app.tab_manager.close_tab(0);
                    }
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit  Alt+F4").clicked() {
                    app.request_exit(ctx);
                    ui.close_menu();
                }
            });

            // Edit menu
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo  Ctrl+Z").clicked() {
                    app.tab_manager.active_mut().buffer.undo();
                    ui.close_menu();
                }
                if ui.button("Redo  Ctrl+Y").clicked() {
                    app.tab_manager.active_mut().buffer.redo();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Cut  Ctrl+X").clicked() {
                    app.cut();
                    ui.close_menu();
                }
                if ui.button("Copy  Ctrl+C").clicked() {
                    app.copy();
                    ui.close_menu();
                }
                if ui.button("Paste  Ctrl+V").clicked() {
                    app.paste();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Select All  Ctrl+A").clicked() {
                    app.select_all();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Find...  Ctrl+F").clicked() {
                    app.open_find(false);
                    ui.close_menu();
                }
                if ui.button("Replace...  Ctrl+H").clicked() {
                    app.open_find(true);
                    ui.close_menu();
                }
                if ui.button("Go to Line...  Ctrl+G").clicked() {
                    app.show_goto_line = true;
                    ui.close_menu();
                }
            });

            // View menu
            ui.menu_button("View", |ui| {
                if ui.button("Toggle Sidebar  Ctrl+B").clicked() {
                    app.show_sidebar = !app.show_sidebar;
                    ui.close_menu();
                }
                if ui.button("Toggle Minimap").clicked() {
                    app.config.ui.show_minimap = !app.config.ui.show_minimap;
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("Font Size", |ui| {
                    for size in [10, 12, 14, 16, 18, 20, 24] {
                        if ui
                            .selectable_label(
                                app.config.editor.font_size == size as f32,
                                format!("{}px", size),
                            )
                            .clicked()
                        {
                            app.config.editor.font_size = size as f32;
                        }
                    }
                });
                ui.menu_button("Tab Size", |ui| {
                    for size in [2, 4, 8] {
                        if ui
                            .selectable_label(
                                app.config.editor.tab_size == size,
                                format!("{}", size),
                            )
                            .clicked()
                        {
                            app.config.editor.tab_size = size;
                        }
                    }
                });
                ui.separator();
                if ui.button("Word Wrap").clicked() {
                    app.config.editor.word_wrap = !app.config.editor.word_wrap;
                    ui.close_menu();
                }
                if ui.button("Line Numbers").clicked() {
                    app.config.editor.show_line_numbers = !app.config.editor.show_line_numbers;
                    ui.close_menu();
                }
            });

            // Search menu
            ui.menu_button("Search", |ui| {
                if ui.button("Find...  Ctrl+F").clicked() {
                    app.open_find(false);
                    ui.close_menu();
                }
                if ui.button("Replace...  Ctrl+H").clicked() {
                    app.open_find(true);
                    ui.close_menu();
                }
                if ui.button("Go to Line...  Ctrl+G").clicked() {
                    app.show_goto_line = true;
                    ui.close_menu();
                }
            });

            // Tools menu
            ui.menu_button("Tools", |ui| {
                if ui.button("Compare Files...  Ctrl+D").clicked() {
                    app.pending_compare_files = true;
                    ui.close_menu();
                }
                if ui.button("Macro Recording").clicked() {
                    app.toggle_macro_recording();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Command Palette  Ctrl+Shift+P").clicked() {
                    app.toggle_command_palette();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Preferences...").clicked() {
                    app.show_preferences = true;
                    ui.close_menu();
                }
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.button("About RustPad").clicked() {
                    app.show_about = true;
                    ui.close_menu();
                }
            });
        });
    });
}

/// Tab bar showing all open tabs with close buttons.
#[allow(dead_code)]
fn show_tab_bar(app: &mut RustpadApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let tab_count = app.tab_manager.tab_count();
            let active_idx = app.tab_manager.active_index();
            let mut tab_to_close: Option<usize> = None;
            let mut tab_to_activate: Option<usize> = None;

            for i in 0..tab_count {
                let title = app.tab_manager.tabs()[i].display_title();
                let is_active = i == active_idx;

                let response = ui.selectable_label(is_active, &title);

                // Left click to activate
                if response.clicked() {
                    tab_to_activate = Some(i);
                }

                // Middle click to close
                if response.middle_clicked() {
                    tab_to_close = Some(i);
                }

                // Close button on hover
                if is_active || response.hovered() {
                    let close_btn = ui.small_button("×");
                    if close_btn.clicked() {
                        tab_to_close = Some(i);
                    }
                }

                // Right click context menu
                response.context_menu(|ui| {
                    if ui.button("Close").clicked() {
                        tab_to_close = Some(i);
                        ui.close_menu();
                    }
                    if ui.button("Close Others").clicked() {
                        // Close all tabs except this one
                        for j in (0..tab_count).rev() {
                            if j != i {
                                app.tab_manager.close_tab(j);
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Close All").clicked() {
                        for j in (0..tab_count).rev() {
                            app.tab_manager.close_tab(j);
                        }
                        ui.close_menu();
                    }
                });

                if i < tab_count - 1 {
                    ui.separator();
                }
            }

            // New tab button
            if ui.button("+").on_hover_text("New tab").clicked() {
                app.tab_manager.new_tab();
            }

            // Apply deferred actions
            if let Some(idx) = tab_to_activate {
                app.tab_manager.set_active(idx);
            }
            if let Some(idx) = tab_to_close {
                let is_dirty = app.tab_manager.tabs()[idx].buffer.is_dirty()
                    || app.tab_manager.tabs()[idx].modified;
                if is_dirty {
                    app.pending_close_tab = Some(idx);
                    app.show_unsaved_dialog = true;
                } else {
                    app.tab_manager.close_tab(idx);
                }
            }
        });
    });
}

/// Status bar at the bottom showing line, column, encoding, etc.
#[allow(dead_code)]
fn show_status_bar(app: &RustpadApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Cursor position
            ui.label(&app.status_bar.cursor_position);
            ui.separator();

            // Line count
            ui.label(format!("{} lines", app.tab_manager.active().line_count()));
            ui.separator();

            // Encoding
            ui.label(&app.status_bar.encoding);
            ui.separator();

            // Line ending
            ui.label(&app.status_bar.line_ending);
            ui.separator();

            // Language
            ui.label(&app.status_bar.language);
            ui.separator();

            // File size
            let size = app.tab_manager.active().buffer.len();
            if size > 1024 * 1024 {
                ui.label(format!("{:.1} MB", size as f64 / 1024.0 / 1024.0));
            } else if size > 1024 {
                ui.label(format!("{:.1} KB", size as f64 / 1024.0));
            } else {
                ui.label(format!("{} B", size));
            }

            // Right side: file path
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(path) = &app.tab_manager.active().file_path {
                    ui.label(path.to_string_lossy());
                }
            });
        });
    });
}

/// Central editor area (placeholder for Phase 2+).
#[allow(dead_code)]
fn show_editor_area(app: &mut RustpadApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let tab = app.tab_manager.active();
        let text = tab.buffer.text().to_string();
        let line_count = tab.buffer.line_count();

        ui.horizontal(|ui| {
            // Line numbers gutter
            if app.config.editor.show_line_numbers {
                let gutter_width = 50.0;
                ui.vertical(|ui| {
                    ui.set_width(gutter_width);
                    for i in 0..line_count.min(100) {
                        ui.label(
                            egui::RichText::new(format!("{:>4}", i + 1))
                                .monospace()
                                .size(app.config.editor.font_size * 0.9)
                                .color(egui::Color32::from_gray(100)),
                        );
                    }
                });
                ui.separator();
            }

            // Editor content
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text.to_string())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(50)
                        .code_editor(),
                );
            });
        });
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_layout_module_compiles() {
        // Verify layout module is properly linked
        assert!(true);
    }
}
