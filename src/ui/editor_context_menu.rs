//! Right-click context menu for the main editor (Notepad-- style).

use eframe::egui;

use crate::app::RustpadApp;
use crate::editor::context_actions::mark_line_color;
use crate::i18n::UiLanguage;

struct CtxLabels {
    cut: &'static str,
    copy: &'static str,
    paste: &'static str,
    delete: &'static str,
    select_all: &'static str,
    show_in_explorer: &'static str,
    mark_color: &'static str,
    display_blank: &'static str,
    display_non_print: &'static str,
    space_tab: &'static str,
    wrap_char: &'static str,
    insert: &'static str,
    insert_datetime: &'static str,
    clear_bookmarks: &'static str,
    toggle_line_comment: &'static str,
    add_block_comment: &'static str,
    del_block_comment: &'static str,
    markdown_view: &'static str,
    md_to_html: &'static str,
    text_to_html: &'static str,
    word_count: &'static str,
    select_line_output: &'static str,
    fold_menu: &'static str,
    fold_current: &'static str,
    unfold_current: &'static str,
    fold_all: &'static str,
    unfold_all: &'static str,
    pinlang_console: &'static str,
    user_menu: &'static str,
    color_red: &'static str,
    color_blue: &'static str,
    color_green: &'static str,
    color_yellow: &'static str,
    color_purple: &'static str,
}

/// Actions that close the menu and run after the popup is dismissed.
#[derive(Debug, Clone, Copy)]
enum MenuAction {
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    ShowInExplorer,
    MarkColor(u8),
    WrapChar,
    InsertDatetime,
    ClearBookmarks,
    ToggleLineComment,
    AddBlockComment,
    DelBlockComment,
    MarkdownView,
    MdToHtml,
    TextToHtml,
    WordCount,
    SelectLineOutput,
    FoldCurrent,
    UnfoldCurrent,
    FoldAll,
    UnfoldAll,
}

fn labels(lang: &str) -> CtxLabels {
    if UiLanguage::parse(lang) == UiLanguage::Zh {
        CtxLabels {
            cut: "剪切",
            copy: "复制",
            paste: "粘贴",
            delete: "删除",
            select_all: "全选",
            show_in_explorer: "在资源管理器中显示",
            mark_color: "颜色标记",
            display_blank: "显示空白字符",
            display_non_print: "显示不可打印字符",
            space_tab: "Tab 显示为空格",
            wrap_char: "按字符换行",
            insert: "插入",
            insert_datetime: "插入日期时间",
            clear_bookmarks: "清除所有书签",
            toggle_line_comment: "添加/删除行注释",
            add_block_comment: "添加块注释",
            del_block_comment: "删除块注释",
            markdown_view: "Markdown 预览",
            md_to_html: "Markdown 转 HTML",
            text_to_html: "文本转 HTML",
            word_count: "字数统计",
            select_line_output: "选中行输出到文件",
            fold_menu: "折叠",
            fold_current: "折叠当前",
            unfold_current: "展开当前",
            fold_all: "全部折叠",
            unfold_all: "全部展开",
            pinlang_console: "Pinlang 控制台",
            user_menu: "用户自定义右键菜单",
            color_red: "红色",
            color_blue: "蓝色",
            color_green: "绿色",
            color_yellow: "黄色",
            color_purple: "紫色",
        }
    } else {
        CtxLabels {
            cut: "Cut",
            copy: "Copy",
            paste: "Paste",
            delete: "Delete",
            select_all: "Select All",
            show_in_explorer: "Show File in Explorer",
            mark_color: "mark with color",
            display_blank: "Display blank chars",
            display_non_print: "Display Non Prints chars",
            space_tab: "Space replacement tab",
            wrap_char: "Wrap by character",
            insert: "Insert",
            insert_datetime: "Date-time",
            clear_bookmarks: "ClearAll BookMark",
            toggle_line_comment: "Add/Del line comment",
            add_block_comment: "Add Block comment",
            del_block_comment: "Del Block comment",
            markdown_view: "Markdown View",
            md_to_html: "Markdown to Html",
            text_to_html: "Text to Html",
            word_count: "Word Count",
            select_line_output: "Select Line or output to file",
            fold_menu: "Fold or UnFold",
            fold_current: "Fold Current",
            unfold_current: "Unfold Current",
            fold_all: "Fold All",
            unfold_all: "Unfold All",
            pinlang_console: "show pinlang console",
            user_menu: "User defined right-click menu",
            color_red: "Red",
            color_blue: "Blue",
            color_green: "Green",
            color_yellow: "Yellow",
            color_purple: "Purple",
        }
    }
}

