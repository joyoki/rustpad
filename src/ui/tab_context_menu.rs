//! Right-click context menu for file tabs (Notepad++ style).

use eframe::egui;

use crate::app::RustpadApp;
use crate::i18n::UiLanguage;

/// Deferred tab context-menu action (applied after the menu closes).
#[derive(Debug, Clone, Copy)]
pub enum TabMenuAction {
    Close(usize),
    CloseOthers(usize),
    CloseLeft(usize),
    CloseRight(usize),
    RevealInFolder(usize),
    OpenTerminal(usize),
    LocateInTree(usize),
    CopyPath(usize),
    Rename(usize),
    SaveAs(usize),
    Reload(usize),
    AddFavorite(usize),
    CompareLeft(usize),
    CompareRight(usize),
}

struct TabMenuLabels {
    close_current: &'static str,
    close_others: &'static str,
    close_left: &'static str,
    close_right: &'static str,
    move_new_window: &'static str,
    move_to_window: &'static str,
    open_folder: &'static str,
    open_terminal: &'static str,
    locate_in_tree: &'static str,
    copy_path: &'static str,
    rename: &'static str,
    save_as: &'static str,
    reopen_text: &'static str,
    force_plain_text: &'static str,
    reopen_binary: &'static str,
    reload: &'static str,
    add_favorite: &'static str,
    compare_left: &'static str,
    compare_right: &'static str,
}

