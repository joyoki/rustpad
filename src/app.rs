use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::AppConfig;
use crate::config::theme::ThemeManager;
use crate::diff::is_likely_binary;
use crate::editor::{Cursor, EncodingProfile, Selection, TabManager};
use crate::highlight::Highlighter;
use crate::search::{SearchEngine, SearchOptions};

/// Which tab is active in the find/replace dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchDialogTab {
    #[default]
    Find,
    Replace,
}

/// One entry in the "Search results" panel.
#[derive(Debug, Clone)]
pub struct SearchResultItem {
    /// Index of the tab the match lives in.
    pub tab: usize,
    /// Display title of the document (file name or "Untitled").
    pub doc: String,
    /// 0-based line number of the match.
    pub line: usize,
    /// 0-based column of the match.
    pub col: usize,
    /// Character offset of the match start.
    pub start: usize,
    /// Character offset of the match end (exclusive).
    pub end: usize,
    /// The full text of the line the match is on (for preview).
    pub preview: String,
}

/// One committed "Find All" session kept in the results panel for later review.
#[derive(Debug, Clone)]
pub struct SearchResultBatch {
    pub id: usize,
    pub pattern: String,
    pub scope_label: String,
    pub items: Vec<SearchResultItem>,
    pub collapsed: bool,
    pub collapsed_docs: HashSet<String>,
}
use crate::session::{AutoSaveManager, Session};
use crate::ui::compare_session::CompareWindowManager;
use crate::ui::file_tree::FileTree;
use crate::ui::keybindings::{Command, KeyBindings};
use crate::ui::minimap::Minimap;
use crate::ui::sidebar::SidebarTab;

/// Command palette state for quick actions.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct CommandPalette {
    pub visible: bool,
    pub query: String,
    pub selected_index: usize,
    pub commands: Vec<CommandEntry>,
}

/// A single command in the command palette.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommandEntry {
    pub name: String,
    pub description: String,
    pub shortcut: String,
}

/// Status bar data shown at the bottom of the window.
#[derive(Debug, Default)]
pub struct StatusBar {
    pub line: usize,
    pub column: usize,
    pub encoding: String,
    pub line_ending: String,
    pub language: String,
    pub cursor_position: String,
}

/// Batch tab-close operation resumed after each unsaved prompt.
#[derive(Debug, Clone, Copy)]
pub enum TabCloseBatch {
    Others { keep: usize },
    LeftOf { from: usize },
    RightOf { from: usize },
}

/// Top-level application state.
#[allow(dead_code)]
pub struct RustpadApp {
    // Core data
    pub tab_manager: TabManager,
    pub active_tab: usize,
    pub config: AppConfig,
    pub session: Session,
    pub highlighter: Highlighter,
    pub search_engine: SearchEngine,

    // UI state
    pub show_sidebar: bool,
    pub show_search: bool,
    pub show_replace: bool,
    pub show_about: bool,
    pub show_preferences: bool,
    pub show_keybindings: bool,
    pub show_goto_line: bool,
    pub show_batch_encoding: bool,
    pub show_unsaved_dialog: bool,
    pub show_quit_unsaved_dialog: bool,
    pub show_save_error_dialog: bool,
    pub save_error_message: String,
    pub show_cross_file_search: bool,
    pub search_replace_mode: bool,
    pub sidebar_tab: SidebarTab,
    pub goto_line_text: String,

    // Command palette
    pub command_palette: CommandPalette,

    // Status bar
    pub status_bar: StatusBar,

    // Search state
    pub search_pattern: String,
    pub replace_pattern: String,
    pub search_options: SearchOptions,
    pub search_dialog_tab: SearchDialogTab,
    pub search_history: Vec<String>,
    pub search_status_message: String,
    pub search_focus_input: bool,
    pub search_focus_replace: bool,
    pub search_panel_text: String,
    pub search_results: Vec<(PathBuf, usize, String)>,
    /// Committed find-all sessions (newest first); accumulates across searches.
    pub search_result_batches: Vec<SearchResultBatch>,
    pub next_search_batch_id: usize,
    /// Batch whose match is currently highlighted in the editor.
    pub active_result_batch_id: Option<usize>,
    /// Whether the dockable "Search results" panel is visible.
    pub show_search_results: bool,

    // Cross-file search
    pub cross_file_directory: String,
    pub cross_file_filter: String,
    pub cross_file_results: Vec<(PathBuf, usize, String)>,

    // Detached compare windows (notepad-- CompareWin / CompareDirs / CompareHexWin)
    pub compare_mgr: CompareWindowManager,
    /// First side picked via tab context menu before opening a compare window.
    compare_pick_left: Option<PathBuf>,
    compare_pick_right: Option<PathBuf>,

    // Workspace
    pub workspace_root: Option<PathBuf>,

    // File tree
    pub file_tree: FileTree,

    // Minimap
    pub minimap: Minimap,

    // Macro
    pub macro_recording: bool,
    pub macro_actions: Vec<String>,

    // Auto-save
    pub auto_save: AutoSaveManager,

    // Keybindings
    pub keybindings: KeyBindings,
    pub keybindings_edit: KeyBindings,
    pub keybindings_recording: Option<Command>,
    pub keybindings_status: String,

    // Theme manager
    pub theme_manager: ThemeManager,

    // Unsaved dialog
    pub pending_close_tab: Option<usize>,
    pub pending_close_batch: Option<TabCloseBatch>,

    // Tab rename dialog
    pub show_rename_tab: bool,
    pub rename_tab_index: Option<usize>,
    pub rename_tab_text: String,

    // Editor focus state
    pub editor_has_focus: bool,

    // Keyboard shortcut state
    pub pending_new_tab: bool,
    pub pending_open_file: bool,
    pub pending_save: bool,
    pub pending_close: bool,
    pub pending_find: bool,
    pub pending_replace: bool,
    pub pending_find_next: bool,
    pub pending_find_prev: bool,
    pub pending_goto: bool,
    pub pending_diff: bool,
    pub pending_compare_files: bool,
    pub pending_compare_current: bool,
    pub pending_compare_dirs: bool,
    pub pending_compare_binary: bool,
    pub pending_undo: bool,
    pub pending_redo: bool,
    pub pending_select_all: bool,
    pub pending_next_tab: bool,
    pub pending_prev_tab: bool,
    pub pending_command_palette: bool,
    pub pending_find_in_files: bool,
    pub pending_cut: bool,
    pub pending_copy: bool,
    pub pending_paste: bool,
    pub pending_save_as: bool,
    pub pending_save_all: bool,
    pub pending_toggle_sidebar: bool,
    pub pending_exit: bool,
    /// Set once the user confirms quitting; bypasses the unsaved-changes guard
    /// so the window can actually close (e.g. after "Don't Save").
    pub force_quit: bool,
    last_session_persist: std::time::Instant,
    last_applied_theme: String,
    last_applied_language: String,

    /// HTML preview window (Markdown / text export).
    pub show_html_preview: bool,
    pub html_preview_content: String,
    pub html_preview_title: String,
    /// Short status line from context-menu actions (word count, etc.).
    pub transient_message: String,

    // Context menu state (managed manually so close works reliably)
    pub show_context_menu: bool,
    pub context_menu_pos: egui::Pos2,
    /// Selection snapshot taken when the context menu opens (preserved for cut/copy/mark).
    pub context_menu_selection: Option<crate::editor::Selection>,
    /// Background marks carried with clipboard cut/copy for paste.
    clipboard_marks: Vec<crate::editor::context_actions::RelativeTextMark>,

    /// Toolbar font-size text field buffer (kept on app so typing is not reset each frame).
    pub toolbar_font_size_text: String,
    /// True while the toolbar font-size field is focused (blocks external text sync).
    pub toolbar_font_size_editing: bool,

    /// In-app logo texture (About dialog, startup splash).
    pub logo_texture: egui::TextureHandle,
    /// Wall-clock time until the startup splash is hidden (`None` after dismiss).
    splash_until: Option<f64>,

    /// Native macOS menu bar (system menu after the app name).
    #[cfg(target_os = "macos")]
    pub macos_menu: Option<muda::Menu>,
    #[cfg(target_os = "macos")]
    pub macos_menu_rx: Option<std::sync::mpsc::Receiver<muda::MenuEvent>>,
    #[cfg(target_os = "macos")]
    pub macos_encoding_open_checks: Vec<(EncodingProfile, muda::CheckMenuItem)>,
    #[cfg(target_os = "macos")]
    pub macos_compare_history: Option<crate::platform::macos_menu::MacosCompareHistory>,
}

