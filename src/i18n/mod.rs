//! Application UI strings (English / 中文).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLanguage {
    En,
    Zh,
}

impl UiLanguage {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "zh" | "zh-cn" | "chinese" | "中文" => Self::Zh,
            _ => Self::En,
        }
    }

    pub fn as_config_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Zh => "中文",
        }
    }
}

/// Localized UI strings for one language.
pub struct Locale {
    pub app_title: &'static str,
    // Menus
    pub menu_file: &'static str,
    pub menu_edit: &'static str,
    pub menu_view: &'static str,
    pub menu_encoding: &'static str,
    pub menu_settings: &'static str,
    pub menu_tools: &'static str,
    pub menu_help: &'static str,
    pub file_new: &'static str,
    pub file_open: &'static str,
    pub file_save: &'static str,
    pub file_save_as: &'static str,
    pub file_save_all: &'static str,
    pub file_close_tab: &'static str,
    pub file_compare: &'static str,
    pub file_compare_current: &'static str,
    pub file_exit: &'static str,
    pub edit_undo: &'static str,
    pub edit_redo: &'static str,
    pub edit_cut: &'static str,
    pub edit_copy: &'static str,
    pub edit_paste: &'static str,
    pub edit_select_all: &'static str,
    pub edit_find: &'static str,
    pub edit_replace: &'static str,
    pub edit_goto_line: &'static str,
    pub edit_copy_column: &'static str,
    pub view_toggle_sidebar: &'static str,
    pub view_toggle_minimap: &'static str,
    pub view_language: &'static str,
    pub view_auto_detect: &'static str,
    pub view_font_size: &'static str,
    pub view_word_wrap: &'static str,
    pub view_line_numbers: &'static str,
    pub tools_compare: &'static str,
    pub tools_macro: &'static str,
    pub tools_preferences: &'static str,
    pub settings_preferences: &'static str,
    pub settings_keybindings: &'static str,
    pub help_about: &'static str,
    // Toolbar
    pub tb_new: &'static str,
    pub tb_open: &'static str,
    pub tb_save: &'static str,
    pub tb_undo: &'static str,
    pub tb_redo: &'static str,
    pub tb_find: &'static str,
    pub tb_compare: &'static str,
    pub tb_encoding: &'static str,
    pub tip_new: &'static str,
    pub tip_open: &'static str,
    pub tip_save: &'static str,
    pub tip_undo: &'static str,
    pub tip_redo: &'static str,
    pub tip_find: &'static str,
    pub tip_compare: &'static str,
    pub tip_encoding: &'static str,
    pub tip_font_dec: &'static str,
    pub tip_font_inc: &'static str,
    pub tip_font_size: &'static str,
    pub tip_tab_scroll_left: &'static str,
    pub tip_tab_scroll_right: &'static str,
    // Dialogs
    pub dlg_preferences: &'static str,
    pub dlg_about: &'static str,
    pub dlg_goto_line: &'static str,
    pub dlg_unsaved: &'static str,
    pub dlg_quit_save: &'static str,
    pub pref_editor: &'static str,
    pub pref_ui: &'static str,
    pub pref_font_size: &'static str,
    pub pref_tab_size: &'static str,
    pub pref_show_line_numbers: &'static str,
    pub pref_word_wrap: &'static str,
    pub pref_auto_indent: &'static str,
    pub pref_highlight_line: &'static str,
    pub pref_show_minimap: &'static str,
    pub pref_theme: &'static str,
    pub pref_ui_language: &'static str,
    pub pref_save: &'static str,
    pub theme_light: &'static str,
    pub theme_dark: &'static str,
    pub goto_line_number: &'static str,
    pub about_tagline: &'static str,
    pub about_powered: &'static str,
    pub about_syntax: &'static str,
    pub about_project: &'static str,
    pub about_releases: &'static str,
    pub btn_save: &'static str,
    pub btn_dont_save: &'static str,
    pub btn_cancel: &'static str,
    pub btn_save_all: &'static str,
    pub btn_close: &'static str,
    pub btn_clear: &'static str,
    // Status bar
    pub status_lines: &'static str,
    pub status_lang: &'static str,
    // Search
    pub find_replace_title: &'static str,
    pub find_tab: &'static str,
    pub replace_tab: &'static str,
    pub dir_find: &'static str,
    pub find_what: &'static str,
    pub replace_with: &'static str,
    pub search_mode: &'static str,
    pub mode_normal: &'static str,
    pub mode_regex: &'static str,
    pub opt_backward: &'static str,
    pub opt_whole_word: &'static str,
    pub opt_match_case: &'static str,
    pub opt_wrap: &'static str,
    pub find_next: &'static str,
    pub find_prev: &'static str,
    pub find_count: &'static str,
    pub find_all_current: &'static str,
    pub find_all_open: &'static str,
    pub clear_result: &'static str,
    pub replace_one: &'static str,
    pub replace_all: &'static str,
    pub search_results: &'static str,
    pub no_search_results: &'static str,
    pub no_recent_searches: &'static str,
    pub find_in_files: &'static str,
    // Diff toolbar
    pub diff_open_left: &'static str,
    pub diff_open_right: &'static str,
    pub diff_swap: &'static str,
    pub diff_prev: &'static str,
    pub diff_next: &'static str,
    pub diff_ignore_ws: &'static str,
    pub diff_ignore_case: &'static str,
    pub diff_save_left: &'static str,
    pub diff_save_right: &'static str,
    pub diff_export: &'static str,
    pub diff_close: &'static str,
    pub tip_diff_prev: &'static str,
    pub tip_diff_next: &'static str,
    // Encoding menu
    pub enc_open_section: &'static str,
    pub enc_convert_section: &'static str,
    pub enc_more: &'static str,
    pub enc_convert_more: &'static str,
    pub enc_batch_convert: &'static str,
    pub dlg_batch_encoding: &'static str,
    pub enc_batch_prompt: &'static str,
    // Keybindings dialog
    pub dlg_keybindings: &'static str,
    pub kb_command_col: &'static str,
    pub kb_shortcut_col: &'static str,
    pub kb_change: &'static str,
    pub kb_reset: &'static str,
    pub kb_scheme: &'static str,
    pub kb_press_key: &'static str,
    pub kb_conflicts: &'static str,
    pub kb_saved: &'static str,
    pub tip_column_select: &'static str,
    // Quick scroll bar
    pub scroll_to_here: &'static str,
    pub scroll_top: &'static str,
    pub scroll_bottom: &'static str,
    pub scroll_page_up: &'static str,
    pub scroll_page_down: &'static str,
    pub scroll_up: &'static str,
    pub scroll_down: &'static str,
}