fn labels(lang: &str) -> TabMenuLabels {
    if UiLanguage::parse(lang) == UiLanguage::Zh {
        TabMenuLabels {
            close_current: "关闭当前文档",
            close_others: "除此之外全部关闭",
            close_left: "关闭左边所有文档",
            close_right: "关闭右边所有文档",
            move_new_window: "移出到新窗口中打开",
            move_to_window: "移出到窗口",
            open_folder: "打开所在文件夹",
            open_terminal: "在文件目录打开CMD窗口",
            locate_in_tree: "定位在文件夹列表中",
            copy_path: "拷贝文本路径到剪切板",
            rename: "重命名当前文件",
            save_as: "当前文件另存为",
            reopen_text: "重新以文本模式打开",
            force_plain_text: "强制以普通文本加载",
            reopen_binary: "重新以二进制模式打开",
            reload: "重加载文件",
            add_favorite: "添加到收藏夹",
            compare_left: "选择为左边对比文件",
            compare_right: "选择为右边对比文件",
        }
    } else {
        TabMenuLabels {
            close_current: "Close",
            close_others: "Close Others",
            close_left: "Close Tabs to the Left",
            close_right: "Close Tabs to the Right",
            move_new_window: "Move to New Window",
            move_to_window: "Move to Window",
            open_folder: "Open Containing Folder",
            open_terminal: "Open Command Prompt Here",
            locate_in_tree: "Locate in File Explorer",
            copy_path: "Copy Path to Clipboard",
            rename: "Rename",
            save_as: "Save As",
            reopen_text: "Reopen in Text Mode",
            force_plain_text: "Force Load as Plain Text",
            reopen_binary: "Reopen in Binary Mode",
            reload: "Reload from Disk",
            add_favorite: "Add to Favorites",
            compare_left: "Select as Left Compare File",
            compare_right: "Select as Right Compare File",
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn terminal_label(lang: &str) -> &'static str {
    if UiLanguage::parse(lang) == UiLanguage::Zh {
        "在文件目录打开终端"
    } else {
        "Open Terminal Here"
    }
}

/// Show the tab context menu; returns an action to run after the panel finishes layout.
pub fn show(
    ui: &mut egui::Ui,
    app: &RustpadApp,
    tab_index: usize,
    tab_count: usize,
) -> Option<TabMenuAction> {
    let lang = app.config.ui.ui_language.clone();
    let l = labels(&lang);
    let has_file = app
        .tab_manager
        .tabs()
        .get(tab_index)
        .and_then(|t| t.file_path.as_ref())
        .is_some();
    let can_locate = has_file
        && app
            .tab_manager
            .tabs()
            .get(tab_index)
            .and_then(|t| t.file_path.as_ref())
            .and_then(|p| app.workspace_root.as_ref().map(|r| p.starts_with(r)))
            .unwrap_or(false);

    if ui.button(l.close_current).clicked() {
        ui.close_menu();
        return Some(TabMenuAction::Close(tab_index));
    }
    if ui
        .add_enabled(tab_count > 1, egui::Button::new(l.close_others))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::CloseOthers(tab_index));
    }
    if ui
        .add_enabled(tab_index > 0, egui::Button::new(l.close_left))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::CloseLeft(tab_index));
    }
    if ui
        .add_enabled(
            tab_index + 1 < tab_count,
            egui::Button::new(l.close_right),
        )
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::CloseRight(tab_index));
    }

    ui.separator();

    ui.add_enabled(false, egui::Button::new(l.move_new_window));
    ui.menu_button(l.move_to_window, |ui| {
        ui.add_enabled(false, egui::Button::new("—"));
    });

    ui.separator();

    if ui
        .add_enabled(has_file, egui::Button::new(l.open_folder))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::RevealInFolder(tab_index));
    }
    let term_label = {
        #[cfg(target_os = "windows")]
        {
            l.open_terminal
        }
        #[cfg(not(target_os = "windows"))]
        {
            terminal_label(&lang)
        }
    };
    if ui
        .add_enabled(has_file, egui::Button::new(term_label))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::OpenTerminal(tab_index));
    }
    if ui
        .add_enabled(can_locate, egui::Button::new(l.locate_in_tree))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::LocateInTree(tab_index));
    }

    ui.separator();

    if ui
        .add_enabled(has_file, egui::Button::new(l.copy_path))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::CopyPath(tab_index));
    }
    if ui
        .add_enabled(has_file, egui::Button::new(l.rename))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::Rename(tab_index));
    }
    if ui.button(l.save_as).clicked() {
        ui.close_menu();
        return Some(TabMenuAction::SaveAs(tab_index));
    }

    ui.separator();

    if ui
        .add_enabled(has_file, egui::Button::new(l.reopen_text))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::Reload(tab_index));
    }
    ui.add_enabled(false, egui::Button::new(l.force_plain_text));
    ui.add_enabled(false, egui::Button::new(l.reopen_binary));
    if ui
        .add_enabled(has_file, egui::Button::new(l.reload))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::Reload(tab_index));
    }

    ui.separator();

    if ui
        .add_enabled(has_file, egui::Button::new(l.add_favorite))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::AddFavorite(tab_index));
    }

    ui.separator();

    if ui
        .add_enabled(has_file, egui::Button::new(l.compare_left))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::CompareLeft(tab_index));
    }
    if ui
        .add_enabled(has_file, egui::Button::new(l.compare_right))
        .clicked()
    {
        ui.close_menu();
        return Some(TabMenuAction::CompareRight(tab_index));
    }

    None
}

/// Apply a deferred tab context-menu action.
pub fn dispatch(app: &mut RustpadApp, action: TabMenuAction) {
    match action {
        TabMenuAction::Close(i) => app.close_tab_at(i),
        TabMenuAction::CloseOthers(i) => app.close_other_tabs(i),
        TabMenuAction::CloseLeft(i) => app.close_tabs_to_left(i),
        TabMenuAction::CloseRight(i) => app.close_tabs_to_right(i),
        TabMenuAction::RevealInFolder(i) => app.reveal_tab_in_folder(i),
        TabMenuAction::OpenTerminal(i) => app.open_terminal_for_tab(i),
        TabMenuAction::LocateInTree(i) => app.locate_tab_in_file_tree(i),
        TabMenuAction::CopyPath(i) => app.copy_tab_path_to_clipboard(i),
        TabMenuAction::Rename(i) => app.begin_rename_tab(i),
        TabMenuAction::SaveAs(i) => app.save_tab_as_at(i),
        TabMenuAction::Reload(i) => app.reload_tab_from_disk(i),
        TabMenuAction::AddFavorite(i) => app.add_tab_to_favorites(i),
        TabMenuAction::CompareLeft(i) => app.compare_tab_as_left(i),
        TabMenuAction::CompareRight(i) => app.compare_tab_as_right(i),
    }
}