impl RustpadApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        log::info!("Initializing RustPad application...");

        setup_cjk_fonts(cc);
        let logo_texture = crate::branding::load_logo_texture(&cc.egui_ctx);
        let splash_until = cc.egui_ctx.input(|i| i.time + 1.4);

        let config = AppConfig::load();
        let toolbar_font_size_text = format!("{}", config.editor.font_size as u32);
        let session = Session::load();
        let mut tab_manager = if session.open_files.is_empty() {
            TabManager::new()
        } else {
            TabManager::from_session(&session.open_files)
        };

        // Restore active tab, cursor, and scroll positions.
        if !session.open_files.is_empty() {
            tab_manager.set_active(session.active_tab.min(tab_manager.tab_count().saturating_sub(1)));
            for tab in tab_manager.tabs_mut() {
                if let Some(path) = &tab.file_path.clone() {
                    if let Some((line, col)) = session.get_cursor(path) {
                        tab.cursor = Cursor::new(line, col);
                    }
                    if let Some(scroll) = session.get_scroll(path) {
                        tab.scroll_offset = scroll;
                    }
                    tab.syntax_override = None;
                }
            }
        }

        let workspace_root = session.workspace_root.clone();

        log::info!(
            "Application initialized with {} tabs",
            tab_manager.tab_count()
        );

        let mut app = Self {
            tab_manager,
            active_tab: 0,
            config,
            session,
            highlighter: Highlighter::new(),
            search_engine: SearchEngine::new(),
            show_sidebar: true,
            show_search: false,
            show_replace: false,
            show_about: false,
            show_preferences: false,
            show_keybindings: false,
            show_goto_line: false,
            show_batch_encoding: false,
            show_unsaved_dialog: false,
            show_quit_unsaved_dialog: false,
            show_save_error_dialog: false,
            save_error_message: String::new(),
            show_cross_file_search: false,
            search_replace_mode: false,
            sidebar_tab: SidebarTab::FileExplorer,
            goto_line_text: String::new(),
            command_palette: CommandPalette::default(),
            status_bar: StatusBar::default(),
            search_pattern: String::new(),
            replace_pattern: String::new(),
            search_options: SearchOptions::default(),
            search_dialog_tab: SearchDialogTab::Find,
            search_history: Vec::new(),
            search_status_message: String::new(),
            search_focus_input: false,
            search_focus_replace: false,
            search_panel_text: String::new(),
            search_results: Vec::new(),
            search_result_batches: Vec::new(),
            next_search_batch_id: 1,
            active_result_batch_id: None,
            show_search_results: false,
            cross_file_directory: String::new(),
            cross_file_filter: "*.rs".to_string(),
            cross_file_results: Vec::new(),
            compare_mgr: CompareWindowManager::default(),
            compare_pick_left: None,
            compare_pick_right: None,
            workspace_root,
            file_tree: FileTree::new(),
            minimap: Minimap::new(),
            macro_recording: false,
            macro_actions: Vec::new(),
            auto_save: AutoSaveManager::default(),
            keybindings: KeyBindings::load(),
            keybindings_edit: KeyBindings::load(),
            keybindings_recording: None,
            keybindings_status: String::new(),
            theme_manager: ThemeManager::new(),
            pending_close_tab: None,
            pending_close_batch: None,
            show_rename_tab: false,
            rename_tab_index: None,
            rename_tab_text: String::new(),
            pending_new_tab: false,
            pending_open_file: false,
            pending_save: false,
            pending_close: false,
            pending_find: false,
            pending_replace: false,
            pending_find_next: false,
            pending_find_prev: false,
            pending_goto: false,
            pending_diff: false,
            pending_compare_files: false,
            pending_compare_current: false,
            pending_compare_dirs: false,
            pending_compare_binary: false,
            pending_undo: false,
            pending_redo: false,
            pending_select_all: false,
            pending_next_tab: false,
            pending_prev_tab: false,
            pending_command_palette: false,
            pending_find_in_files: false,
            pending_cut: false,
            pending_copy: false,
            pending_paste: false,
            pending_save_as: false,
            pending_save_all: false,
            pending_toggle_sidebar: false,
            pending_exit: false,
            force_quit: false,
            editor_has_focus: false,
            last_session_persist: std::time::Instant::now(),
            last_applied_theme: String::new(),
            last_applied_language: String::new(),
            show_html_preview: false,
            html_preview_content: String::new(),
            html_preview_title: String::new(),
            transient_message: String::new(),
            show_context_menu: false,
            context_menu_pos: egui::Pos2::ZERO,
            context_menu_selection: None,
            clipboard_marks: Vec::new(),
            toolbar_font_size_text,
            toolbar_font_size_editing: false,
            logo_texture,
            splash_until: Some(splash_until),
            #[cfg(target_os = "macos")]
            macos_menu: None,
            #[cfg(target_os = "macos")]
            macos_menu_rx: None,
            #[cfg(target_os = "macos")]
            macos_encoding_open_checks: Vec::new(),
            #[cfg(target_os = "macos")]
            macos_compare_history: None,
        };

        app.apply_theme(&cc.egui_ctx);
        app.last_applied_theme = app.config.ui.theme.clone();
        app.last_applied_language = app.config.ui.ui_language.clone();
        app
    }

    /// Apply UI + editor theme from config.
    pub fn apply_theme(&mut self, ctx: &egui::Context) {
        let normalized = self.config.ui.theme.to_lowercase();
        let editor_theme_name = if normalized == "dark" {
            "Dark"
        } else {
            "Light"
        };

        self.theme_manager.set_theme(editor_theme_name);

        if editor_theme_name == "Dark" {
            ctx.set_visuals(egui::Visuals::dark());
            self.highlighter.set_theme("base16-ocean.dark");
        } else {
            ctx.set_visuals(egui::Visuals::light());
            self.highlighter.set_theme("InspiredGitHub");
        }
        self.highlighter.invalidate_all();
    }

    /// Localized UI strings for the current language setting.
    pub fn tr(&self) -> &'static crate::i18n::Locale {
        crate::i18n::locale(&self.config.ui.ui_language)
    }

    fn ui_lang(&self) -> &str {
        &self.config.ui.ui_language
    }

    /// Re-apply theme when preferences change.
    pub fn sync_theme(&mut self, ctx: &egui::Context) {
        if self.config.ui.theme != self.last_applied_theme {
            self.apply_theme(ctx);
            self.last_applied_theme = self.config.ui.theme.clone();
        }
        self.sync_language(ctx);
    }

    /// Update window title when UI language changes.
    pub fn sync_language(&mut self, ctx: &egui::Context) {
        if self.config.ui.ui_language != self.last_applied_language {
            let title = self.tr().app_title.to_string();
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
            self.last_applied_language = self.config.ui.ui_language.clone();
            ctx.request_repaint();
        }
    }

    /// Called after the user picks a new UI language in preferences.
    pub fn on_language_changed(&mut self, ctx: &egui::Context) {
        self.sync_language(ctx);
    }

    /// Called after the user picks a new theme in preferences.
    pub fn on_theme_changed(&mut self, ctx: &egui::Context) {
        self.apply_theme(ctx);
        self.last_applied_theme = self.config.ui.theme.clone();
    }

    /// Set syntax highlighting language for the active tab.
    pub fn set_active_language(&mut self, syntax_name: &str) {
        self.tab_manager.active_mut().syntax_override = Some(syntax_name.to_string());
        self.highlighter.clear_cache();
        self.highlighter.invalidate_all();
    }

    /// Reset syntax highlighting to auto-detect from file extension.
    pub fn clear_active_language(&mut self) {
        self.tab_manager.active_mut().syntax_override = None;
        self.highlighter.clear_cache();
        self.highlighter.invalidate_all();
    }

    /// Handle window close requests from the OS (red button / Cmd+Q).
    pub fn handle_close_request(&mut self, ctx: &egui::Context) {
        // Once the user has confirmed the quit, let the close proceed without
        // re-triggering the unsaved-changes guard (buffers may still be dirty
        // after "Don't Save").
        if self.force_quit {
            return;
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.tab_manager.has_unsaved_changes() {
            self.show_quit_unsaved_dialog = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.request_repaint();
        } else if ctx.input(|i| i.viewport().close_requested()) {
            self.persist_session();
        }
    }

    /// Apply theme visuals to a secondary viewport (compare windows).
    pub fn apply_theme_to_context(&self, ctx: &egui::Context) {
        let normalized = self.config.ui.theme.to_lowercase();
        if normalized == "dark" {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }

    /// Open a detached file compare window (pick paths inside the window).
    pub fn open_compare_window(&mut self) {
        self.compare_mgr.new_file_window();
    }

    pub fn open_folder_compare_window(&mut self) {
        self.compare_mgr.new_folder_window();
    }

    pub fn open_compare_from_history(&mut self, pair: crate::config::ComparePair) {
        self.compare_mgr.open_with_paths(pair.left, pair.right);
    }

    pub fn record_compare_history(
        &mut self,
        left: std::path::PathBuf,
        right: std::path::PathBuf,
        mode: crate::ui::compare_session::CompareMode,
    ) {
        use crate::ui::compare_session::CompareMode;
        match mode {
            CompareMode::Text | CompareMode::Binary => {
                self.config.add_recent_file_compare(left, right);
            }
            CompareMode::Folder => {
                self.config.add_recent_folder_compare(left, right);
            }
            CompareMode::None => return,
        }
        let _ = self.config.save();
        #[cfg(target_os = "macos")]
        crate::platform::macos_menu::sync_compare_history(self);
    }

    fn open_file_compare(&mut self, left: PathBuf, right: PathBuf) {
        self.compare_mgr.open_with_paths(left, right);
    }

    fn open_binary_compare(&mut self, left: PathBuf, right: PathBuf) {
        self.compare_mgr.open_with_paths(left, right);
    }

    /// Open file compare window (notepad-- style: window first, then pick files inside).
    pub fn compare_files_dialog(&mut self) {
        self.open_compare_window();
    }

    /// Open compare window with the active tab preloaded on the left side.
    pub fn compare_current_with_dialog(&mut self) {
        let current = self.tab_manager.active().file_path.clone();
        let Some(current) = current else {
            log::warn!("Compare current: active tab has no saved file path");
            return;
        };

        let id = self.compare_mgr.new_file_window();
        if let Some(session) = self.compare_mgr.session_mut(id) {
            session.left_path_text = current.display().to_string();
        }
    }

    /// Open folder compare window (pick folders inside the window).
    pub fn compare_dirs_dialog(&mut self) {
        self.open_folder_compare_window();
    }

    pub fn compare_binary_files_dialog(&mut self) {
        self.open_compare_window();
    }

    /// If the given file is open in a tab, refresh its buffer from disk text.
    pub fn sync_open_tab_with_disk(&mut self, path: &std::path::Path, text: &str) {
        for tab in self.tab_manager.tabs_mut() {
            if tab.file_path.as_deref() == Some(path) {
                tab.buffer = crate::editor::buffer::TextBuffer::from_str(text);
                tab.modified = false;
            }
        }
        self.highlighter.invalidate_all();
    }

    /// Update status bar with current tab information.
    pub fn update_status_bar(&mut self) {
        let tab = self.tab_manager.active();
        self.status_bar.line = tab.cursor.line + 1;
        self.status_bar.column = tab.cursor.col + 1;
        self.status_bar.encoding = tab.encoding.status_label().to_string();
        self.status_bar.line_ending = format!("{:?}", tab.buffer.line_ending());
        self.status_bar.language = {
            let filename = tab
                .file_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| tab.title.clone());
            self.highlighter
                .syntax_name_for_file(&filename, tab.syntax_override.as_deref())
        };
        self.status_bar.cursor_position = format!(
            "Ln {}, Col {}",
            self.status_bar.line, self.status_bar.column
        );
    }

    /// Open a file dialog and open the selected file.
    pub fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("All Files", &["*"])
            .add_filter(
                "Text Files",
                &["txt", "md", "rs", "py", "js", "ts", "html", "css", "json", "toml", "yaml"],
            )
            .pick_file()
        {
            self.open_file(path);
        }
    }

    /// Open a file in a new tab.
    pub fn open_file(&mut self, path: PathBuf) {
        log::info!("Opening file: {:?}", path);
        match self.tab_manager.open_file(&path) {
            Ok(_) => {
                self.tab_manager.active_mut().syntax_override = None;
                self.highlighter.clear_cache();
                self.highlighter.invalidate_all();
                self.config.add_recent_file(path.clone());
                self.session.add_recent_file(path);
                self.persist_session();
            }
            Err(e) => {
                log::error!("Failed to open file: {}", e);
            }
        }
    }

    /// Open files dragged onto the application window.
    pub fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let paths: Vec<PathBuf> = ctx.input(|i| {
            i.raw.dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect()
        });
        if paths.is_empty() {
            return;
        }

        // When compare windows are open, route drops to the active compare session
        // instead of opening files in the editor (drops may land on the parent ctx).
        if let Some(id) = self.compare_mgr.sessions.last().map(|s| s.id) {
            let pointer = ctx.pointer_latest_pos();
            for (index, path) in paths.into_iter().enumerate() {
                let to_right = if let Some(session) = self.compare_mgr.session(id) {
                    if let Some(p) = pointer {
                        session.drop_side_at_pointer(p)
                    } else {
                        None
                    }
                } else {
                    None
                }
                .unwrap_or(index % 2 == 1);
                if let Some(session) = self.compare_mgr.session_mut(id) {
                    session.apply_dropped_path(path, to_right);
                }
            }
            ctx.input_mut(|i| i.raw.dropped_files.clear());
            return;
        }

        for path in paths {
            if path.is_dir() {
                self.open_workspace_folder(path);
            } else if path.is_file() {
                self.open_file(path);
            } else {
                log::warn!("Ignored dropped path: {:?}", path);
            }
        }
        ctx.input_mut(|i| i.raw.dropped_files.clear());
    }

    /// Open a folder as the workspace (file explorer sidebar).
    pub fn open_workspace_folder(&mut self, path: PathBuf) {
        let path = path.canonicalize().unwrap_or(path);
        if !path.is_dir() {
            log::warn!("Ignored workspace path (not a directory): {:?}", path);
            return;
        }
        log::info!("Opening workspace folder: {:?}", path);
        self.workspace_root = Some(path.clone());
        self.file_tree.load(&path);
        self.show_sidebar = true;
        self.sidebar_tab = SidebarTab::FileExplorer;
        self.persist_session();
    }

    /// Save the current tab.
    pub fn save_current_tab(&mut self) {
        let has_path = self.tab_manager.active().file_path.is_some();
        if has_path {
            let idx = self.tab_manager.active_index();
            let _ = self.save_tab_at_index(idx);
        } else {
            self.save_as_dialog();
        }
    }

    /// Save a tab by index. Returns false when the write fails.
    pub fn save_tab_at_index(&mut self, idx: usize) -> bool {
        let Some(tab) = self.tab_manager.tabs().get(idx) else {
            return false;
        };
        let file_label = tab
            .file_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| tab.display_title());
        match self.tab_manager.tabs_mut()[idx].save() {
            Ok(()) => true,
            Err(e) => {
                self.report_save_failure(&file_label, &e);
                false
            }
        }
    }

    /// Save the active tab to a new path. Returns false when the write fails.
    pub fn save_active_tab_as(&mut self, path: &PathBuf) -> bool {
        let file_label = path.display().to_string();
        match self.tab_manager.active_mut().save_as(path) {
            Ok(()) => {
                self.tab_manager.active_mut().syntax_override = None;
                self.highlighter.clear_cache();
                self.highlighter.invalidate_all();
                true
            }
            Err(e) => {
                self.report_save_failure(&file_label, &e);
                false
            }
        }
    }

    fn report_file_write_failure(
        &mut self,
        operation: &str,
        file_label: &str,
        err: &impl std::fmt::Display,
    ) {
        let lang = self.config.ui.ui_language.clone();
        let message =
            crate::i18n::file_write_failed_message(&lang, operation, file_label, &err.to_string());
        log::error!("{message}");
        self.save_error_message = message.clone();
        self.transient_message = message;
        self.show_save_error_dialog = true;
    }

    fn report_save_failure(&mut self, file_label: &str, err: &impl std::fmt::Display) {
        let op = self.tr().err_op_save;
        self.report_file_write_failure(op, file_label, err);
    }

    /// Write a file and surface failures in the status bar and error dialog.
    pub fn write_file_with_feedback(
        &mut self,
        path: impl AsRef<std::path::Path>,
        contents: impl AsRef<[u8]>,
        operation: &'static str,
    ) -> bool {
        let path = path.as_ref();
        let file_label = path.display().to_string();
        match crate::atomic_write::atomic_write(path, contents.as_ref()) {
            Ok(()) => true,
            Err(e) => {
                self.report_file_write_failure(operation, &file_label, &e);
                false
            }
        }
    }

    /// Save As dialog for the current tab.
    pub fn save_as_dialog(&mut self) {
        let default_name = self
            .tab_manager
            .active()
            .file_path
            .as_ref()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "untitled.txt".to_string());

        if let Some(mut path) = rfd::FileDialog::new()
            .add_filter("Text File (*.txt)", &["txt"])
            .add_filter("Rust (*.rs)", &["rs"])
            .add_filter("C (*.c)", &["c"])
            .add_filter("C/C++ Header (*.h)", &["h", "hpp"])
            .add_filter("C++ (*.cpp)", &["cpp", "cc", "cxx"])
            .add_filter("Java (*.java)", &["java"])
            .add_filter("Python (*.py)", &["py"])
            .add_filter("JavaScript (*.js)", &["js"])
            .add_filter("TypeScript (*.ts)", &["ts"])
            .add_filter("Go (*.go)", &["go"])
            .add_filter("HTML (*.html)", &["html", "htm"])
            .add_filter("CSS (*.css)", &["css"])
            .add_filter("JSON (*.json)", &["json"])
            .add_filter("TOML (*.toml)", &["toml"])
            .add_filter("YAML (*.yaml)", &["yaml", "yml"])
            .add_filter("Markdown (*.md)", &["md"])
            .add_filter("XML (*.xml)", &["xml"])
            .add_filter("Shell (*.sh)", &["sh"])
            .add_filter("All Files (*.*)", &["*"])
            .set_file_name(&default_name)
            .save_file()
        {
            if path.extension().is_none() {
                path.set_extension("txt");
            }
            let _ = self.save_active_tab_as(&path);
        }
    }

    /// Save all tabs that already have a file path.
    pub fn save_all_tabs(&mut self) {
        let count = self.tab_manager.tab_count();
        for idx in 0..count {
            if self.tab_manager.tabs()[idx].file_path.is_some()
                && !self.save_tab_at_index(idx)
            {
                break;
            }
        }
    }

    /// Re-read the active file from disk using the given encoding.
    pub fn open_with_encoding(&mut self, profile: EncodingProfile) {
        if self.tab_manager.active().file_path.is_none() {
            return;
        }
        let idx = self.tab_manager.active_index();
        match self.tab_manager.tabs_mut()[idx].reload_with_encoding(profile) {
            Ok(()) => {
                self.highlighter.clear_cache();
                self.highlighter.invalidate_all();
                self.transient_message = format!(
                    "{}: {}",
                    self.tr().enc_open_section,
                    profile.display_name()
                );
            }
            Err(e) => log::error!("Failed to reopen with encoding: {e}"),
        }
    }

    /// Set the save encoding for the active tab (Unicode buffer unchanged).
    pub fn convert_to_encoding(&mut self, profile: EncodingProfile) {
        self.tab_manager.active_mut().convert_to_encoding(profile);
        self.transient_message = format!(
            "{}: {}",
            self.tr().enc_convert_section,
            profile.display_name()
        );
    }

    /// Convert all open tabs to the given save encoding.
    pub fn batch_convert_encoding(&mut self, profile: EncodingProfile) {
        for tab in self.tab_manager.tabs_mut() {
            tab.convert_to_encoding(profile);
        }
        self.transient_message = format!(
            "{}: {}",
            self.tr().enc_batch_convert,
            profile.display_name()
        );
    }

    /// Save every dirty tab before quitting. Returns false if the user cancels.
    pub fn save_all_tabs_for_quit(&mut self) -> bool {
        let dirty_indices: Vec<usize> = self.tab_manager.unsaved_tab_indices();
        for idx in dirty_indices {
            self.tab_manager.set_active(idx);
            let tab = self.tab_manager.active();
            if tab.file_path.is_none() {
                self.save_as_dialog();
                let still_unsaved = self.tab_manager.active().file_path.is_none()
                    && (self.tab_manager.active().buffer.is_dirty()
                        || self.tab_manager.active().modified);
                if still_unsaved {
                    return false;
                }
            } else if !self.save_tab_at_index(idx) {
                return false;
            }
        }
        true
    }

    /// Persist open files, cursor, scroll, and workspace to disk.
    pub fn persist_session(&mut self) {
        self.session.open_files = self
            .tab_manager
            .tabs()
            .iter()
            .filter_map(|t| t.file_path.clone())
            .collect();
        self.session.active_tab = self.tab_manager.active_index();
        for tab in self.tab_manager.tabs() {
            if let Some(path) = &tab.file_path {
                self.session
                    .update_cursor(path, tab.cursor.line, tab.cursor.col);
                self.session.update_scroll(path, tab.scroll_offset);
            }
        }
        if let Some(root) = &self.workspace_root {
            self.session.set_workspace(root.clone());
        }
        let _ = self.session.save();
    }

    /// Begin application exit; show save prompt when needed.
    pub fn request_exit(&mut self, ctx: &egui::Context) {
        if self.tab_manager.has_unsaved_changes() {
            self.show_quit_unsaved_dialog = true;
            ctx.request_repaint();
        } else {
            self.persist_session();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Finish quit after the user chose to discard unsaved changes.
    pub fn confirm_quit_without_save(&mut self, ctx: &egui::Context) {
        self.show_quit_unsaved_dialog = false;
        self.force_quit = true;
        self.persist_session();
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    /// Finish quit after saving all tabs.
    pub fn confirm_quit_after_save(&mut self, ctx: &egui::Context) -> bool {
        if self.save_all_tabs_for_quit() {
            self.show_quit_unsaved_dialog = false;
            self.force_quit = true;
            self.persist_session();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            true
        } else {
            false
        }
    }

    /// Close the current tab.
    pub fn close_current_tab(&mut self) {
        let idx = self.tab_manager.active_index();
        self.close_tab_at(idx);
    }

    fn is_tab_dirty(&self, index: usize) -> bool {
        self.tab_manager
            .tabs()
            .get(index)
            .map(|t| t.buffer.is_dirty() || t.modified)
            .unwrap_or(false)
    }

    /// Close a single tab by index (prompts when unsaved).
    pub fn close_tab_at(&mut self, index: usize) {
        if index >= self.tab_manager.tab_count() {
            return;
        }
        self.pending_close_batch = None;
        if self.is_tab_dirty(index) {
            self.pending_close_tab = Some(index);
            self.show_unsaved_dialog = true;
        } else if self.tab_manager.close_tab(index) {
            self.highlighter.clear_cache();
            self.highlighter.invalidate_all();
        }
    }

    /// Close every tab except `keep`.
    pub fn close_other_tabs(&mut self, keep: usize) {
        if keep >= self.tab_manager.tab_count() {
            return;
        }
        self.pending_close_batch = Some(TabCloseBatch::Others { keep });
        let indices: Vec<usize> = (0..self.tab_manager.tab_count())
            .filter(|&i| i != keep)
            .collect();
        self.request_close_tab_indices(indices);
    }

    /// Close all tabs to the left of `from`.
    pub fn close_tabs_to_left(&mut self, from: usize) {
        if from == 0 {
            return;
        }
        self.pending_close_batch = Some(TabCloseBatch::LeftOf { from });
        let indices: Vec<usize> = (0..from).collect();
        self.request_close_tab_indices(indices);
    }

    /// Close all tabs to the right of `from`.
    pub fn close_tabs_to_right(&mut self, from: usize) {
        if from + 1 >= self.tab_manager.tab_count() {
            return;
        }
        self.pending_close_batch = Some(TabCloseBatch::RightOf { from });
        let indices: Vec<usize> = (from + 1..self.tab_manager.tab_count()).collect();
        self.request_close_tab_indices(indices);
    }

    fn request_close_tab_indices(&mut self, mut indices: Vec<usize>) {
        indices.sort_unstable_by(|a, b| b.cmp(a));
        let mut closed_any = false;
        for idx in indices {
            if self.tab_manager.tab_count() <= 1 {
                self.pending_close_batch = None;
                break;
            }
            if self.is_tab_dirty(idx) {
                self.pending_close_tab = Some(idx);
                self.show_unsaved_dialog = true;
                return;
            }
            if self.tab_manager.close_tab(idx) {
                closed_any = true;
            }
        }
        if closed_any {
            self.highlighter.clear_cache();
            self.highlighter.invalidate_all();
        }
        self.pending_close_batch = None;
    }

    /// Resume a batch tab close after the unsaved-changes dialog closes one tab.
    pub fn continue_tab_close_batch(&mut self, just_closed: usize) {
        let Some(batch) = self.pending_close_batch else {
            return;
        };
        match batch {
            TabCloseBatch::Others { keep } => {
                let keep = if just_closed < keep {
                    keep.saturating_sub(1)
                } else {
                    keep
                };
                self.close_other_tabs(keep);
            }
            TabCloseBatch::LeftOf { from } => {
                let from = if just_closed < from {
                    from.saturating_sub(1)
                } else {
                    from
                };
                self.close_tabs_to_left(from);
            }
            TabCloseBatch::RightOf { from } => {
                let from = if just_closed <= from {
                    from.saturating_sub(1)
                } else {
                    from
                };
                self.close_tabs_to_right(from);
            }
        }
    }

    pub fn cancel_pending_tab_close(&mut self) {
        self.pending_close_tab = None;
        self.pending_close_batch = None;
        self.show_unsaved_dialog = false;
    }

    pub fn finish_pending_tab_close(&mut self, closed_index: usize) {
        self.pending_close_tab = None;
        self.show_unsaved_dialog = false;
        if self.pending_close_batch.is_some() {
            self.continue_tab_close_batch(closed_index);
        }
    }

    pub fn reveal_tab_in_folder(&self, tab_index: usize) {
        let Some(path) = self
            .tab_manager
            .tabs()
            .get(tab_index)
            .and_then(|t| t.file_path.clone())
        else {
            return;
        };
        crate::platform::shell::reveal_file_in_folder(&path);
    }

    pub fn open_terminal_for_tab(&self, tab_index: usize) {
        let Some(path) = self
            .tab_manager
            .tabs()
            .get(tab_index)
            .and_then(|t| t.file_path.clone())
        else {
            return;
        };
        crate::platform::shell::open_terminal_in_directory(&path);
    }

    pub fn locate_tab_in_file_tree(&mut self, tab_index: usize) {
        let Some(path) = self
            .tab_manager
            .tabs()
            .get(tab_index)
            .and_then(|t| t.file_path.clone())
        else {
            return;
        };
        let Some(root) = self.workspace_root.clone() else {
            return;
        };
        if !path.starts_with(&root) {
            return;
        }
        self.file_tree.reveal_path(&path);
        self.show_sidebar = true;
        self.sidebar_tab = SidebarTab::FileExplorer;
    }

    pub fn copy_tab_path_to_clipboard(&mut self, tab_index: usize) {
        if let Some(path) = self
            .tab_manager
            .tabs()
            .get(tab_index)
            .and_then(|t| t.file_path.as_ref())
        {
            copy_to_clipboard(&path.to_string_lossy());
            self.transient_message = path.display().to_string();
        }
    }

    pub fn begin_rename_tab(&mut self, tab_index: usize) {
        let Some(tab) = self.tab_manager.tabs().get(tab_index) else {
            return;
        };
        if tab.file_path.is_none() {
            return;
        }
        self.rename_tab_index = Some(tab_index);
        self.rename_tab_text = tab.title.clone();
        self.show_rename_tab = true;
    }

    pub fn commit_rename_tab(&mut self) -> bool {
        let Some(idx) = self.rename_tab_index else {
            return false;
        };
        let Some(old_path) = self.tab_manager.tabs()[idx].file_path.clone() else {
            return false;
        };
        let new_name = self.rename_tab_text.trim();
        if new_name.is_empty() {
            return false;
        }
        let new_path = match old_path.parent() {
            Some(parent) => parent.join(new_name),
            None => return false,
        };
        if new_path == old_path {
            return true;
        }
        match std::fs::rename(&old_path, &new_path) {
            Ok(()) => {
                let tab = &mut self.tab_manager.tabs_mut()[idx];
                tab.file_path = Some(new_path.clone());
                tab.title = new_name.to_string();
                self.config.add_recent_file(new_path);
                let _ = self.config.save();
                self.highlighter.clear_cache();
                self.highlighter.invalidate_all();
                true
            }
            Err(e) => {
                log::error!("Failed to rename file: {e}");
                false
            }
        }
    }

    pub fn save_tab_as_at(&mut self, tab_index: usize) {
        self.tab_manager.set_active(tab_index);
        self.save_as_dialog();
    }

    pub fn reload_tab_from_disk(&mut self, tab_index: usize) {
        let Some(path) = self.tab_manager.tabs()[tab_index].file_path.clone() else {
            return;
        };
        self.tab_manager.set_active(tab_index);
        match self.tab_manager.active_mut().open_file(&path) {
            Ok(()) => {
                self.highlighter.clear_cache();
                self.highlighter.invalidate_all();
            }
            Err(e) => log::error!("Failed to reload {}: {e}", path.display()),
        }
    }

    pub fn add_tab_to_favorites(&mut self, tab_index: usize) {
        let Some(path) = self
            .tab_manager
            .tabs()
            .get(tab_index)
            .and_then(|t| t.file_path.clone())
        else {
            return;
        };
        self.config.add_recent_file(path);
        let _ = self.config.save();
        let lang = &self.config.ui.ui_language;
        self.transient_message = if crate::i18n::UiLanguage::parse(lang)
            == crate::i18n::UiLanguage::Zh
        {
            "已添加到收藏夹".to_string()
        } else {
            "Added to favorites".to_string()
        };
    }

    pub fn compare_tab_as_left(&mut self, tab_index: usize) {
        let Some(left) = self.tab_manager.tabs()[tab_index].file_path.clone() else {
            return;
        };
        self.tab_manager.set_active(tab_index);

        if let Some(right) = self.compare_pick_right.take() {
            if left != right {
                if is_likely_binary(&left) || is_likely_binary(&right) {
                    self.open_binary_compare(left, right);
                } else {
                    self.open_file_compare(left, right);
                }
            }
            self.compare_pick_left = None;
            return;
        }

        self.compare_pick_left = Some(left);
        self.transient_message = if crate::i18n::UiLanguage::parse(&self.config.ui.ui_language)
            == crate::i18n::UiLanguage::Zh
        {
            "已选为左侧对比文件，请再选右侧文件".to_string()
        } else {
            "Selected as left compare file; pick the right file".to_string()
        };
    }

    pub fn compare_tab_as_right(&mut self, tab_index: usize) {
        let Some(right) = self.tab_manager.tabs()[tab_index].file_path.clone() else {
            return;
        };
        self.tab_manager.set_active(tab_index);

        if let Some(left) = self.compare_pick_left.take() {
            if left != right {
                if is_likely_binary(&left) || is_likely_binary(&right) {
                    self.open_binary_compare(left, right);
                } else {
                    self.open_file_compare(left, right);
                }
            }
            self.compare_pick_right = None;
            return;
        }

        self.compare_pick_right = Some(right);
        self.transient_message = if crate::i18n::UiLanguage::parse(&self.config.ui.ui_language)
            == crate::i18n::UiLanguage::Zh
        {
            "已选为右侧对比文件，请再选左侧文件".to_string()
        } else {
            "Selected as right compare file; pick the left file".to_string()
        };
    }

    /// Cut selected text to clipboard.
    pub fn cut(&mut self) {
        self.restore_context_menu_selection_if_needed();
        let tab = self.tab_manager.active();
        if tab.selection.is_empty() {
            self.context_menu_selection = None;
            return;
        }
        let (start, end) = tab.selection.to_byte_range(&tab.buffer);
        if let Some(text) = self.get_selected_text() {
            copy_to_clipboard(&text);
            self.clipboard_marks = crate::editor::context_actions::marks_within_char_range(
                &tab.editor_extras,
                &tab.buffer,
                start,
                end,
            );
            let mapped = crate::editor::context_actions::remap_marks_for_delete(
                &tab.editor_extras,
                &tab.buffer,
                start,
                end,
            );
            let normalized = tab.selection.normalized();
            let tab = self.tab_manager.active_mut();
            tab.buffer.delete_range(start, end);
            crate::editor::context_actions::apply_char_marks(
                &mut tab.editor_extras,
                &tab.buffer,
                &mapped,
            );
            tab.cursor = normalized.start;
            tab.selection = Selection::cursor(normalized.start);
            tab.modified = true;
            self.highlighter.invalidate_all();
        }
        self.context_menu_selection = None;
    }

    /// Copy selected text to clipboard (includes background marks for paste).
    pub fn copy(&mut self) {
        self.restore_context_menu_selection_if_needed();
        let selection = self.effective_context_selection();
        if selection.is_empty() {
            return;
        }
        let tab = self.tab_manager.active();
        let text = if tab.column_selection {
            crate::editor::column_selection::extract_text(&tab.buffer, &selection)
        } else if let Some(t) = self.get_selected_text_from(selection) {
            t
        } else {
            return;
        };
        copy_to_clipboard(&text);
        let tab = self.tab_manager.active();
        let (start, end) = if tab.column_selection {
            (0, 0)
        } else {
            selection.to_byte_range(&tab.buffer)
        };
        if !tab.column_selection {
            self.clipboard_marks = crate::editor::context_actions::marks_within_char_range(
                &tab.editor_extras,
                &tab.buffer,
                start,
                end,
            );
        }
    }

    /// Copy rectangular column text from the current selection.
    pub fn copy_column(&mut self) {
        self.restore_context_menu_selection_if_needed();
        let selection = self.effective_context_selection();
        if selection.is_empty() {
            return;
        }
        let tab = self.tab_manager.active();
        let text = crate::editor::column_selection::extract_text(&tab.buffer, &selection);
        if text.is_empty() {
            return;
        }
        copy_to_clipboard(&text);
        self.transient_message = self.tr().edit_copy_column.to_string();
    }

    /// Paste from clipboard.
    pub fn paste(&mut self) {
        if let Some(text) = paste_from_clipboard() {
            self.delete_selection();
            let (line, col) = {
                let tab = self.tab_manager.active();
                (tab.cursor.line, tab.cursor.col)
            };
            let offset = self.tab_manager.active().buffer.char_pos_for_line_col(line, col);
            let insert_len = text.chars().count();
            let added = self.clipboard_marks.clone();
            let tab = self.tab_manager.active();
            let mapped = crate::editor::context_actions::remap_marks_for_insert(
                &tab.editor_extras,
                &tab.buffer,
                offset,
                insert_len,
                &added,
            );
            let tab = self.tab_manager.active_mut();
            tab.buffer.insert_str(offset, &text);
            crate::editor::context_actions::apply_char_marks(
                &mut tab.editor_extras,
                &tab.buffer,
                &mapped,
            );
            let lines: Vec<&str> = text.split('\n').collect();
            if lines.len() > 1 {
                tab.cursor.line += lines.len() - 1;
                tab.cursor.col = lines.last().map(|l| l.chars().count()).unwrap_or(0);
            } else {
                tab.cursor.col += insert_len;
            }
            self.highlighter.invalidate_all();
        }
    }

    /// Select all text in the current buffer.
    pub fn select_all(&mut self) {
        let tab = self.tab_manager.active_mut();
        let last_line = tab.line_count().saturating_sub(1);
        let last_col = tab.buffer.line_len(last_line);
        let end = Cursor::new(last_line, last_col);
        tab.selection = Selection::new(Cursor::new(0, 0), end);
        tab.cursor = end;
    }

    /// Delete the current selection, if any. Returns true when text was removed.
    pub fn delete_selection(&mut self) -> bool {
        let tab = self.tab_manager.active();
        if tab.selection.is_empty() {
            return false;
        }
        let (start, end) = tab.selection.to_byte_range(&tab.buffer);
        let normalized = tab.selection.normalized();
        let mapped = crate::editor::context_actions::remap_marks_for_delete(
            &tab.editor_extras,
            &tab.buffer,
            start,
            end,
        );
        let tab = self.tab_manager.active_mut();
        tab.buffer.delete_range(start, end);
        crate::editor::context_actions::apply_char_marks(&mut tab.editor_extras, &tab.buffer, &mapped);
        tab.cursor = normalized.start;
        tab.selection = Selection::cursor(normalized.start);
        self.highlighter.invalidate_all();
        true
    }

    /// Get selected text from the active tab.
    fn get_selected_text(&self) -> Option<String> {
        self.get_selected_text_from(self.tab_manager.active().selection)
    }

    fn get_selected_text_from(&self, selection: Selection) -> Option<String> {
        if selection.is_empty() {
            return None;
        }
        let tab = self.tab_manager.active();
        let (start, end) = selection.to_byte_range(&tab.buffer);
        Some(tab.buffer.slice(start, end))
    }

    /// Selection to use for context-menu cut/copy/mark (snapshot or live).
    pub(crate) fn effective_context_selection(&self) -> Selection {
        let live = self.tab_manager.active().selection;
        if !live.is_empty() {
            return live;
        }
        self.context_menu_selection.unwrap_or(live)
    }

    fn restore_context_menu_selection_if_needed(&mut self) {
        let live = self.tab_manager.active().selection;
        if live.is_empty() {
            if let Some(saved) = self.context_menu_selection {
                if !saved.is_empty() {
                    self.tab_manager.active_mut().selection = saved;
                }
            }
        }
    }

    /// Toggle macro recording.
    pub fn toggle_macro_recording(&mut self) {
        self.macro_recording = !self.macro_recording;
        if self.macro_recording {
            self.macro_actions.clear();
            log::info!("Macro recording started");
        } else {
            log::info!("Macro recording stopped ({} actions)", self.macro_actions.len());
        }
    }

    /// Toggle command palette visibility.
    pub fn toggle_command_palette(&mut self) {
        self.command_palette.visible = !self.command_palette.visible;
        if self.command_palette.visible {
            self.command_palette.query.clear();
            self.command_palette.selected_index = 0;
        }
    }

    /// Whether a modal dialog should take priority over the editor for focus.
    /// Delete selection or the character at the cursor (editor context menu).
    pub fn editor_delete(&mut self) {
        self.restore_context_menu_selection_if_needed();
        if self.delete_selection() {
            self.context_menu_selection = None;
            return;
        }
        let cursor = self.tab_manager.active().cursor;
        let selection = self.tab_manager.active().selection;
        let tab = self.tab_manager.active_mut();
        if crate::editor::context_actions::delete_at_cursor(
            &mut tab.buffer,
            &cursor,
            &selection,
        ) {
            tab.cursor.col = tab.cursor.col.saturating_sub(1);
            self.highlighter.invalidate_all();
        }
    }

    /// Reveal the active file in the system file manager.
    pub fn show_file_in_explorer(&self) {
        let Some(path) = self.tab_manager.active().file_path.clone() else {
            return;
        };
        crate::platform::shell::reveal_file_in_folder(&path);
    }

    pub fn active_syntax_name(&self) -> String {
        let tab = self.tab_manager.active();
        let filename = tab
            .file_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| tab.title.clone());
        self.highlighter
            .syntax_name_for_file(&filename, tab.syntax_override.as_deref())
    }

    pub fn toggle_line_comment(&mut self) {
        let style =
            crate::editor::context_actions::comment_style_for_syntax(&self.active_syntax_name());
        let tab = self.tab_manager.active_mut();
        let selection = if tab.selection.is_empty() {
            Selection::new(
                Cursor::new(tab.cursor.line, 0),
                Cursor::new(tab.cursor.line, tab.buffer.line_len(tab.cursor.line)),
            )
        } else {
            tab.selection
        };
        crate::editor::context_actions::toggle_line_comments(&mut tab.buffer, &selection, style);
        self.highlighter.invalidate_all();
    }

    pub fn add_block_comment(&mut self) {
        let style =
            crate::editor::context_actions::comment_style_for_syntax(&self.active_syntax_name());
        let tab = self.tab_manager.active_mut();
        if !tab.selection.is_empty() {
            crate::editor::context_actions::add_block_comment(
                &mut tab.buffer,
                &tab.selection,
                style,
            );
            self.highlighter.invalidate_all();
        }
    }

    pub fn remove_block_comment(&mut self) {
        let style =
            crate::editor::context_actions::comment_style_for_syntax(&self.active_syntax_name());
        let tab = self.tab_manager.active_mut();
        if !tab.selection.is_empty() {
            crate::editor::context_actions::remove_block_comment(
                &mut tab.buffer,
                &tab.selection,
                style,
            );
            self.highlighter.invalidate_all();
        }
    }

    pub fn clear_all_bookmarks(&mut self) {
        crate::editor::context_actions::clear_all_bookmarks(
            &mut self.tab_manager.active_mut().editor_extras,
        );
    }

    pub fn mark_selection_with_color(&mut self, color: u8) {
        self.restore_context_menu_selection_if_needed();
        let selection = self.effective_context_selection();
        if selection.is_empty() {
            let line = self.tab_manager.active().cursor.line;
            let line_len = self.tab_manager.active().buffer.line_len(line);
            let whole_line = Selection::new(Cursor::new(line, 0), Cursor::new(line, line_len));
            self.tab_manager
                .active_mut()
                .editor_extras
                .text_marks
                .push(crate::editor::context_actions::TextMark {
                    selection: whole_line.normalized(),
                    color,
                });
            crate::editor::context_actions::rebuild_line_marks(
                &mut self.tab_manager.active_mut().editor_extras,
            );
            crate::editor::context_actions::toggle_bookmark(
                &mut self.tab_manager.active_mut().editor_extras,
                line,
            );
            self.tab_manager.active_mut().selection =
                Selection::cursor(Cursor::new(line, 0));
            self.context_menu_selection = None;
            return;
        }

        let norm = selection.normalized();
        let start_line = norm.start.line;
        self.tab_manager
            .active_mut()
            .editor_extras
            .text_marks
            .push(crate::editor::context_actions::TextMark {
                selection: norm,
                color,
            });
        crate::editor::context_actions::rebuild_line_marks(
            &mut self.tab_manager.active_mut().editor_extras,
        );
        crate::editor::context_actions::toggle_bookmark(
            &mut self.tab_manager.active_mut().editor_extras,
            start_line,
        );
        // Collapse selection so the blue selection highlight does not cover the mark.
        self.tab_manager.active_mut().selection = Selection::cursor(norm.start);
        self.context_menu_selection = None;
    }

    #[allow(dead_code)]
    pub fn mark_current_line_with_color(&mut self, color: u8) {
        self.mark_selection_with_color(color);
    }

    pub fn insert_at_cursor(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.delete_selection();
        let (line, col) = {
            let tab = self.tab_manager.active();
            (tab.cursor.line, tab.cursor.col)
        };
        let offset = self
            .tab_manager
            .active()
            .buffer
            .char_pos_for_line_col(line, col);
        let insert_len = text.chars().count();
        let tab = self.tab_manager.active();
        let mapped = crate::editor::context_actions::remap_marks_for_insert(
            &tab.editor_extras,
            &tab.buffer,
            offset,
            insert_len,
            &[],
        );
        let tab = self.tab_manager.active_mut();
        tab.buffer.insert_str(offset, text);
        crate::editor::context_actions::apply_char_marks(
            &mut tab.editor_extras,
            &tab.buffer,
            &mapped,
        );
        if text.contains('\n') {
            let lines: Vec<&str> = text.split('\n').collect();
            tab.cursor.line += lines.len() - 1;
            tab.cursor.col = lines.last().map(|l| l.chars().count()).unwrap_or(0);
        } else {
            tab.cursor.col += insert_len;
        }
        tab.modified = true;
        self.highlighter.invalidate_all();
    }

    /// Delete one character before the cursor, keeping color marks in sync.
    pub fn delete_char_before_cursor(&mut self) {
        if self.delete_selection() {
            return;
        }
        let line = self.tab_manager.active().cursor.line;
        let col = self.tab_manager.active().cursor.col;
        if col > 0 {
            let pos = self
                .tab_manager
                .active()
                .buffer
                .char_pos_for_line_col(line, col);
            let del_start = pos - 1;
            let tab = self.tab_manager.active();
            let mapped = crate::editor::context_actions::remap_marks_for_delete(
                &tab.editor_extras,
                &tab.buffer,
                del_start,
                pos,
            );
            let tab = self.tab_manager.active_mut();
            tab.buffer.delete_range(del_start, pos);
            crate::editor::context_actions::apply_char_marks(
                &mut tab.editor_extras,
                &tab.buffer,
                &mapped,
            );
            tab.cursor.col -= 1;
            tab.modified = true;
            self.highlighter.invalidate_all();
        } else if line > 0 {
            let prev_line_len = self.tab_manager.active().buffer.line_len(line - 1);
            let pos = self
                .tab_manager
                .active()
                .buffer
                .char_pos_for_line_col(line, 0);
            if pos > 0 {
                let del_start = pos - 1;
                let tab = self.tab_manager.active();
                let mapped = crate::editor::context_actions::remap_marks_for_delete(
                    &tab.editor_extras,
                    &tab.buffer,
                    del_start,
                    pos,
                );
                let tab = self.tab_manager.active_mut();
                tab.buffer.delete_range(del_start, pos);
                crate::editor::context_actions::apply_char_marks(
                    &mut tab.editor_extras,
                    &tab.buffer,
                    &mapped,
                );
                tab.cursor.line -= 1;
                tab.cursor.col = prev_line_len;
                tab.modified = true;
                self.highlighter.invalidate_all();
            }
        }
    }

    /// Delete one character at the cursor, keeping color marks in sync.
    pub fn delete_char_at_cursor(&mut self) {
        if self.delete_selection() {
            return;
        }
        let line = self.tab_manager.active().cursor.line;
        let col = self.tab_manager.active().cursor.col;
        let pos = self
            .tab_manager
            .active()
            .buffer
            .char_pos_for_line_col(line, col);
        if pos < self.tab_manager.active().buffer.len() {
            let del_end = pos + 1;
            let tab = self.tab_manager.active();
            let mapped = crate::editor::context_actions::remap_marks_for_delete(
                &tab.editor_extras,
                &tab.buffer,
                pos,
                del_end,
            );
            let tab = self.tab_manager.active_mut();
            tab.buffer.delete_range(pos, del_end);
            crate::editor::context_actions::apply_char_marks(
                &mut tab.editor_extras,
                &tab.buffer,
                &mapped,
            );
            tab.modified = true;
            self.highlighter.invalidate_all();
        }
    }

    pub fn insert_current_datetime(&mut self) {
        self.insert_at_cursor(&local_timestamp());
    }

    pub fn show_word_count(&mut self) {
        let text = self.tab_manager.active().buffer.text();
        let (chars, words, lines) = crate::editor::context_actions::word_count(&text);
        let lang = self.ui_lang();
        self.transient_message = if crate::i18n::UiLanguage::parse(lang) == crate::i18n::UiLanguage::Zh
        {
            format!("字数统计：{chars} 字符，{words} 词，{lines} 行")
        } else {
            format!("Word count: {chars} characters, {words} words, {lines} lines")
        };
    }

    pub fn open_markdown_preview(&mut self) {
        let text = self.tab_manager.active().buffer.text();
        self.html_preview_content = crate::editor::context_actions::markdown_to_html(&text);
        self.html_preview_title = "Markdown Preview".to_string();
        self.show_html_preview = true;
    }

    pub fn export_markdown_to_html(&mut self) {
        let text = self.tab_manager.active().buffer.text();
        self.html_preview_content = crate::editor::context_actions::markdown_to_html(&text);
        self.html_preview_title = "Markdown → HTML".to_string();
        self.show_html_preview = true;
    }

    pub fn export_text_to_html(&mut self) {
        let text = self.tab_manager.active().buffer.text();
        self.html_preview_content = crate::editor::context_actions::text_to_html(&text);
        self.html_preview_title = "Text → HTML".to_string();
        self.show_html_preview = true;
    }

    pub fn save_html_preview(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("HTML", &["html", "htm"])
            .save_file()
        {
            let content = self.html_preview_content.clone();
            let op = self.tr().err_op_export;
            let _ = self.write_file_with_feedback(path, content, op);
        }
    }

    pub fn output_selection_to_file(&mut self) {
        let tab = self.tab_manager.active();
        let text = if tab.selection.is_empty() {
            tab.buffer
                .line(tab.cursor.line)
                .unwrap_or_default()
                .to_string()
        } else {
            let (start, end) = tab.selection.to_byte_range(&tab.buffer);
            tab.buffer.slice(start, end)
        };
        if let Some(path) = rfd::FileDialog::new().save_file() {
            let op = self.tr().err_op_export;
            let _ = self.write_file_with_feedback(path, text, op);
        }
    }

    pub fn refresh_folds(&mut self) {
        let text = self.tab_manager.active().buffer.text();
        self.tab_manager
            .active_mut()
            .editor_extras
            .fold_state
            .detect_folds(&text);
    }

    pub fn fold_at_cursor(&mut self) {
        self.refresh_folds();
        let line = self.tab_manager.active().cursor.line;
        self.tab_manager
            .active_mut()
            .editor_extras
            .fold_state
            .toggle_fold_at_line(line);
    }

    pub fn unfold_all(&mut self) {
        for range in &mut self.tab_manager.active_mut().editor_extras.fold_state.ranges {
            range.folded = false;
        }
    }

    pub fn fold_all(&mut self) {
        self.refresh_folds();
        for range in &mut self.tab_manager.active_mut().editor_extras.fold_state.ranges {
            range.folded = true;
        }
    }

    pub fn unfold_at_cursor(&mut self) {
        self.refresh_folds();
        let line = self.tab_manager.active().cursor.line;
        self.tab_manager
            .active_mut()
            .editor_extras
            .fold_state
            .unfold_at_line(line);
    }

    pub fn toggle_wrap_by_character(&mut self) {
        self.config.editor.wrap_by_character = !self.config.editor.wrap_by_character;
        self.config.editor.word_wrap = self.config.editor.wrap_by_character;
    }

    pub fn blocks_editor_focus(&self) -> bool {
        self.show_search
            || self.show_cross_file_search
            || self.show_goto_line
            || self.show_batch_encoding
            || self.show_preferences
            || self.show_keybindings
            || self.show_about
            || self.show_unsaved_dialog
            || self.show_quit_unsaved_dialog
            || self.show_save_error_dialog
            || self.show_html_preview
    }

    /// Open find/replace dialog and run an initial search.
    pub fn open_find(&mut self, replace_mode: bool) {
        self.search_dialog_tab = if replace_mode {
            SearchDialogTab::Replace
        } else {
            SearchDialogTab::Find
        };
        self.search_replace_mode = replace_mode;
        self.show_search = true;
        self.search_focus_input = true;
        self.search_status_message.clear();
        // Release editor focus so the dialog text fields can receive keyboard input.
        self.editor_has_focus = false;

        if let Some(selected) = self.get_selected_text() {
            if !selected.is_empty() && !selected.contains('\n') {
                self.search_pattern = selected;
            }
        }

        if !self.search_pattern.is_empty() {
            self.refresh_search_results(true);
            let n = self.search_engine.results().len();
            self.search_status_message = crate::i18n::msg_found_matches(self.ui_lang(), n);
            if n > 0 {
                self.show_search_results = true;
            }
        }
    }

    fn remember_search_pattern(&mut self) {
        if self.search_pattern.is_empty() {
            return;
        }
        self.search_history
            .retain(|s| s != &self.search_pattern);
        self.search_history
            .insert(0, self.search_pattern.clone());
        self.search_history.truncate(20);
    }

    /// Find next respecting the Backward direction option.
    pub fn find_next_in_dialog(&mut self) {
        if self.search_options.backward {
            self.find_prev_match();
        } else {
            self.find_next_match();
        }
        self.remember_search_pattern();
        self.update_search_status_after_nav();
    }

    /// Find previous respecting the Backward direction option.
    pub fn find_prev_in_dialog(&mut self) {
        if self.search_options.backward {
            self.find_next_match();
        } else {
            self.find_prev_match();
        }
        self.remember_search_pattern();
        self.update_search_status_after_nav();
    }

    fn update_search_status_after_nav(&mut self) {
        let lang = self.ui_lang().to_string();
        let total = self.search_engine.results().len();
        if total == 0 {
            self.search_status_message = crate::i18n::msg_no_matches(&lang);
        } else if let Some(idx) = self.search_engine.current_index() {
            self.search_status_message =
                crate::i18n::msg_match_of(&lang, idx + 1, total);
        }
    }

    /// Count matches in the current document.
    pub fn search_count_current(&mut self) {
        self.refresh_search_results(false);
        let count = self.search_engine.results().len();
        self.search_status_message = crate::i18n::msg_count_matches(self.ui_lang(), count);
    }

    /// Find all matches in the current document and jump to the first.
    pub fn find_all_in_current_document(&mut self) {
        self.refresh_search_results(true);
        let count = self.search_engine.results().len();
        let items = self.build_search_items_from_engine_active_tab();
        let scope = if crate::i18n::UiLanguage::parse(self.ui_lang()) == crate::i18n::UiLanguage::Zh {
            "当前文件"
        } else {
            "Current file"
        };
        self.append_search_batch(scope.to_string(), items);
        self.search_status_message =
            crate::i18n::msg_find_all_current(self.ui_lang(), count);
        self.remember_search_pattern();
    }

    /// Report match counts across every open tab.
    pub fn find_all_in_open_documents(&mut self) {
        self.sync_search_engine_options();
        let pattern = self.search_pattern.clone();
        if pattern.is_empty() {
            self.search_status_message =
                crate::i18n::msg_enter_search_text(self.ui_lang());
            return;
        }

        let mut total = 0usize;
        let mut parts = Vec::new();
        let mut items = Vec::new();
        for (tab_idx, tab) in self.tab_manager.tabs().iter().enumerate() {
            let mut engine = SearchEngine::new();
            engine.set_options(self.search_options.clone());
            let matches = engine.find_all(&tab.buffer.text(), &pattern);
            let n = matches.len();
            if n > 0 {
                total += n;
                let title = tab.display_title();
                parts.push(format!("{title}: {n}"));
                for m in &matches {
                    let preview = tab
                        .buffer
                        .line(m.line)
                        .unwrap_or_default()
                        .trim_end()
                        .to_string();
                    items.push(SearchResultItem {
                        tab: tab_idx,
                        doc: title.clone(),
                        line: m.line,
                        col: m.col,
                        start: m.start,
                        end: m.end,
                        preview,
                    });
                }
            }
        }

        let scope = if crate::i18n::UiLanguage::parse(self.ui_lang()) == crate::i18n::UiLanguage::Zh {
            "所有打开文件"
        } else {
            "All open files"
        };
        self.append_search_batch(scope.to_string(), items);
        self.sync_search_engine_for_active_tab();
        self.search_status_message = crate::i18n::msg_find_all_open(
            self.ui_lang(),
            total,
            &parts.join(", "),
        );
        self.remember_search_pattern();
    }

    pub fn clear_search_status(&mut self) {
        self.search_status_message.clear();
        self.search_engine.set_current_index(None);
        self.search_result_batches.clear();
        self.active_result_batch_id = None;
        self.show_search_results = false;
    }

    fn append_search_batch(&mut self, scope_label: String, items: Vec<SearchResultItem>) {
        let id = self.next_search_batch_id;
        self.next_search_batch_id += 1;
        self.search_result_batches.insert(
            0,
            SearchResultBatch {
                id,
                pattern: self.search_pattern.clone(),
                scope_label,
                items,
                collapsed: false,
                collapsed_docs: HashSet::new(),
            },
        );
        self.active_result_batch_id = Some(id);
        self.show_search_results = true;
    }

    fn build_search_items_from_engine_active_tab(&self) -> Vec<SearchResultItem> {
        let tab_idx = self.tab_manager.active_index();
        let doc = self.tab_manager.active().display_title();
        self.search_engine
            .results()
            .iter()
            .map(|m| {
                let preview = self
                    .tab_manager
                    .active()
                    .buffer
                    .line(m.line)
                    .unwrap_or_default()
                    .trim_end()
                    .to_string();
                SearchResultItem {
                    tab: tab_idx,
                    doc: doc.clone(),
                    line: m.line,
                    col: m.col,
                    start: m.start,
                    end: m.end,
                    preview,
                }
            })
            .collect()
    }

    pub fn search_result_total_matches(&self) -> usize {
        self.search_result_batches
            .iter()
            .map(|b| b.items.len())
            .sum()
    }

    /// Sync UI search options into the search engine.
    fn sync_search_engine_options(&mut self) {
        self.search_engine
            .set_options(self.search_options.clone());
    }

    /// Re-run search in the active tab only (keeps the results panel list unchanged).
    fn sync_search_engine_for_active_tab(&mut self) {
        if self.search_pattern.is_empty() {
            return;
        }
        self.sync_search_engine_options();
        let text = self.tab_manager.active().buffer.text();
        let pattern = self.search_pattern.clone();
        self.search_engine.find_all(&text, &pattern);
    }

    /// Re-run search in the active tab and optionally jump to the first match.
    pub fn refresh_search_results(&mut self, jump_to_first: bool) {
        self.sync_search_engine_options();
        let text = self.tab_manager.active().buffer.text();
        let pattern = self.search_pattern.clone();
        self.search_engine.find_all(&text, &pattern);

        if jump_to_first {
            if self.search_engine.next_match().is_some() {
                let m = self.search_engine.results()[0].clone();
                self.go_to_search_match(&m);
            }
        } else if let Some(idx) = self.search_engine.current_index() {
            if let Some(m) = self.search_engine.results().get(idx).cloned() {
                self.go_to_search_match(&m);
            }
        }
    }

    /// Jump to a result-list entry, switching tabs if needed.
    pub fn jump_to_batch_item(&mut self, batch_id: usize, item_idx: usize) {
        let Some(batch) = self
            .search_result_batches
            .iter()
            .find(|b| b.id == batch_id)
            .cloned()
        else {
            return;
        };
        let Some(item) = batch.items.get(item_idx).cloned() else {
            return;
        };

        self.active_result_batch_id = Some(batch_id);
        self.search_pattern = batch.pattern;

        if item.tab < self.tab_manager.tabs().len() {
            self.tab_manager.set_active(item.tab);
        }
        self.sync_search_engine_for_active_tab();
        if let Some(engine_idx) = self
            .search_engine
            .results()
            .iter()
            .position(|m| m.start == item.start && m.line == item.line)
        {
            self.search_engine.set_current_index(Some(engine_idx));
        }
        let m = crate::search::SearchMatch {
            start: item.start,
            end: item.end,
            text: String::new(),
            line: item.line,
            col: item.col,
        };
        self.go_to_search_match(&m);
        self.update_search_status_after_nav();
        self.show_search_results = true;
        self.editor_has_focus = true;
    }

    /// Move the editor cursor/selection to a search match and scroll it into view.
    pub fn go_to_search_match(&mut self, m: &crate::search::SearchMatch) {
        let (start_line, start_col) = self
            .tab_manager
            .active()
            .buffer
            .line_col_for_char_pos(m.start);
        let (end_line, end_col) = self
            .tab_manager
            .active()
            .buffer
            .line_col_for_char_pos(m.end);

        let tab = self.tab_manager.active_mut();
        // Invalidate auto-scroll guard so editor_view scrolls the match into view.
        tab.last_auto_scroll_cursor = tab.cursor;
        tab.cursor = Cursor::new(end_line, end_col);
        tab.selection = Selection::new(
            Cursor::new(start_line, start_col),
            Cursor::new(end_line, end_col),
        );
    }

    /// Find and highlight the next match.
    pub fn find_next_match(&mut self) {
        if self.search_engine.results().is_empty() && !self.search_pattern.is_empty() {
            self.refresh_search_results(true);
            return;
        }
        if let Some(m) = self.search_engine.next_match() {
            let m = m.clone();
            self.go_to_search_match(&m);
            self.show_search_results = true;
        }
    }

    /// Find and highlight the previous match.
    pub fn find_prev_match(&mut self) {
        if self.search_engine.results().is_empty() && !self.search_pattern.is_empty() {
            self.refresh_search_results(true);
            return;
        }
        if let Some(m) = self.search_engine.prev_match() {
            let m = m.clone();
            self.go_to_search_match(&m);
            self.show_search_results = true;
        }
    }

    /// Replace the current match and advance to the next one.
    pub fn replace_current_match(&mut self) {
        if self.search_engine.results().is_empty() && !self.search_pattern.is_empty() {
            self.refresh_search_results(true);
        }
        let replacement = self.replace_pattern.clone();
        if let Some((start, end, new_text)) = self.search_engine.replace_current("", &replacement)
        {
            self.tab_manager
                .active_mut()
                .buffer
                .replace_range(start, end, &new_text);
            self.highlighter.invalidate_all();
            self.refresh_search_results(false);
            self.find_next_match();
        }
    }

    /// Replace every match in the active tab.
    pub fn replace_all_matches(&mut self) {
        if self.search_engine.results().is_empty() && !self.search_pattern.is_empty() {
            self.refresh_search_results(false);
        }
        let replacement = self.replace_pattern.clone();
        let matches: Vec<_> = self.search_engine.results().to_vec();
        if matches.is_empty() {
            return;
        }
        {
            let buffer = &mut self.tab_manager.active_mut().buffer;
            for m in matches.iter().rev() {
                buffer.replace_range(m.start, m.end, &replacement);
            }
        }
        self.highlighter.invalidate_all();
        self.refresh_search_results(false);
    }

    /// Perform cross-file search.
    pub fn perform_cross_file_search(&mut self) {
        use std::fs;

        self.cross_file_results.clear();
        let dir = PathBuf::from(&self.cross_file_directory);
        if !dir.exists() {
            return;
        }

        let pattern = &self.search_pattern;
        if pattern.is_empty() {
            return;
        }

        let filter = &self.cross_file_filter;
        let extensions: Vec<&str> = filter
            .split(',')
            .map(|s| s.trim().trim_start_matches("*."))
            .collect();

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path
                        .extension()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if extensions.iter().any(|e| e == &ext || *e == "*") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            for (line_num, line) in content.lines().enumerate() {
                                if line.contains(pattern) {
                                    self.cross_file_results.push((
                                        path.clone(),
                                        line_num,
                                        line.to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Perform auto-save if needed.
    pub fn auto_save_check(&mut self) {
        if self.auto_save.should_save() {
            for tab in self.tab_manager.tabs() {
                if let Some(path) = &tab.file_path {
                    if tab.buffer.is_dirty() {
                        let _ = self.auto_save.write_crash_recovery(path, &tab.buffer.text());
                    }
                }
            }
            self.auto_save.mark_saved();
        }
    }

    /// Process pending keyboard shortcut actions.
    fn process_pending_actions(&mut self) {
        if self.pending_new_tab {
            self.pending_new_tab = false;
            self.tab_manager.new_tab();
        }
        if self.pending_save {
            self.pending_save = false;
            self.save_current_tab();
        }
        if self.pending_close {
            self.pending_close = false;
            self.close_current_tab();
        }
        if self.pending_find {
            self.pending_find = false;
            self.open_find(false);
        }
        if self.pending_replace {
            self.pending_replace = false;
            self.open_find(true);
        }
        if self.pending_find_next {
            self.pending_find_next = false;
            if self.show_search {
                self.find_next_in_dialog();
            } else if !self.search_pattern.is_empty() {
                self.find_next_match();
            } else {
                self.open_find(false);
            }
        }
        if self.pending_find_prev {
            self.pending_find_prev = false;
            if self.show_search {
                self.find_prev_in_dialog();
            } else if !self.search_pattern.is_empty() {
                self.find_prev_match();
            } else {
                self.open_find(false);
            }
        }
        if self.pending_goto {
            self.pending_goto = false;
            self.show_goto_line = true;
        }
        if self.pending_diff {
            self.pending_diff = false;
            self.open_compare_window();
        }
        if self.pending_undo {
            self.pending_undo = false;
            self.tab_manager.active_mut().buffer.undo();
        }
        if self.pending_redo {
            self.pending_redo = false;
            self.tab_manager.active_mut().buffer.redo();
        }
        if self.pending_select_all {
            self.pending_select_all = false;
            self.select_all();
        }
        if self.pending_next_tab {
            self.pending_next_tab = false;
            let next = (self.tab_manager.active_index() + 1) % self.tab_manager.tab_count();
            self.tab_manager.set_active(next);
        }
        if self.pending_prev_tab {
            self.pending_prev_tab = false;
            let count = self.tab_manager.tab_count();
            let prev = if count == 0 {
                0
            } else {
                (self.tab_manager.active_index() + count - 1) % count
            };
            self.tab_manager.set_active(prev);
        }
        if self.pending_command_palette {
            self.pending_command_palette = false;
            self.toggle_command_palette();
        }
        if self.pending_find_in_files {
            self.pending_find_in_files = false;
            self.editor_has_focus = false;
            self.show_cross_file_search = true;
        }
        if self.pending_cut {
            self.pending_cut = false;
            self.cut();
            self.highlighter.invalidate_all();
        }
        if self.pending_copy {
            self.pending_copy = false;
            self.copy();
        }
        if self.pending_paste {
            self.pending_paste = false;
            self.paste();
            self.highlighter.invalidate_all();
        }
        if self.pending_save_as {
            self.pending_save_as = false;
            self.save_as_dialog();
        }
        if self.pending_save_all {
            self.pending_save_all = false;
            self.save_all_tabs();
        }
        if self.pending_toggle_sidebar {
            self.pending_toggle_sidebar = false;
            self.show_sidebar = !self.show_sidebar;
        }
    }

    /// Collect keyboard shortcut flags using the keybinding system.
    fn collect_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F3) {
                self.pending_find_next = true;
            }
            if i.key_pressed(egui::Key::F4) {
                self.pending_find_prev = true;
            }
            if self.keybindings.is_command_pressed(&Command::NewTab, i) {
                self.pending_new_tab = true;
            }
            if self.keybindings.is_command_pressed(&Command::OpenFile, i) {
                self.pending_open_file = true;
            }
            if self.keybindings.is_command_pressed(&Command::Save, i) {
                self.pending_save = true;
            }
            if self.keybindings.is_command_pressed(&Command::CloseTab, i) {
                self.pending_close = true;
            }
            if self.keybindings.is_command_pressed(&Command::Find, i) {
                self.pending_find = true;
            }
            if self.keybindings.is_command_pressed(&Command::Replace, i) {
                self.pending_replace = true;
            }
            if self.keybindings.is_command_pressed(&Command::GotoLine, i) {
                self.pending_goto = true;
            }
            if self.keybindings.is_command_pressed(&Command::ToggleDiffView, i) {
                self.pending_diff = true;
            }
            if self.keybindings.is_command_pressed(&Command::Undo, i) {
                self.pending_undo = true;
            }
            if self.keybindings.is_command_pressed(&Command::Redo, i) {
                self.pending_redo = true;
            }
            if self.keybindings.is_command_pressed(&Command::SelectAll, i) {
                self.pending_select_all = true;
            }
            if self.keybindings.is_command_pressed(&Command::NextTab, i) {
                self.pending_next_tab = true;
            }
            if self.keybindings.is_command_pressed(&Command::PrevTab, i) {
                self.pending_prev_tab = true;
            }
            if self.keybindings.is_command_pressed(&Command::Palette, i) {
                self.pending_command_palette = true;
            }
            if self.keybindings.is_command_pressed(&Command::FindInFiles, i) {
                self.pending_find_in_files = true;
            }
            if self.keybindings.is_command_pressed(&Command::Cut, i) {
                self.pending_cut = true;
            }
            if self.keybindings.is_command_pressed(&Command::Copy, i) {
                self.pending_copy = true;
            }
            if self.keybindings.is_command_pressed(&Command::CopyColumn, i) {
                self.copy_column();
            }
            if self.keybindings.is_command_pressed(&Command::Paste, i) {
                self.pending_paste = true;
            }
            if self.keybindings.is_command_pressed(&Command::SaveAs, i) {
                self.pending_save_as = true;
            }
            if self.keybindings.is_command_pressed(&Command::SaveAll, i) {
                self.pending_save_all = true;
            }
            if self.keybindings.is_command_pressed(&Command::ToggleSidebar, i) {
                self.pending_toggle_sidebar = true;
            }
            if self.keybindings.is_command_pressed(&Command::Exit, i) {
                self.pending_exit = true;
            }
        });
    }
}

/// Load system CJK fonts so Chinese characters render correctly.
fn setup_cjk_fonts(cc: &eframe::CreationContext<'_>) {
    #[cfg(target_os = "macos")]
    {
        let font_paths = [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
        ];
        for path in font_paths {
            if let Ok(data) = std::fs::read(path) {
                let mut fonts = egui::FontDefinitions::default();
                fonts
                    .font_data
                    .insert("cjk".to_owned(), egui::FontData::from_owned(data));
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "cjk".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "cjk".to_owned());
                cc.egui_ctx.set_fonts(fonts);
                log::info!("Loaded CJK font from {}", path);
                return;
            }
        }
        log::warn!("No CJK system font found; Chinese may not render correctly");
    }
}

impl eframe::App for RustpadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_close_request(ctx);
        self.sync_theme(ctx);

        // Save prompts block interaction but the normal (bright) UI still renders
        // behind them, so the window keeps the app's light styling.
        let blocking_dialog = self.show_quit_unsaved_dialog
            || self.show_unsaved_dialog
            || self.show_save_error_dialog;

        if !blocking_dialog {
            #[cfg(target_os = "macos")]
            {
                crate::platform::macos_menu::drain_events(self, ctx);
                crate::platform::macos_menu::sync_encoding_open_checks(self);
            }
            self.collect_shortcuts(ctx);
            self.handle_dropped_files(ctx);
            self.process_pending_actions();
        }
        self.update_status_bar();
        self.auto_save_check();

        // Keep minimap visibility in sync with preferences.
        self.minimap.enabled = self.config.ui.show_minimap;

        crate::ui::menu::show(self, ctx);
        crate::ui::toolbar::show(self, ctx);
        crate::ui::tab_bar::show(self, ctx);
        crate::ui::sidebar::show(self, ctx);
        // Bottom panels must be registered before the central editor panel so the
        // editor (and its quick-scroll arrows) shrink instead of being covered.
        crate::ui::status_bar::show(self, ctx);
        crate::ui::search_panel::show_results_panel(self, ctx);
        crate::ui::editor_view::show(self, ctx);
        // Search dialog must render after the editor so it stays on top and keeps focus.
        crate::ui::search_panel::show(self, ctx);
        crate::ui::search_panel::show_cross_file_search(self, ctx);
        crate::ui::dialogs::show_non_blocking(self, ctx);
        crate::ui::compare_viewport::show_all(self, ctx);

        // Blocking save prompts render last, on top of the bright UI.
        if self.show_quit_unsaved_dialog {
            crate::ui::dialogs::show_quit_unsaved_dialog(self, ctx);
        }
        if self.show_unsaved_dialog {
            crate::ui::dialogs::show_unsaved_dialog(self, ctx);
        }
        if self.show_save_error_dialog {
            crate::ui::dialogs::show_save_error_dialog(self, ctx);
        }

        if let Some(until) = self.splash_until {
            if ctx.input(|i| i.time) < until {
                crate::branding::paint_startup_splash(ctx, &self.logo_texture);
                ctx.request_repaint();
            } else {
                self.splash_until = None;
            }
        }

        // Native file dialogs must run after UI rendering (macOS requirement).
        if self.pending_open_file {
            self.pending_open_file = false;
            self.open_file_dialog();
        }

        if self.pending_compare_files {
            self.pending_compare_files = false;
            self.compare_files_dialog();
        }

        if self.pending_compare_current {
            self.pending_compare_current = false;
            self.compare_current_with_dialog();
        }

        if self.pending_compare_dirs {
            self.pending_compare_dirs = false;
            self.compare_dirs_dialog();
        }

        if self.pending_compare_binary {
            self.pending_compare_binary = false;
            self.compare_binary_files_dialog();
        }

        if self.pending_exit {
            self.pending_exit = false;
            self.request_exit(ctx);
        }

        // Periodically persist session so reopening restores the last workspace.
        if self.last_session_persist.elapsed().as_secs() >= 2 {
            self.persist_session();
            self.last_session_persist = std::time::Instant::now();
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        log::info!("Saving application state...");
        self.persist_session();
        let _ = self.config.save();
    }
}