static LOCALE_EN: Locale = Locale {
    app_title: "RustPad",
    menu_file: "File",
    menu_edit: "Edit",
    menu_view: "View",
    menu_encoding: "Encoding",
    menu_settings: "Settings",
    menu_tools: "Tools",
    menu_help: "Help",
    file_new: "New  Ctrl+N",
    file_open: "Open...  Ctrl+O",
    file_save: "Save  Ctrl+S",
    file_save_as: "Save As...  Ctrl+Shift+S",
    file_save_all: "Save All",
    file_close_tab: "Close Tab  Ctrl+W",
    file_compare: "Compare Files...  Ctrl+D",
    file_compare_current: "Compare Current File With...",
    file_exit: "Exit  Alt+F4",
    edit_undo: "Undo  Ctrl+Z",
    edit_redo: "Redo  Ctrl+Y",
    edit_cut: "Cut  Ctrl+X",
    edit_copy: "Copy  Ctrl+C",
    edit_paste: "Paste  Ctrl+V",
    edit_select_all: "Select All  Ctrl+A",
    edit_find: "Find...  Ctrl+F",
    edit_replace: "Replace...  Ctrl+H",
    edit_goto_line: "Go to Line...  Ctrl+G",
    edit_copy_column: "Copy Column  Alt+Shift+C",
    view_toggle_sidebar: "Toggle Sidebar  Ctrl+B",
    view_toggle_minimap: "Toggle Minimap",
    view_language: "Language",
    view_auto_detect: "Auto Detect",
    view_font_size: "Font Size",
    view_word_wrap: "Word Wrap",
    view_line_numbers: "Line Numbers",
    tools_compare: "Compare Files...  Ctrl+D",
    tools_macro: "Macro Recording",
    tools_preferences: "Preferences...",
    settings_preferences: "Preferences...",
    settings_keybindings: "Keyboard Shortcuts...",
    help_about: "About RustPad",
    tb_new: "📄",
    tb_open: "📂",
    tb_save: "💾",
    tb_undo: "↩",
    tb_redo: "↪",
    tb_find: "🔍",
    tb_compare: "🆚",
    tb_encoding: "🌐",
    tip_new: "New file (Ctrl+N)",
    tip_open: "Open file (Ctrl+O)",
    tip_save: "Save (Ctrl+S)",
    tip_undo: "Undo (Ctrl+Z)",
    tip_redo: "Redo (Ctrl+Y)",
    tip_find: "Find (Ctrl+F)",
    tip_compare: "Compare files (Ctrl+D)",
    tip_encoding: "Open with / convert encoding",
    tip_font_dec: "Decrease font size",
    tip_font_inc: "Increase font size",
    tip_font_size: "Font size (8–72 px, type to set)",
    tip_tab_scroll_left: "Scroll tabs left",
    tip_tab_scroll_right: "Scroll tabs right",
    dlg_preferences: "Preferences",
    dlg_about: "About RustPad",
    dlg_goto_line: "Go to Line",
    dlg_unsaved: "Unsaved Changes",
    dlg_quit_save: "Save Changes?",
    pref_editor: "Editor",
    pref_ui: "UI",
    pref_font_size: "Font Size:",
    pref_tab_size: "Tab Size:",
    pref_show_line_numbers: "Show line numbers",
    pref_word_wrap: "Word wrap",
    pref_auto_indent: "Auto indent",
    pref_highlight_line: "Highlight current line",
    pref_show_minimap: "Show minimap",
    pref_theme: "Theme:",
    pref_ui_language: "UI Language:",
    pref_save: "Save Preferences",
    theme_light: "Light",
    theme_dark: "Dark",
    goto_line_number: "Line number:",
    about_tagline: "A modern code editor built with Rust",
    about_powered: "Powered by egui + eframe",
    about_syntax: "Syntax highlighting by syntect",
    about_project: "RustPad:",
    about_releases: "Latest Release:",
    btn_save: "Save",
    btn_dont_save: "Don't Save",
    btn_cancel: "Cancel",
    btn_save_all: "Save All",
    btn_close: "Close",
    btn_clear: "Clear",
    status_lines: "lines",
    status_lang: "Lang:",
    find_replace_title: "Find / Replace",
    find_tab: "Find",
    replace_tab: "Replace",
    dir_find: "Dir Find",
    find_what: "Find what :",
    replace_with: "Replace with :",
    search_mode: "Search Mode",
    mode_normal: "Normal",
    mode_regex: "Regular expression",
    opt_backward: "Backward direction",
    opt_whole_word: "Match whole word only",
    opt_match_case: "Match case",
    opt_wrap: "Wrap around",
    find_next: "Find Next  (F3)",
    find_prev: "Find Prev  (F4)",
    find_count: "Count",
    find_all_current: "Find All in Current Document",
    find_all_open: "Find All in All Opened Documents",
    clear_result: "Clear Result",
    replace_one: "Replace",
    replace_all: "Replace All",
    search_results: "Search results",
    no_search_results: "No results. Use \"Find All\" to populate this list.",
    no_recent_searches: "(no recent searches)",
    find_in_files: "Find in Files",
    diff_open_left: "Open Left…",
    diff_open_right: "Open Right…",
    diff_swap: "⇄ Swap",
    diff_prev: "⬆ Prev",
    diff_next: "⬇ Next",
    diff_ignore_ws: "Ignore Whitespace",
    diff_ignore_case: "Ignore Case",
    diff_save_left: "💾 Save Left",
    diff_save_right: "💾 Save Right",
    diff_export: "Export Report…",
    diff_close: "✖ Close Compare",
    tip_diff_prev: "Previous difference (F7)",
    tip_diff_next: "Next difference (F8)",
    enc_open_section: "Open with encoding",
    enc_convert_section: "Convert to encoding",
    enc_more: "More…",
    enc_convert_more: "Convert to more encodings",
    enc_batch_convert: "Batch convert encoding",
    dlg_batch_encoding: "Batch Convert Encoding",
    enc_batch_prompt: "Convert all open tabs to:",
    dlg_keybindings: "Keyboard Shortcuts",
    kb_command_col: "Command",
    kb_shortcut_col: "Shortcut",
    kb_change: "Change",
    kb_reset: "Reset to Default",
    kb_scheme: "Scheme",
    kb_press_key: "Press a key combination… (Esc to cancel)",
    kb_conflicts: "Conflicting shortcuts detected",
    kb_saved: "Shortcuts saved",
    tip_column_select: "Hold Alt and drag to select a column",
    scroll_to_here: "Scroll to here",
    scroll_top: "Top",
    scroll_bottom: "Bottom",
    scroll_page_up: "Previous page",
    scroll_page_down: "Next page",
    scroll_up: "Scroll up",
    scroll_down: "Scroll down",
};