fn dispatch_action(app: &mut RustpadApp, action: MenuAction) {
    match action {
        MenuAction::Cut => app.cut(),
        MenuAction::Copy => app.copy(),
        MenuAction::Paste => app.paste(),
        MenuAction::Delete => app.editor_delete(),
        MenuAction::SelectAll => app.select_all(),
        MenuAction::ShowInExplorer => app.show_file_in_explorer(),
        MenuAction::MarkColor(idx) => app.mark_selection_with_color(idx),
        MenuAction::WrapChar => app.toggle_wrap_by_character(),
        MenuAction::InsertDatetime => app.insert_current_datetime(),
        MenuAction::ClearBookmarks => app.clear_all_bookmarks(),
        MenuAction::ToggleLineComment => app.toggle_line_comment(),
        MenuAction::AddBlockComment => app.add_block_comment(),
        MenuAction::DelBlockComment => app.remove_block_comment(),
        MenuAction::MarkdownView => app.open_markdown_preview(),
        MenuAction::MdToHtml => app.export_markdown_to_html(),
        MenuAction::TextToHtml => app.export_text_to_html(),
        MenuAction::WordCount => app.show_word_count(),
        MenuAction::SelectLineOutput => app.output_selection_to_file(),
        MenuAction::FoldCurrent => app.fold_at_cursor(),
        MenuAction::UnfoldCurrent => app.unfold_at_cursor(),
        MenuAction::FoldAll => app.fold_all(),
        MenuAction::UnfoldAll => app.unfold_all(),
    }
}

