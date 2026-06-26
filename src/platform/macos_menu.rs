//! Native macOS menu bar (system menu after the RustPad app name).
//!
//! Menu **content** matches the in-window menu in `ui/menu.rs`; only placement differs
//! (system menu bar for notch / MacBook Air M4 safe area).

use std::sync::mpsc::Receiver;

use eframe::egui;
use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};

use crate::app::RustpadApp;
use crate::config::{AppConfig, ComparePair};
use crate::editor::EncodingProfile;
use crate::highlight::MENU_LANGUAGES;
use crate::i18n::Locale;
use crate::ui::compare_menu::{self, HISTORY_MAX};
use crate::ui::menu_actions::MENU_FONT_SIZES;

fn cmd_accel(code: Code) -> Accelerator {
    Accelerator::new(Some(Modifiers::SUPER), code)
}

fn shift_cmd_accel(code: Code) -> Accelerator {
    Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), code)
}

/// Strip shortcut hints baked into egui menu labels (e.g. "新建  Ctrl+N" → "新建").
fn menu_label(text: &str) -> &str {
    text.split("  ").next().unwrap_or(text)
}

fn item(id: &str, text: &str, accel: Option<Accelerator>) -> MenuItem {
    MenuItem::with_id(id, menu_label(text), true, accel)
}

/// Handles for compare history menu slots (updated when history changes).
pub struct MacosCompareHistory {
    pub file_items: Vec<MenuItem>,
    pub folder_items: Vec<MenuItem>,
}

/// Handles for native menu items that must stay in sync with editor state.
pub struct MacosMenuInstall {
    pub menu: Menu,
    pub rx: Receiver<MenuEvent>,
    pub encoding_open_checks: Vec<(EncodingProfile, CheckMenuItem)>,
    pub compare_history: MacosCompareHistory,
}

fn encoding_open_check(
    profile: EncodingProfile,
    text: String,
    has_file: bool,
    current: EncodingProfile,
) -> CheckMenuItem {
    CheckMenuItem::with_id(
        format!("enc.open.{}", profile.menu_id()),
        text,
        has_file,
        current == profile,
        None,
    )
}

/// Install the application menu bar and return handles for runtime sync.
pub fn install(
    t: &Locale,
    current_encoding: EncodingProfile,
    has_open_file: bool,
    config: &AppConfig,
) -> MacosMenuInstall {
    let (tx, rx) = std::sync::mpsc::channel();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = tx.send(event);
    }));

    let menu = Menu::new();
    let mut encoding_open_checks = Vec::new();
    let mut compare_history = MacosCompareHistory {
        file_items: Vec::new(),
        folder_items: Vec::new(),
    };
    if let Err(err) = build_menu(
        &menu,
        t,
        current_encoding,
        has_open_file,
        config,
        &mut encoding_open_checks,
        &mut compare_history,
    ) {
        log::error!("Failed to build macOS menu bar: {err}");
    }
    menu.init_for_nsapp();
    MacosMenuInstall {
        menu,
        rx,
        encoding_open_checks,
        compare_history,
    }
}