static LOCALE_ZH: Locale = Locale {
    app_title: "RustPad",
    menu_file: "文件",
    menu_edit: "编辑",
    menu_view: "视图",
    menu_encoding: "编码",
    menu_settings: "设置",
    menu_tools: "工具",
    menu_help: "帮助",
    file_new: "新建  Ctrl+N",
    file_open: "打开...  Ctrl+O",
    file_save: "保存  Ctrl+S",
    file_save_as: "另存为...  Ctrl+Shift+S",
    file_save_all: "全部保存",
    file_close_tab: "关闭标签  Ctrl+W",
    file_compare: "对比文件...  Ctrl+D",
    file_compare_current: "将当前文件与...对比",
    file_exit: "退出  Alt+F4",
    edit_undo: "撤销  Ctrl+Z",
    edit_redo: "重做  Ctrl+Y",
    edit_cut: "剪切  Ctrl+X",
    edit_copy: "复制  Ctrl+C",
    edit_paste: "粘贴  Ctrl+V",
    edit_select_all: "全选  Ctrl+A",
    edit_find: "查找...  Ctrl+F",
    edit_replace: "替换...  Ctrl+H",
    edit_goto_line: "跳转到行...  Ctrl+G",
    edit_copy_column: "列复制  Alt+Shift+C",
    view_toggle_sidebar: "切换侧边栏  Ctrl+B",
    view_toggle_minimap: "切换缩略图",
    view_language: "语言",
    view_auto_detect: "自动检测",
    view_font_size: "字号",
    view_word_wrap: "自动换行",
    view_line_numbers: "行号",
    tools_compare: "对比文件...  Ctrl+D",
    tools_macro: "宏录制",
    tools_preferences: "首选项...",
    settings_preferences: "首选项...",
    settings_keybindings: "快捷键管理...",
    help_about: "关于 RustPad",
    tb_new: "📄",
    tb_open: "📂",
    tb_save: "💾",
    tb_undo: "↩",
    tb_redo: "↪",
    tb_find: "🔍",
    tb_compare: "🆚",
    tb_encoding: "🌐",
    tip_new: "新建文件 (Ctrl+N)",
    tip_open: "打开文件 (Ctrl+O)",
    tip_save: "保存 (Ctrl+S)",
    tip_undo: "撤销 (Ctrl+Z)",
    tip_redo: "重做 (Ctrl+Y)",
    tip_find: "查找 (Ctrl+F)",
    tip_compare: "对比文件 (Ctrl+D)",
    tip_encoding: "使用指定编码打开 / 转换编码",
    tip_font_dec: "减小字号",
    tip_font_inc: "增大字号",
    tip_font_size: "字号 (8–72 px，可直接输入)",
    tip_tab_scroll_left: "向左滚动标签",
    tip_tab_scroll_right: "向右滚动标签",
    dlg_preferences: "首选项",
    dlg_about: "关于 RustPad",
    dlg_goto_line: "跳转到行",
    dlg_unsaved: "未保存的更改",
    dlg_quit_save: "保存更改？",
    pref_editor: "编辑器",
    pref_ui: "界面",
    pref_font_size: "字号：",
    pref_tab_size: "Tab 宽度：",
    pref_show_line_numbers: "显示行号",
    pref_word_wrap: "自动换行",
    pref_auto_indent: "自动缩进",
    pref_highlight_line: "高亮当前行",
    pref_show_minimap: "显示缩略图",
    pref_theme: "主题：",
    pref_ui_language: "界面语言：",
    pref_save: "保存首选项",
    theme_light: "浅色",
    theme_dark: "深色",
    goto_line_number: "行号：",
    about_tagline: "用 Rust 打造的现代化代码编辑器",
    about_powered: "基于 egui + eframe",
    about_syntax: "语法高亮由 syntect 提供",
    about_project: "RustPad 介绍：",
    about_releases: "RustPad 最新版本：",
    btn_save: "保存",
    btn_dont_save: "不保存",
    btn_cancel: "取消",
    btn_save_all: "全部保存",
    btn_close: "关闭",
    btn_clear: "清除",
    status_lines: "行",
    status_lang: "语言：",
    find_replace_title: "查找 / 替换",
    find_tab: "查找",
    replace_tab: "替换",
    dir_find: "目录查找",
    find_what: "查找内容：",
    replace_with: "替换为：",
    search_mode: "搜索模式",
    mode_normal: "普通",
    mode_regex: "正则表达式",
    opt_backward: "反向查找",
    opt_whole_word: "全字匹配",
    opt_match_case: "区分大小写",
    opt_wrap: "循环查找",
    find_next: "查找下一个  (F3)",
    find_prev: "查找上一个  (F4)",
    find_count: "计数",
    find_all_current: "在当前文档中查找全部",
    find_all_open: "在所有打开文档中查找全部",
    clear_result: "清除结果",
    replace_one: "替换",
    replace_all: "全部替换",
    search_results: "查找结果",
    no_search_results: "无结果。使用「查找全部」填充此列表。",
    no_recent_searches: "（无最近搜索记录）",
    find_in_files: "在文件中查找",
    diff_open_left: "打开左侧…",
    diff_open_right: "打开右侧…",
    diff_swap: "⇄ 交换",
    diff_prev: "⬆ 上一处",
    diff_next: "⬇ 下一处",
    diff_ignore_ws: "忽略空白",
    diff_ignore_case: "忽略大小写",
    diff_save_left: "💾 保存左侧",
    diff_save_right: "💾 保存右侧",
    diff_export: "导出报告…",
    diff_close: "✖ 关闭对比",
    tip_diff_prev: "上一处差异 (F7)",
    tip_diff_next: "下一处差异 (F8)",
    enc_open_section: "使用编码打开",
    enc_convert_section: "转换为编码",
    enc_more: "更多…",
    enc_convert_more: "转换为更多编码",
    enc_batch_convert: "批量转换编码",
    dlg_batch_encoding: "批量转换编码",
    enc_batch_prompt: "将所有打开的标签转换为：",
    dlg_keybindings: "快捷键管理",
    kb_command_col: "命令",
    kb_shortcut_col: "快捷键",
    kb_change: "更改",
    kb_reset: "恢复默认",
    kb_scheme: "方案",
    kb_press_key: "请按下快捷键…（Esc 取消）",
    kb_conflicts: "检测到快捷键冲突",
    kb_saved: "快捷键已保存",
    tip_column_select: "按住 Alt 并拖动以列选择",
    scroll_to_here: "滚动到这里",
    scroll_top: "顶部",
    scroll_bottom: "底部",
    scroll_page_up: "上一页",
    scroll_page_down: "下一页",
    scroll_up: "向上滚动",
    scroll_down: "向下滚动",
};