fn local_timestamp() -> String {
    #[cfg(unix)]
    {
        std::process::Command::new("date")
            .arg("+%Y-%m-%d %H:%M:%S")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "1970-01-01 00:00:00".to_string())
    }
    #[cfg(not(unix))]
    {
        "1970-01-01 00:00:00".to_string()
    }
}

fn copy_to_clipboard(text: &str) {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(text.to_string());
    }
}

fn paste_from_clipboard() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut c| c.get_text().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_default() {
        let status = StatusBar::default();
        assert_eq!(status.line, 0);
        assert_eq!(status.column, 0);
        assert!(status.encoding.is_empty());
    }

    #[test]
    fn test_command_palette_default() {
        let palette = CommandPalette::default();
        assert!(!palette.visible);
        assert!(palette.query.is_empty());
        assert_eq!(palette.selected_index, 0);
    }

    #[test]
    fn test_copy_paste_clipboard() {
        let text = "Hello, RustPad!";
        copy_to_clipboard(text);
    }

    #[test]
    fn test_select_all_sets_full_range() {
        let mut tab_manager = TabManager::new();
        tab_manager.active_mut().buffer.insert_str(0, "hello\nworld");
        let last_line = tab_manager.active().line_count().saturating_sub(1);
        let last_col = tab_manager.active().buffer.line_len(last_line);
        tab_manager.active_mut().selection = Selection::new(
            Cursor::new(0, 0),
            Cursor::new(last_line, last_col),
        );
        tab_manager.active_mut().cursor = Cursor::new(last_line, last_col);
        assert!(!tab_manager.active().selection.is_empty());
        let norm = tab_manager.active().selection.normalized();
        assert_eq!(norm.end.line, 1);
        assert_eq!(norm.end.col, 5);
    }
}