/// Show the editor right-click menu using a state-managed Area popup.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_context_menu {
        return;
    }

    let lang = app.config.ui.ui_language.clone();
    let l = labels(&lang);
    let has_file = app.tab_manager.active().file_path.is_some();
    let pos = app.context_menu_pos;
    let has_selection = !app.effective_context_selection().is_empty();

    let mut pending_action: Option<MenuAction> = None;

    let area_response = egui::Area::new(egui::Id::new("__editor_context_menu"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .interactable(true)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(180.0);

                ui.add_enabled_ui(has_selection, |ui| {
                    if ui.button(l.cut).clicked() {
                        pending_action = Some(MenuAction::Cut);
                    }
                    if ui.button(l.copy).clicked() {
                        pending_action = Some(MenuAction::Copy);
                    }
                });
                if ui.button(l.paste).clicked() {
                    pending_action = Some(MenuAction::Paste);
                }
                ui.add_enabled_ui(has_selection, |ui| {
                    if ui.button(l.delete).clicked() {
                        pending_action = Some(MenuAction::Delete);
                    }
                });
                ui.separator();
                if ui.button(l.select_all).clicked() {
                    pending_action = Some(MenuAction::SelectAll);
                }
                ui.separator();

                if has_file && ui.button(l.show_in_explorer).clicked() {
                    pending_action = Some(MenuAction::ShowInExplorer);
                }

                ui.label(l.mark_color);
                let colors = [
                    (0u8, l.color_red),
                    (1, l.color_blue),
                    (2, l.color_green),
                    (3, l.color_yellow),
                    (4, l.color_purple),
                ];
                ui.horizontal(|ui| {
                    for (idx, _name) in colors {
                        let (r, g, b) = mark_line_color(idx);
                        let swatch = egui::Color32::from_rgb(r, g, b);
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("  ").background_color(swatch))
                                    .min_size(egui::vec2(28.0, 22.0)),
                            )
                            .clicked()
                        {
                            pending_action = Some(MenuAction::MarkColor(idx));
                        }
                    }
                });

                let mut blank = app.config.editor.display_blank_chars;
                if ui.checkbox(&mut blank, l.display_blank).changed() {
                    app.config.editor.display_blank_chars = blank;
                }
                let mut non_print = app.config.editor.display_non_print_chars;
                if ui.checkbox(&mut non_print, l.display_non_print).changed() {
                    app.config.editor.display_non_print_chars = non_print;
                }
                let mut tabs_as_space = app.config.editor.show_tabs_as_spaces;
                if ui.checkbox(&mut tabs_as_space, l.space_tab).changed() {
                    app.config.editor.show_tabs_as_spaces = tabs_as_space;
                }
                if ui.button(l.wrap_char).clicked() {
                    pending_action = Some(MenuAction::WrapChar);
                }

                ui.separator();

                ui.menu_button(l.insert, |ui| {
                    if ui.button(l.insert_datetime).clicked() {
                        pending_action = Some(MenuAction::InsertDatetime);
                    }
                });

                if ui.button(l.clear_bookmarks).clicked() {
                    pending_action = Some(MenuAction::ClearBookmarks);
                }
                if ui.button(l.toggle_line_comment).clicked() {
                    pending_action = Some(MenuAction::ToggleLineComment);
                }
                if ui.button(l.add_block_comment).clicked() {
                    pending_action = Some(MenuAction::AddBlockComment);
                }
                if ui.button(l.del_block_comment).clicked() {
                    pending_action = Some(MenuAction::DelBlockComment);
                }

                ui.separator();

                if ui.button(l.markdown_view).clicked() {
                    pending_action = Some(MenuAction::MarkdownView);
                }
                if ui.button(l.md_to_html).clicked() {
                    pending_action = Some(MenuAction::MdToHtml);
                }
                if ui.button(l.text_to_html).clicked() {
                    pending_action = Some(MenuAction::TextToHtml);
                }
                if ui.button(l.word_count).clicked() {
                    pending_action = Some(MenuAction::WordCount);
                }

                ui.separator();

                if ui.button(l.select_line_output).clicked() {
                    pending_action = Some(MenuAction::SelectLineOutput);
                }

                ui.label(l.fold_menu);
                if ui.button(l.fold_current).clicked() {
                    pending_action = Some(MenuAction::FoldCurrent);
                }
                if ui.button(l.unfold_current).clicked() {
                    pending_action = Some(MenuAction::UnfoldCurrent);
                }
                if ui.button(l.fold_all).clicked() {
                    pending_action = Some(MenuAction::FoldAll);
                }
                if ui.button(l.unfold_all).clicked() {
                    pending_action = Some(MenuAction::UnfoldAll);
                }

                ui.add_enabled_ui(false, |ui| {
                    let _ = ui.button(l.pinlang_console);
                });

                ui.menu_button(l.user_menu, |ui| {
                    ui.label(if UiLanguage::parse(&lang) == UiLanguage::Zh {
                        "（可在后续版本自定义）"
                    } else {
                        "(customize in a future version)"
                    });
                });
            });
        });

    if let Some(action) = pending_action {
        app.show_context_menu = false;
        dispatch_action(app, action);
        ctx.request_repaint();
        return;
    }

    // Dismiss when left-clicking outside the menu (right-click keeps it open).
    if ctx.input(|i| i.pointer.primary_pressed()) {
        let pointer_pos = ctx.input(|i| i.pointer.latest_pos());
        let inside = pointer_pos
            .map(|p| area_response.response.rect.contains(p))
            .unwrap_or(false);
        if !inside {
            app.show_context_menu = false;
            app.context_menu_selection = None;
        }
    }
}