pub fn locale(lang: &str) -> &'static Locale {
    match UiLanguage::parse(lang) {
        UiLanguage::En => &LOCALE_EN,
        UiLanguage::Zh => &LOCALE_ZH,
    }
}

impl Locale {
    fn is_zh(&self) -> bool {
        std::ptr::eq(self, &LOCALE_ZH)
    }

    pub fn enc_open_with(&self, profile: crate::editor::EncodingProfile) -> String {
        let name = profile.display_name();
        if self.is_zh() {
            format!("使用编码 {name} 打开")
        } else {
            format!("Open with {name}")
        }
    }

    pub fn enc_convert_to(&self, profile: crate::editor::EncodingProfile) -> String {
        let name = profile.display_name();
        if self.is_zh() {
            format!("转换为 {name} 编码")
        } else {
            format!("Convert to {name}")
        }
    }
}

/// Format: `"file" has unsaved changes...`
pub fn unsaved_message(lang: &str, file: &str) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh => format!("「{file}」有未保存的更改。您要如何处理？"),
        UiLanguage::En => {
            format!("\"{file}\" has unsaved changes. What would you like to do?")
        }
    }
}

/// Format: quit with N unsaved files.
pub fn quit_unsaved_message(lang: &str, count: usize) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh => format!(
            "您有 {count} 个文件包含未保存的更改。关闭 RustPad 前是否保存？"
        ),
        UiLanguage::En => format!(
            "You have {count} file(s) with unsaved changes. Save before closing RustPad?"
        ),
    }
}

