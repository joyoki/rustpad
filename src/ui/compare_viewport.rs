//! Render detached compare windows via egui viewports.

use eframe::egui;

use crate::app::RustpadApp;
use crate::diff::FileStatus;
use crate::ui::compare_window;

const COMPARE_WIN_SIZE: [f32; 2] = [1120.0, 760.0];

pub fn show_all(app: &mut RustpadApp, parent_ctx: &egui::Context) {
    let app_ptr = app as *mut RustpadApp;
    let parent_style = (*parent_ctx.style()).clone();

    let ids: Vec<u64> = app.compare_mgr.sessions.iter().map(|s| s.id).collect();
    for id in ids {
        let Some(session) = app.compare_mgr.sessions.iter().find(|s| s.id == id) else {
            continue;
        };
        let viewport_id = session.viewport_id;
        let title = compare_window::window_title(app, session);
        parent_ctx.show_viewport_immediate(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title(&title)
                .with_inner_size(COMPARE_WIN_SIZE)
                .with_min_inner_size([720.0, 480.0])
                .with_drag_and_drop(true),
            {
                let parent_style = parent_style.clone();
                let title = title.clone();
                move |ctx, class| {
                    let app = unsafe { &mut *app_ptr };
                    sync_viewport_appearance(app, ctx, &parent_style);
                    run_viewport_ui(class, ctx, &title, |ctx| {
                        render_compare_window(app, id, ctx);
                    });
                }
            },
        );
    }

    process_all_pending(app);
    app.compare_mgr.retain_open();
}

fn sync_viewport_appearance(app: &RustpadApp, ctx: &egui::Context, parent_style: &egui::Style) {
    ctx.set_style(parent_style.clone());
    app.apply_theme_to_context(ctx);
}

fn run_viewport_ui(
    class: egui::ViewportClass,
    ctx: &egui::Context,
    title: &str,
    render: impl FnOnce(&egui::Context),
) {
    match class {
        egui::ViewportClass::Embedded => {
            egui::Window::new(title)
                .resizable(true)
                .default_size(COMPARE_WIN_SIZE)
                .show(ctx, |_ui| {
                    render(ctx);
                });
        }
        egui::ViewportClass::Immediate
        | egui::ViewportClass::Root
        | egui::ViewportClass::Deferred => {
            render(ctx);
        }
    }
}

fn render_compare_window(app: &mut RustpadApp, id: u64, ctx: &egui::Context) {
    let app_ptr = app as *const RustpadApp;
    if let Some(session) = app.compare_mgr.session_mut(id) {
        session.handle_keys(ctx);
        let app_imm = unsafe { &*app_ptr };
        compare_window::show(app_imm, session, ctx);
    }
    if let Some(session) = app.compare_mgr.session_mut(id) {
        if ctx.input(|i| i.viewport().close_requested()) {
            session.open = false;
        }
    }
    process_session_pending(app, id);
    maybe_record_history(app, id);
}

fn process_all_pending(app: &mut RustpadApp) {
    let ids: Vec<u64> = app.compare_mgr.sessions.iter().map(|s| s.id).collect();
    for id in ids {
        process_session_pending(app, id);
    }
}

fn maybe_record_history(app: &mut RustpadApp, id: u64) {
    let record = app
        .compare_mgr
        .session_mut(id)
        .and_then(|session| {
            let mode = session.pending_history_record.take()?;
            Some((session.left_path.clone(), session.right_path.clone(), mode))
        });
    if let Some((left, right, mode)) = record {
        app.record_compare_history(left, right, mode);
    }
}

fn process_session_pending(app: &mut RustpadApp, id: u64) {
    let (save_left, save_right, export, open_file, open_binary) = {
        let Some(session) = app.compare_mgr.session_mut(id) else {
            return;
        };
        let flags = (
            session.pending_save_left,
            session.pending_save_right,
            session.pending_export,
            session.pending_open_file_compare.take(),
            session.pending_open_binary_compare.take(),
        );
        session.pending_save_left = false;
        session.pending_save_right = false;
        session.pending_export = false;
        flags
    };

    if save_left {
        let data = app
            .compare_mgr
            .session_mut(id)
            .map(|s| (s.left_path.clone(), s.left_text.clone()));
        if let Some((path, text)) = data {
            let op = app.tr().err_op_save;
            if app.write_file_with_feedback(&path, &text, op) {
                if let Some(session) = app.compare_mgr.session_mut(id) {
                    session.left_dirty = false;
                }
                app.sync_open_tab_with_disk(&path, &text);
            }
        }
    }
    if save_right {
        let data = app
            .compare_mgr
            .session_mut(id)
            .map(|s| (s.right_path.clone(), s.right_text.clone()));
        if let Some((path, text)) = data {
            let op = app.tr().err_op_save;
            if app.write_file_with_feedback(&path, &text, op) {
                if let Some(session) = app.compare_mgr.session_mut(id) {
                    session.right_dirty = false;
                }
                app.sync_open_tab_with_disk(&path, &text);
            }
        }
    }
    if export {
        let html = app.compare_mgr.session(id).and_then(|s| s.export_html());
        if let Some(html) = html {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("HTML", &["html"])
                .save_file()
            {
                let op = app.tr().err_op_export;
                let _ = app.write_file_with_feedback(&path, &html, op);
            }
        }
    }

    if let Some(entry_index) = open_file {
        try_open_nested_compare(app, id, entry_index, false);
    }
    if let Some(entry_index) = open_binary {
        try_open_nested_compare(app, id, entry_index, true);
    }
}

fn try_open_nested_compare(app: &mut RustpadApp, folder_session_id: u64, entry_index: usize, _binary_only: bool) {
    let paths = app
        .compare_mgr
        .session(folder_session_id)
        .and_then(|session| {
            let result = session.folder_result.as_ref()?;
            let entry = result.entries.get(entry_index)?;
            if entry.status == FileStatus::Identical {
                return None;
            }
            Some((entry.left_path.clone()?, entry.right_path.clone()?))
        });
    if let Some((left, right)) = paths {
        app.compare_mgr.open_with_paths(left, right);
    }
}
