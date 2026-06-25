use eframe::egui;
use std::path::PathBuf;

use crate::app::RustpadApp;
use crate::i18n::Locale;
use crate::ui::text_util;

/// Sidebar panel content.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    #[default]
    FileExplorer,
    Search,
    Bookmarks,
}

/// Render the sidebar (file explorer, etc.).
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_sidebar {
        return;
    }

    egui::SidePanel::left("sidebar")
        .default_width(app.config.ui.sidebar_width)
        .min_width(180.0)
        .show(ctx, |ui| {
            let t = app.tr();
            ui.horizontal_wrapped(|ui| {
                let mut tab = app.sidebar_tab;
                ui.selectable_value(&mut tab, SidebarTab::FileExplorer, t.sidebar_tab_files);
                ui.selectable_value(&mut tab, SidebarTab::Search, t.sidebar_tab_search);
                ui.selectable_value(&mut tab, SidebarTab::Bookmarks, t.sidebar_tab_bookmarks);
                app.sidebar_tab = tab;
            });
            ui.separator();

            match app.sidebar_tab {
                SidebarTab::FileExplorer => show_file_explorer(app, ui, t),
                SidebarTab::Search => show_sidebar_search(app, ui, t),
                SidebarTab::Bookmarks => show_bookmarks(ui, t),
            }
        });
}

fn path_row(ui: &mut egui::Ui, display: &str, full: &str) -> egui::Response {
    ui.add(
        egui::Label::new(display)
            .truncate()
            .sense(egui::Sense::click()),
    )
    .on_hover_text(full)
}

fn show_file_explorer(app: &mut RustpadApp, ui: &mut egui::Ui, t: &'static Locale) {
    ui.heading(t.sidebar_file_explorer);
    ui.separator();

    ui.label(t.sidebar_recent_files);
    let recent = app.config.recent_files.clone();
    egui::ScrollArea::vertical()
        .max_height(120.0)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if recent.is_empty() {
                ui.weak(t.menu_no_recent_files);
            } else {
                let width = ui.available_width();
                for path in &recent {
                    let full_path = path.display().to_string();
                    let display = text_util::ellipsis_path_for_width(&full_path, width);
                    if path_row(ui, &display, &full_path).clicked() {
                        let p = path.clone();
                        app.tab_manager.open_file(&p).ok();
                    }
                }
            }
        });

    ui.separator();

    if ui.button(t.sidebar_open_folder).clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            app.open_workspace_folder(path);
        }
    }

    if let Some(root) = app.workspace_root.clone() {
        ui.separator();
        let full_workspace = root.display().to_string();
        let workspace_display =
            text_util::ellipsis_path_for_width(&full_workspace, ui.available_width());
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("{}:", t.sidebar_workspace));
            path_row(ui, &workspace_display, &full_workspace);
        });

        let results = collect_files(&root);
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let width = ui.available_width();
                for entry_path in results {
                    let full_path = entry_path.display().to_string();
                    let name = entry_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| full_path.clone());
                    let display = text_util::ellipsis_path_for_width(&name, width);
                    let label = format!("📄 {display}");
                    if path_row(ui, &label, &full_path).clicked() {
                        app.tab_manager.open_file(&entry_path).ok();
                    }
                }
            });
    }
}

fn collect_files(path: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if entry_path.is_file() {
                files.push(entry_path);
            } else if entry_path.is_dir() {
                files.extend(collect_files(&entry_path));
            }
        }
    }
    files
}

fn show_sidebar_search(app: &mut RustpadApp, ui: &mut egui::Ui, t: &'static Locale) {
    ui.heading(t.sidebar_search_in_files);
    ui.separator();

    let mut search_text = app.search_panel_text.clone();
    ui.text_edit_singleline(&mut search_text);
    let changed = search_text != app.search_panel_text;
    app.search_panel_text = search_text;

    if changed && !app.search_panel_text.is_empty() {
        app.search_results.clear();
        if let Some(root) = app.workspace_root.clone() {
            search_in_directory(&mut app.search_results, &root, &app.search_panel_text);
        }
    }

    ui.separator();

    let results = app.search_results.clone();
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let width = ui.available_width();
            for (path, line_num, line_text) in &results {
                let full = format!("{}:{}: {}", path.display(), line_num + 1, line_text.trim());
                let display = text_util::ellipsis_path_for_width(&full, width);
                if path_row(ui, &display, &full).clicked() {
                    app.tab_manager.open_file(path).ok();
                }
            }
        });
}

fn search_in_directory(results: &mut Vec<(PathBuf, usize, String)>, path: &PathBuf, pattern: &str) {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                search_in_directory(results, &entry_path, pattern);
            } else if entry_path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&entry_path) {
                    for (i, line) in content.lines().enumerate() {
                        if line.contains(pattern) {
                            results.push((entry_path.clone(), i, line.to_string()));
                        }
                    }
                }
            }
        }
    }
}

fn show_bookmarks(ui: &mut egui::Ui, t: &'static Locale) {
    ui.heading(t.sidebar_bookmarks);
    ui.separator();
    ui.label(t.sidebar_no_bookmarks);
}