pub fn line_label(lang: &str, line: usize) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh => format!("第 {} 行", line),
        UiLanguage::En => format!("Line {}", line),
    }
}

pub fn matches_count(lang: &str, n: usize) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh => format!("（{n} 个匹配）"),
        UiLanguage::En => format!("({n} matches)"),
    }
}

pub fn search_for(lang: &str, pattern: &str) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh => format!("关键词「{pattern}」"),
        UiLanguage::En => format!("for \"{pattern}\""),
    }
}

pub fn msg_no_matches(lang: &str) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh => "未找到匹配项。".to_string(),
        UiLanguage::En => "No matches found.".to_string(),
    }
}

pub fn msg_found_matches(lang: &str, n: usize) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh => format!("找到 {n} 个匹配项。"),
        UiLanguage::En => format!("Found {n} match(es)."),
    }
}

pub fn msg_match_of(lang: &str, idx: usize, total: usize) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh => format!("第 {idx} 个，共 {total} 个匹配。"),
        UiLanguage::En => format!("Match {idx} of {total}."),
    }
}

pub fn msg_enter_search_text(lang: &str) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh => "请输入要查找的文本。".to_string(),
        UiLanguage::En => "Enter text to find.".to_string(),
    }
}

pub fn msg_count_matches(lang: &str, count: usize) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh if count == 0 => "计数：0 个匹配。".to_string(),
        UiLanguage::Zh => format!("计数：当前文档中 {count} 个匹配。"),
        UiLanguage::En if count == 0 => "Count: 0 matches.".to_string(),
        UiLanguage::En => format!("Count: {count} match(es) in current document."),
    }
}

