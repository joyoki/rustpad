//! Shared menu action dispatch (in-window menu and macOS native menu bar).

use eframe::egui;

use crate::app::RustpadApp;
use crate::editor::EncodingProfile;
use crate::highlight::MENU_LANGUAGES;
use crate::ui::encoding_menu::EncodingMenuAction;

/// Font sizes offered under View → Font Size.
pub const MENU_FONT_SIZES: [u32; 7] = [10, 12, 14, 16, 18, 20, 24];

/// Apply a menu action by stable id (see `macos_menu` ids).
pub fn dispatch(app: &mut RustpadApp, id: &str, ctx: &egui::Context) {
    match id {
        "app.about" | "help.about" => app.show_about = true,
        "app.preferences" | "settings.preferences" => app.show_preferences = true,
        "app.keybindings" | "settings.keybindings" => {
            crate::ui::keybindings_dialog::open_editor(app);
        }
        "app.quit" | "file.exit" => app.request_exit(ctx),

        "file.new" => {
            let _ = app.tab_manager.new_tab();
        }
        "file.open" => app.pending_open_file = true,
        "file.save" => app.save_current_tab(),
        "file.save_as" => app.save_as_dialog(),
        "file.save_all" => app.save_all_tabs(),
        "file.close_tab" => app.close_current_tab(),
        "file.compare" => app.pending_compare_files = true,
        "file.compare_current" => app.pending_compare_current = true,

        "edit.undo" => {
            app.tab_manager.active_mut().buffer.undo();
        }
        "edit.redo" => {
            app.tab_manager.active_mut().buffer.redo();
        }
        "edit.cut" => app.cut(),
        "edit.copy" => app.copy(),
        "edit.paste" => app.paste(),
        "edit.select_all" => app.select_all(),
        "edit.find" => app.open_find(false),
        "edit.replace" => app.open_find(true),
        "edit.goto_line" => app.show_goto_line = true,
        "edit.copy_column" => app.copy_column(),

        "view.sidebar" => app.show_sidebar = !app.show_sidebar,
        "view.minimap" => app.config.ui.show_minimap = !app.config.ui.show_minimap,
        "view.word_wrap" => app.config.editor.word_wrap = !app.config.editor.word_wrap,
        "view.line_numbers" => {
            app.config.editor.show_line_numbers = !app.config.editor.show_line_numbers;
        }
        "view.lang.auto" => app.clear_active_language(),

        "enc.batch" => app.show_batch_encoding = true,

        "tools.compare" => app.pending_compare_files = true,
        "tools.macro" => app.toggle_macro_recording(),

        id if id.starts_with("view.lang.") => {
            if let Some(rest) = id.strip_prefix("view.lang.") {
                if let Ok(index) = rest.parse::<usize>() {
                    if let Some(&lang) = MENU_LANGUAGES.get(index) {
                        app.set_active_language(lang);
                    }
                }
            }
        }
        id if id.starts_with("view.font.") => {
            if let Some(rest) = id.strip_prefix("view.font.") {
                if let Ok(size) = rest.parse::<u32>() {
                    app.config.editor.font_size = size as f32;
                    app.toolbar_font_size_text = size.to_string();
                    app.toolbar_font_size_editing = false;
                }
            }
        }
        id if id.starts_with("enc.open.") => {
            if let Some(rest) = id.strip_prefix("enc.open.") {
                if let Some(profile) = EncodingProfile::from_menu_id(rest) {
                    app.open_with_encoding(profile);
                }
            }
        }
        id if id.starts_with("enc.convert.") => {
            if let Some(rest) = id.strip_prefix("enc.convert.") {
                if let Some(profile) = EncodingProfile::from_menu_id(rest) {
                    app.convert_to_encoding(profile);
                }
            }
        }

        _ => log::debug!("Unhandled menu action: {id}"),
    }
    ctx.request_repaint();
}

/// Apply encoding submenu result (egui encoding dropdown).
pub fn apply_encoding_action(app: &mut RustpadApp, action: EncodingMenuAction) {
    match action {
        EncodingMenuAction::OpenWith(profile) => app.open_with_encoding(profile),
        EncodingMenuAction::ConvertTo(profile) => app.convert_to_encoding(profile),
        EncodingMenuAction::BatchConvert => app.show_batch_encoding = true,
        EncodingMenuAction::None => {}
    }
}
