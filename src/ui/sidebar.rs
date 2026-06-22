use eframe::egui;
use std::path::PathBuf;

use crate::app::RustpadApp;

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
        .show(ctx, |ui| {
            // Tab buttons
            ui.horizontal(|ui| {
                let mut tab = app.sidebar_tab;
                ui.selectable_value(&mut tab, SidebarTab::FileExplorer, "📁 Files");
                ui.selectable_value(&mut tab, SidebarTab::Search, "🔍 Search");
                ui.selectable_value(&mut tab, SidebarTab::Bookmarks, "🔖 Bookmarks");
                app.sidebar_tab = tab;
            });
            ui.separator();

            match app.sidebar_tab {
                SidebarTab::FileExplorer => show_file_explorer(app, ui),
                SidebarTab::Search => show_sidebar_search(app, ui),
                SidebarTab::Bookmarks => show_bookmarks(ui),
            }
        });
}

fn show_file_explorer(app: &mut RustpadApp, ui: &mut egui::Ui) {
    ui.heading("File Explorer");
    ui.separator();

    // Recent files
    ui.label("Recent Files:");
    let recent = app.config.recent_files.clone();
    for path in &recent {
        let full_path = path.display().to_string();

        if ui
            .selectable_label(false, &full_path)
            .on_hover_text(&full_path)
            .clicked()
        {
            let p = path.clone();
            app.tab_manager.open_file(&p).ok();
        }
    }

    ui.separator();

    // Open folder button
    if ui.button("Open Folder...").clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            app.workspace_root = Some(path);
        }
    }

    // Show workspace files
    if let Some(root) = app.workspace_root.clone() {
        ui.separator();
        ui.label(format!("Workspace: {}", root.display()));
        let results = collect_files(&root);
        for entry_path in results {
            let full_path = entry_path.display().to_string();
            if ui
                .selectable_label(false, format!("📄 {}", full_path))
                .on_hover_text(&full_path)
                .clicked()
            {
                app.tab_manager.open_file(&entry_path).ok();
            }
        }
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

fn show_sidebar_search(app: &mut RustpadApp, ui: &mut egui::Ui) {
    ui.heading("Search in Files");
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

    // Show search results (clone to avoid borrow issues)
    let results = app.search_results.clone();
    for (path, line_num, line_text) in &results {
        let label = format!("{}:{}: {}", path.display(), line_num + 1, line_text.trim());
        if ui.selectable_label(false, &label).clicked() {
            app.tab_manager.open_file(path).ok();
        }
    }
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

fn show_bookmarks(ui: &mut egui::Ui) {
    ui.heading("Bookmarks");
    ui.separator();
    ui.label("No bookmarks yet.");
}