pub fn msg_find_all_current(lang: &str, count: usize) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh if count == 0 => "当前文档中无匹配。".to_string(),
        UiLanguage::Zh => format!("在当前文档中找到 {count} 个匹配。"),
        UiLanguage::En if count == 0 => "No matches in current document.".to_string(),
        UiLanguage::En => format!("Found {count} match(es) in current document."),
    }
}

pub fn msg_find_all_open(lang: &str, total: usize, parts: &str) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh if total == 0 => "所有打开文档中无匹配。".to_string(),
        UiLanguage::Zh => format!("共找到 {total} 个匹配 — {parts}"),
        UiLanguage::En if total == 0 => "No matches in open documents.".to_string(),
        UiLanguage::En => format!("Found {total} match(es) — {parts}"),
    }
}

pub fn msg_replaced(lang: &str, n: usize) -> String {
    match UiLanguage::parse(lang) {
        UiLanguage::Zh => format!("已替换 {n} 处。"),
        UiLanguage::En => format!("Replaced {n} occurrence(s)."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_language() {
        assert_eq!(UiLanguage::parse("zh"), UiLanguage::Zh);
        assert_eq!(UiLanguage::parse("en"), UiLanguage::En);
        assert_eq!(UiLanguage::parse("unknown"), UiLanguage::En);
    }
}