fn build_menu(
    menu: &Menu,
    t: &Locale,
    current_encoding: EncodingProfile,
    has_open_file: bool,
    config: &AppConfig,
    encoding_open_checks: &mut Vec<(EncodingProfile, CheckMenuItem)>,
    compare_history: &mut MacosCompareHistory,
) -> muda::Result<()> {
    // macOS application menu (standard); does not replace File/Settings/Help entries.
    let app_menu = Submenu::with_items(
        "RustPad",
        true,
        &[
            &item("app.about", t.help_about, None),
            &item(
                "app.preferences",
                t.settings_preferences,
                Some(cmd_accel(Code::Comma)),
            ),
            &item("app.keybindings", t.settings_keybindings, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &item("app.quit", t.file_exit, Some(cmd_accel(Code::KeyQ))),
        ],
    )?;
    menu.append(&app_menu)?;

    let file_menu = Submenu::with_items(
        t.menu_file,
        true,
        &[
            &item("file.new", t.file_new, Some(cmd_accel(Code::KeyN))),
            &item("file.open", t.file_open, Some(cmd_accel(Code::KeyO))),
            &PredefinedMenuItem::separator(),
            &item("file.save", t.file_save, Some(cmd_accel(Code::KeyS))),
            &item("file.save_as", t.file_save_as, Some(shift_cmd_accel(Code::KeyS))),
            &item("file.save_all", t.file_save_all, None),
            &PredefinedMenuItem::separator(),
            &item("file.close_tab", t.file_close_tab, Some(cmd_accel(Code::KeyW))),
            &PredefinedMenuItem::separator(),
            &item("file.exit", t.file_exit, Some(cmd_accel(Code::KeyQ))),
        ],
    )?;
    menu.append(&file_menu)?;

    let edit_menu = Submenu::with_items(
        t.menu_edit,
        true,
        &[
            &item("edit.undo", t.edit_undo, Some(cmd_accel(Code::KeyZ))),
            &item("edit.redo", t.edit_redo, Some(shift_cmd_accel(Code::KeyZ))),
            &PredefinedMenuItem::separator(),
            &item("edit.cut", t.edit_cut, Some(cmd_accel(Code::KeyX))),
            &item("edit.copy", t.edit_copy, Some(cmd_accel(Code::KeyC))),
            &item("edit.paste", t.edit_paste, Some(cmd_accel(Code::KeyV))),
            &item("edit.select_all", t.edit_select_all, Some(cmd_accel(Code::KeyA))),
            &PredefinedMenuItem::separator(),
            &item("edit.find", t.edit_find, Some(cmd_accel(Code::KeyF))),
            &item("edit.replace", t.edit_replace, Some(cmd_accel(Code::KeyH))),
            &item("edit.goto_line", t.edit_goto_line, Some(cmd_accel(Code::KeyG))),
            &PredefinedMenuItem::separator(),
            &item("edit.copy_column", t.edit_copy_column, None),
        ],
    )?;
    menu.append(&edit_menu)?;

    let view_menu = Submenu::new(t.menu_view, true);
    view_menu.append(&item(
        "view.sidebar",
        t.view_toggle_sidebar,
        Some(cmd_accel(Code::KeyB)),
    ))?;
    view_menu.append(&item("view.minimap", t.view_toggle_minimap, None))?;
    view_menu.append(&PredefinedMenuItem::separator())?;

    let lang_menu = Submenu::new(t.view_language, true);
    lang_menu.append(&MenuItem::with_id(
        "view.lang.auto",
        t.view_auto_detect,
        true,
        None,
    ))?;
    lang_menu.append(&PredefinedMenuItem::separator())?;
    for (index, lang) in MENU_LANGUAGES.iter().enumerate() {
        lang_menu.append(&MenuItem::with_id(
            format!("view.lang.{index}"),
            *lang,
            true,
            None,
        ))?;
    }
    view_menu.append(&lang_menu)?;
    view_menu.append(&PredefinedMenuItem::separator())?;

    let font_menu = Submenu::new(t.view_font_size, true);
    for size in MENU_FONT_SIZES {
        font_menu.append(&MenuItem::with_id(
            format!("view.font.{size}"),
            format!("{size}px"),
            true,
            None,
        ))?;
    }
    view_menu.append(&font_menu)?;
    view_menu.append(&PredefinedMenuItem::separator())?;
    view_menu.append(&item("view.word_wrap", t.view_word_wrap, None))?;
    view_menu.append(&item("view.line_numbers", t.view_line_numbers, None))?;
    menu.append(&view_menu)?;

    let encoding_menu = Submenu::new(t.menu_encoding, true);
    let open_menu = Submenu::new(t.enc_open_section, true);
    for profile in EncodingProfile::MAIN {
        let check = encoding_open_check(
            profile,
            t.enc_open_with(profile),
            has_open_file,
            current_encoding,
        );
        open_menu.append(&check)?;
        encoding_open_checks.push((profile, check));
    }
    let open_more = Submenu::new(t.enc_more, true);
    for profile in EncodingProfile::MORE {
        let check = encoding_open_check(
            profile,
            t.enc_open_with(profile),
            has_open_file,
            current_encoding,
        );
        open_more.append(&check)?;
        encoding_open_checks.push((profile, check));
    }
    open_menu.append(&open_more)?;
    encoding_menu.append(&open_menu)?;
    encoding_menu.append(&PredefinedMenuItem::separator())?;

    for profile in EncodingProfile::MAIN {
        encoding_menu.append(&MenuItem::with_id(
            format!("enc.convert.{}", profile.menu_id()),
            t.enc_convert_to(profile),
            true,
            None,
        ))?;
    }
    let convert_more = Submenu::new(t.enc_convert_more, true);
    for profile in EncodingProfile::MORE {
        convert_more.append(&MenuItem::with_id(
            format!("enc.convert.{}", profile.menu_id()),
            t.enc_convert_to(profile),
            true,
            None,
        ))?;
    }
    encoding_menu.append(&convert_more)?;
    encoding_menu.append(&PredefinedMenuItem::separator())?;
    encoding_menu.append(&MenuItem::with_id(
        "enc.batch",
        t.enc_batch_convert,
        true,
        None,
    ))?;
    menu.append(&encoding_menu)?;

    let compare_menu = build_compare_menu(t, compare_history)?;
    menu.append(&compare_menu)?;
    sync_compare_history_items(compare_history, config, t);

    let tools_menu = Submenu::with_items(
        t.menu_tools,
        true,
        &[&item("tools.macro", t.tools_macro, None)],
    )?;
    menu.append(&tools_menu)?;

    let settings_menu = Submenu::with_items(
        t.menu_settings,
        true,
        &[
            &item("settings.preferences", t.settings_preferences, None),
            &item("settings.keybindings", t.settings_keybindings, None),
        ],
    )?;
    menu.append(&settings_menu)?;

    let help_menu = Submenu::with_items(
        t.menu_help,
        true,
        &[&item("help.about", t.help_about, None)],
    )?;
    menu.append(&help_menu)?;

    Ok(())
}

fn build_compare_menu(
    t: &Locale,
    compare_history: &mut MacosCompareHistory,
) -> muda::Result<Submenu> {
    let compare_menu = Submenu::new(t.menu_compare, true);
    compare_menu.append(&item(
        "compare.files",
        t.cmp_menu_files,
        Some(cmd_accel(Code::KeyD)),
    ))?;
    compare_menu.append(&item("compare.folders", t.cmp_menu_folders, None))?;

    let recent_menu = Submenu::new(t.cmp_recent, true);

    let folder_hist_menu = Submenu::new(t.cmp_history_folders, true);
    append_history_slots(
        &folder_hist_menu,
        "compare.history.folder.",
        &mut compare_history.folder_items,
    )?;
    recent_menu.append(&folder_hist_menu)?;

    let file_hist_menu = Submenu::new(t.cmp_history_files, true);
    append_history_slots(
        &file_hist_menu,
        "compare.history.file.",
        &mut compare_history.file_items,
    )?;
    recent_menu.append(&file_hist_menu)?;

    compare_menu.append(&recent_menu)?;
    Ok(compare_menu)
}

fn append_history_slots(
    submenu: &Submenu,
    id_prefix: &str,
    items: &mut Vec<MenuItem>,
) -> muda::Result<()> {
    for index in 0..HISTORY_MAX {
        let item = MenuItem::with_id(format!("{id_prefix}{index}"), "", false, None);
        submenu.append(&item)?;
        items.push(item);
    }
    Ok(())
}

fn sync_compare_history_items(
    handles: &MacosCompareHistory,
    config: &AppConfig,
    t: &Locale,
) {
    sync_history_slot(
        &handles.folder_items,
        &config.recent_folder_compares,
        t.cmp_history_empty,
    );
    sync_history_slot(
        &handles.file_items,
        &config.recent_file_compares,
        t.cmp_history_empty,
    );
}

fn sync_history_slot(items: &[MenuItem], history: &[ComparePair], empty_label: &str) {
    for (index, item) in items.iter().enumerate() {
        if let Some(pair) = history.get(index) {
            item.set_text(compare_menu::history_label(pair));
            item.set_enabled(true);
        } else if index == 0 && history.is_empty() {
            item.set_text(empty_label);
            item.set_enabled(false);
        } else {
            item.set_text("");
            item.set_enabled(false);
        }
    }
}

/// Keep compare history menu items aligned with persisted config.
pub fn sync_compare_history(app: &RustpadApp) {
    let Some(handles) = app.macos_compare_history.as_ref() else {
        return;
    };
    sync_compare_history_items(handles, &app.config, app.tr());
}

/// Keep native "open with encoding" checks aligned with the active tab.
pub fn sync_encoding_open_checks(app: &RustpadApp) {
    let current = app.tab_manager.active().encoding;
    let has_file = app.tab_manager.active().file_path.is_some();
    for (profile, item) in &app.macos_encoding_open_checks {
        item.set_enabled(has_file);
        item.set_checked(*profile == current);
    }
}

/// Drain pending native menu events and dispatch them to application actions.
pub fn drain_events(app: &mut RustpadApp, ctx: &egui::Context) {
    let ids: Vec<String> = match app.macos_menu_rx.as_ref() {
        Some(rx) => rx.try_iter().map(|event| event.id().0.clone()).collect(),
        None => return,
    };
    for id in ids {
        crate::ui::menu_actions::dispatch(app, &id, ctx);
    }
}
