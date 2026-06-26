//! Unified pop-up compare window (notepad-- CompareWin style).

use std::path::{Path, PathBuf};

use eframe::egui;

use crate::diff::{
    compare_binary_files, is_likely_binary, BinaryDiffResult, DiffAlgorithm, DiffEngine,
    DiffOptions, DiffResult, FolderDiff, FolderDiffFilter, FolderDiffOptions,
    FolderDiffResult, SyncDirection, SyncScope, sync_folder,
};

type LineRangePair = (std::ops::Range<usize>, std::ops::Range<usize>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompareMode {
    #[default]
    None,
    Text,
    Binary,
    Folder,
}

/// How this compare window was opened (controls pickers and validation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompareIntent {
    #[default]
    File,
    Folder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidePreviewKind {
    #[default]
    None,
    Text,
    Binary,
    Folder,
}

/// Manages detached compare viewports.
#[derive(Default)]
pub struct CompareWindowManager {
    next_id: u64,
    pub sessions: Vec<CompareSession>,
}

impl CompareWindowManager {
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn new_window(&mut self) -> u64 {
        self.new_file_window()
    }

    pub fn new_file_window(&mut self) -> u64 {
        let id = self.alloc_id();
        let mut session = CompareSession::new_empty(id);
        session.compare_intent = CompareIntent::File;
        self.sessions.push(session);
        id
    }

    pub fn new_folder_window(&mut self) -> u64 {
        let id = self.alloc_id();
        let mut session = CompareSession::new_empty(id);
        session.compare_intent = CompareIntent::Folder;
        self.sessions.push(session);
        id
    }

    pub fn open_with_paths(&mut self, left: PathBuf, right: PathBuf) -> u64 {
        let intent = if left.is_dir() || right.is_dir() {
            CompareIntent::Folder
        } else {
            CompareIntent::File
        };
        let id = match intent {
            CompareIntent::File => self.new_file_window(),
            CompareIntent::Folder => self.new_folder_window(),
        };
        if let Some(session) = self.session_mut(id) {
            session.left_path_text = left.display().to_string();
            session.right_path_text = right.display().to_string();
            session.refresh_display();
        }
        id
    }

    pub fn session_mut(&mut self, id: u64) -> Option<&mut CompareSession> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    pub fn session(&self, id: u64) -> Option<&CompareSession> {
        self.sessions.iter().find(|s| s.id == id)
    }

    pub fn retain_open(&mut self) {
        self.sessions.retain(|s| s.open);
    }
}

pub struct CompareSession {
    pub id: u64,
    pub viewport_id: egui::ViewportId,
    pub open: bool,

    /// File compare vs folder compare (set when the window is created).
    pub compare_intent: CompareIntent,
    /// Editable path inputs (draft; do not drive the session bar until commit).
    pub left_path_text: String,
    pub right_path_text: String,
    /// Committed paths shown in the session bar and used for compare content.
    pub left_path: PathBuf,
    pub right_path: PathBuf,

    pub mode: CompareMode,
    pub status_message: String,
    pub font_size: f32,
    pub show_whitespace: bool,
    pub show_diff_map: bool,
    pub strict_mode: bool,
    /// When true, diff map / navigation show all lines; when false, compress equal runs.
    pub expand_unchanged: bool,
    pub word_wrap: bool,

    pub algorithm: DiffAlgorithm,
    pub ignore_whitespace: bool,
    pub ignore_case: bool,

    pub text_result: Option<DiffResult>,
    pub left_text: String,
    pub right_text: String,
    pub left_dirty: bool,
    pub right_dirty: bool,
    pub text_current_change: usize,
    pub text_scroll_to_row: Option<usize>,
    /// Set when an inline diff cell edit changes text; recomputed once per frame.
    pub text_edit_pending_recompute: bool,
    pub text_scroll_to_line: Option<usize>,
    pub cmp_sync_scroll_y: f32,
    /// Inline diff cell being edited: (left side?, 0-based line index).
    pub cmp_editing: Option<(bool, usize)>,
    edit_undo_stack: Vec<(String, String)>,

    pub folder_options: FolderDiffOptions,
    pub folder_filter: FolderDiffFilter,
    pub folder_result: Option<FolderDiffResult>,
    pub folder_selected: Option<usize>,
    pub folder_sync_message: String,

    pub binary_result: Option<BinaryDiffResult>,
    pub binary_current_diff: usize,
    pub binary_scroll_to_row: Option<usize>,

    pub pending_save_left: bool,
    pub pending_save_right: bool,
    pub pending_export: bool,
    pub pending_next: bool,
    pub pending_prev: bool,
    pub pending_open_file_compare: Option<usize>,
    pub pending_open_binary_compare: Option<usize>,
    pub pending_history_record: Option<CompareMode>,

    pub left_encoding: String,
    pub right_encoding: String,
    pub left_save_encoding: String,
    pub right_save_encoding: String,

    pub left_preview_kind: SidePreviewKind,
    pub right_preview_kind: SidePreviewKind,
    pub left_binary_bytes: Vec<u8>,
    pub right_binary_bytes: Vec<u8>,
    pub left_folder_entries: Vec<String>,
    pub right_folder_entries: Vec<String>,
    /// Set during UI render so drag-and-drop targets the hovered pane.
    pub drop_target_left: bool,
    pub drop_target_right: bool,
    /// Accumulated screen rects for left/right drop targets (path row + content).
    pub left_drop_rect: Option<egui::Rect>,
    pub right_drop_rect: Option<egui::Rect>,
    pub same_path_compare: bool,
}

impl CompareSession {
    pub fn new_empty(id: u64) -> Self {
        Self {
            id,
            viewport_id: egui::ViewportId::from_hash_of(("rustpad_compare", id)),
            open: true,
            compare_intent: CompareIntent::File,
            left_path_text: String::new(),
            right_path_text: String::new(),
            left_path: PathBuf::new(),
            right_path: PathBuf::new(),
            mode: CompareMode::None,
            status_message: String::new(),
            font_size: 13.0,
            show_whitespace: false,
            show_diff_map: false,
            strict_mode: false,
            expand_unchanged: true,
            word_wrap: false,
            algorithm: DiffAlgorithm::default(),
            ignore_whitespace: false,
            ignore_case: false,
            text_result: None,
            left_text: String::new(),
            right_text: String::new(),
            left_dirty: false,
            right_dirty: false,
            text_current_change: 0,
            text_scroll_to_row: None,
            text_edit_pending_recompute: false,
            text_scroll_to_line: None,
            cmp_sync_scroll_y: 0.0,
            cmp_editing: None,
            edit_undo_stack: Vec::new(),
            folder_options: FolderDiffOptions::default(),
            folder_filter: FolderDiffFilter::default(),
            folder_result: None,
            folder_selected: None,
            folder_sync_message: String::new(),
            binary_result: None,
            binary_current_diff: 0,
            binary_scroll_to_row: None,
            pending_save_left: false,
            pending_save_right: false,
            pending_export: false,
            pending_next: false,
            pending_prev: false,
            pending_open_file_compare: None,
            pending_open_binary_compare: None,
            pending_history_record: None,
            left_encoding: "UTF-8".to_string(),
            right_encoding: "UTF-8".to_string(),
            left_save_encoding: "UTF-8".to_string(),
            right_save_encoding: "UTF-8".to_string(),
            left_preview_kind: SidePreviewKind::None,
            right_preview_kind: SidePreviewKind::None,
            left_binary_bytes: Vec::new(),
            right_binary_bytes: Vec::new(),
            left_folder_entries: Vec::new(),
            right_folder_entries: Vec::new(),
            drop_target_left: false,
            drop_target_right: false,
            left_drop_rect: None,
            right_drop_rect: None,
            same_path_compare: false,
        }
    }

    pub fn reset_drop_tracking(&mut self) {
        self.drop_target_left = false;
        self.drop_target_right = false;
        self.left_drop_rect = None;
        self.right_drop_rect = None;
    }

    pub fn register_drop_rect(&mut self, left: bool, rect: egui::Rect) {
        if left {
            self.left_drop_rect = Some(match self.left_drop_rect {
                Some(existing) => existing.union(rect),
                None => rect,
            });
        } else {
            self.right_drop_rect = Some(match self.right_drop_rect {
                Some(existing) => existing.union(rect),
                None => rect,
            });
        }
    }

    pub fn drop_side_at_pointer(&self, pointer: egui::Pos2) -> Option<bool> {
        let on_left = self.left_drop_rect.is_some_and(|r| r.contains(pointer));
        let on_right = self.right_drop_rect.is_some_and(|r| r.contains(pointer));
        match (on_left, on_right) {
            (true, false) => Some(false),
            (false, true) => Some(true),
            (true, true) => {
                let lx = self.left_drop_rect.map(|r| r.center().x).unwrap_or(0.0);
                let rx = self.right_drop_rect.map(|r| r.center().x).unwrap_or(0.0);
                Some(pointer.x >= (lx + rx) * 0.5)
            }
            (false, false) => None,
        }
    }

    pub fn set_drop_target_from_pointer(&mut self, pointer: Option<egui::Pos2>) {
        let Some(pointer) = pointer else { return };
        if let Some(to_right) = self.drop_side_at_pointer(pointer) {
            self.drop_target_left = !to_right;
            self.drop_target_right = to_right;
        }
    }

    pub fn has_side_preview(&self, left: bool) -> bool {
        if left {
            self.left_preview_kind != SidePreviewKind::None
        } else {
            self.right_preview_kind != SidePreviewKind::None
        }
    }

    pub fn has_any_preview(&self) -> bool {
        self.has_side_preview(true) || self.has_side_preview(false)
    }

    pub fn window_title(&self) -> String {
        match self.mode {
            CompareMode::Text => {
                if self.left_path.as_os_str().is_empty() || self.right_path.as_os_str().is_empty() {
                    return "Compare Files".to_string();
                }
                let left = file_label(&self.left_path);
                let right = file_label(&self.right_path);
                format!("Compare: {left} | {right}")
            }
            CompareMode::Binary => {
                if self.left_path.as_os_str().is_empty() || self.right_path.as_os_str().is_empty() {
                    return "Binary Compare".to_string();
                }
                let left = file_label(&self.left_path);
                let right = file_label(&self.right_path);
                format!("Binary: {left} | {right}")
            }
            CompareMode::Folder => {
                if self.left_path.as_os_str().is_empty() || self.right_path.as_os_str().is_empty() {
                    return "Compare Folders".to_string();
                }
                let left = file_label(&self.left_path);
                let right = file_label(&self.right_path);
                format!("Compare dirs: {left} | {right}")
            }
            CompareMode::None => "Compare".to_string(),
        }
    }

    pub fn set_left_path(&mut self, path: PathBuf) {
        self.left_path_text = path.display().to_string();
        self.refresh_display();
    }

    pub fn set_right_path(&mut self, path: PathBuf) {
        self.right_path_text = path.display().to_string();
        self.refresh_display();
    }

    pub fn clear_paths(&mut self) {
        self.left_path_text.clear();
        self.right_path_text.clear();
        self.left_path.clear();
        self.right_path.clear();
        self.mode = CompareMode::None;
        self.status_message.clear();
        self.text_result = None;
        self.folder_result = None;
        self.binary_result = None;
        self.left_text.clear();
        self.right_text.clear();
        self.edit_undo_stack.clear();
        self.same_path_compare = false;
        self.cmp_sync_scroll_y = 0.0;
        self.clear_preview_side(true);
        self.clear_preview_side(false);
    }

    fn clear_preview_side(&mut self, left: bool) {
        if left {
            self.left_preview_kind = SidePreviewKind::None;
            self.left_text.clear();
            self.left_binary_bytes.clear();
            self.left_folder_entries.clear();
        } else {
            self.right_preview_kind = SidePreviewKind::None;
            self.right_text.clear();
            self.right_binary_bytes.clear();
            self.right_folder_entries.clear();
        }
    }

    /// Commit draft paths and refresh compare / preview content.
    pub fn refresh_display(&mut self) {
        self.same_path_compare = false;
        self.left_path_text = self.left_path_text.trim().to_string();
        self.right_path_text = self.right_path_text.trim().to_string();

        let left = resolve_existing_path(&self.left_path_text);
        let right = resolve_existing_path(&self.right_path_text);

        if self.left_path_text.is_empty() {
            self.left_path.clear();
            self.clear_preview_side(true);
        } else if let Some(l) = &left {
            self.left_path = l.clone();
        }

        if self.right_path_text.is_empty() {
            self.right_path.clear();
            self.clear_preview_side(false);
        } else if let Some(r) = &right {
            self.right_path = r.clone();
        }

        if let (Some(l), Some(r)) = (&left, &right) {
            if let Err(msg) = self.validate_intent(l, r) {
                self.mode = CompareMode::None;
                self.text_result = None;
                self.binary_result = None;
                self.folder_result = None;
                self.status_message = msg.to_string();
                self.load_preview_for_side(true, l);
                self.load_preview_for_side(false, r);
                return;
            }
            self.left_path = l.clone();
            self.right_path = r.clone();
            match detect_mode(l, r) {
                Ok(_) => {
                    self.run_compare();
                    self.same_path_compare = paths_equal(l, r);
                    return;
                }
                Err(msg) => {
                    self.mode = CompareMode::None;
                    self.text_result = None;
                    self.binary_result = None;
                    self.folder_result = None;
                    self.status_message = msg.to_string();
                    self.load_preview_for_side(true, l);
                    self.load_preview_for_side(false, r);
                    return;
                }
            }
        }

        self.mode = CompareMode::None;
        self.text_result = None;
        self.binary_result = None;
        self.folder_result = None;
        self.status_message.clear();

        if let Some(l) = &left {
            self.left_path = l.clone();
            self.load_preview_for_side(true, l);
        }
        if let Some(r) = &right {
            self.right_path = r.clone();
            self.load_preview_for_side(false, r);
        }
    }

    fn validate_intent(&self, left: &Path, right: &Path) -> Result<(), &'static str> {
        match self.compare_intent {
            CompareIntent::File => {
                if left.is_dir() || right.is_dir() {
                    Err("File compare requires files on both sides")
                } else {
                    Ok(())
                }
            }
            CompareIntent::Folder => {
                if left.is_file() || right.is_file() {
                    Err("Folder compare requires directories on both sides")
                } else {
                    Ok(())
                }
            }
        }
    }

    fn load_preview_for_side(&mut self, left: bool, path: &Path) {
        let kind = preview_kind_for_path(path);
        let path_matches = if left {
            paths_equal(&self.left_path, path) && self.left_preview_kind == kind
        } else {
            paths_equal(&self.right_path, path) && self.right_preview_kind == kind
        };
        if path_matches && self.side_preview_loaded(left, kind) {
            return;
        }

        if left {
            self.left_preview_kind = kind;
            self.left_path = path.to_path_buf();
        } else {
            self.right_preview_kind = kind;
            self.right_path = path.to_path_buf();
        }

        match kind {
            SidePreviewKind::None => self.clear_preview_side(left),
            SidePreviewKind::Text => {
                let text = std::fs::read_to_string(path).unwrap_or_default();
                if left {
                    self.left_text = text;
                } else {
                    self.right_text = text;
                }
            }
            SidePreviewKind::Binary => {
                let bytes = read_binary_preview(path);
                if left {
                    self.left_binary_bytes = bytes;
                } else {
                    self.right_binary_bytes = bytes;
                }
            }
            SidePreviewKind::Folder => {
                let entries = list_folder_entries(path);
                if left {
                    self.left_folder_entries = entries;
                } else {
                    self.right_folder_entries = entries;
                }
            }
        }
    }

    fn side_preview_loaded(&self, left: bool, kind: SidePreviewKind) -> bool {
        match kind {
            SidePreviewKind::None => false,
            SidePreviewKind::Text => true,
            SidePreviewKind::Binary => {
                if left {
                    !self.left_binary_bytes.is_empty()
                } else {
                    !self.right_binary_bytes.is_empty()
                }
            }
            SidePreviewKind::Folder => {
                if left {
                    !self.left_folder_entries.is_empty()
                } else {
                    !self.right_folder_entries.is_empty()
                }
            }
        }
    }

    pub fn swap_sides(&mut self) {
        std::mem::swap(&mut self.left_path_text, &mut self.right_path_text);
        std::mem::swap(&mut self.left_path, &mut self.right_path);
        std::mem::swap(&mut self.left_text, &mut self.right_text);
        std::mem::swap(&mut self.left_dirty, &mut self.right_dirty);
        std::mem::swap(&mut self.left_encoding, &mut self.right_encoding);
        std::mem::swap(&mut self.left_save_encoding, &mut self.right_save_encoding);
        std::mem::swap(&mut self.left_preview_kind, &mut self.right_preview_kind);
        std::mem::swap(&mut self.left_binary_bytes, &mut self.right_binary_bytes);
        std::mem::swap(&mut self.left_folder_entries, &mut self.right_folder_entries);
        self.refresh_display();
    }

    pub fn commit_paths_from_text(&mut self) -> bool {
        self.left_path_text = self.left_path_text.trim().to_string();
        self.right_path_text = self.right_path_text.trim().to_string();
        let Some(left) = resolve_existing_path(&self.left_path_text) else {
            self.status_message = "Left path is missing or invalid".to_string();
            return false;
        };
        let Some(right) = resolve_existing_path(&self.right_path_text) else {
            self.status_message = "Right path is missing or invalid".to_string();
            return false;
        };
        self.left_path = left;
        self.right_path = right;
        true
    }

    pub fn run_compare(&mut self) {
        if !self.commit_paths_from_text() {
            self.mode = CompareMode::None;
            return;
        }
        match detect_mode(&self.left_path, &self.right_path) {
            Ok(mode) => {
                self.mode = mode;
                self.status_message.clear();
                match mode {
                    CompareMode::Text => self.run_text_compare(),
                    CompareMode::Binary => self.run_binary_compare(),
                    CompareMode::Folder => self.run_folder_compare(),
                    CompareMode::None => {}
                }
                if self.compare_succeeded(mode) {
                    self.pending_history_record = Some(mode);
                }
            }
            Err(msg) => {
                self.mode = CompareMode::None;
                self.status_message = msg.to_string();
                self.text_result = None;
                self.folder_result = None;
                self.binary_result = None;
            }
        }
    }

    fn run_text_compare(&mut self) {
        match (
            std::fs::read_to_string(&self.left_path),
            std::fs::read_to_string(&self.right_path),
        ) {
            (Ok(left_text), Ok(right_text)) => {
                self.left_text = left_text;
                self.right_text = right_text;
                self.left_dirty = false;
                self.right_dirty = false;
                self.edit_undo_stack.clear();
                self.cmp_sync_scroll_y = 0.0;
                self.recompute_text();
            }
            _ => {
                self.status_message = "Failed to read text files".to_string();
                self.text_result = None;
            }
        }
    }

    fn run_binary_compare(&mut self) {
        match compare_binary_files(&self.left_path, &self.right_path) {
            Ok(result) => {
                if self.binary_current_diff >= result.diff_count() {
                    self.binary_current_diff = 0;
                }
                self.binary_result = Some(result);
            }
            Err(e) => {
                self.status_message = format!("Binary compare failed: {e}");
                self.binary_result = None;
            }
        }
    }

    pub fn run_folder_compare(&mut self) {
        let engine = FolderDiff::with_options(self.folder_options.clone());
        match engine.diff_folders(&self.left_path, &self.right_path) {
            Ok(result) => {
                self.folder_result = Some(result);
                self.folder_selected = None;
            }
            Err(e) => {
                self.status_message = format!("Folder compare failed: {e}");
                self.folder_result = None;
            }
        }
    }

    fn compare_succeeded(&self, mode: CompareMode) -> bool {
        match mode {
            CompareMode::Text => self.text_result.is_some(),
            CompareMode::Binary => self.binary_result.is_some(),
            CompareMode::Folder => self.folder_result.is_some(),
            CompareMode::None => false,
        }
    }

    pub fn recompute_text(&mut self) {
        if self.mode != CompareMode::Text {
            return;
        }
        let ignore_whitespace = if self.strict_mode {
            false
        } else {
            self.ignore_whitespace
        };
        let ignore_case = if self.strict_mode {
            false
        } else {
            self.ignore_case
        };
        let engine = DiffEngine::new()
            .with_algorithm(self.algorithm)
            .with_options(DiffOptions {
                ignore_whitespace,
                ignore_case,
                ignore_line_endings: !self.strict_mode,
            });
        let result = engine.diff(&self.left_text, &self.right_text);
        if self.text_current_change >= result.change_count() {
            self.text_current_change = 0;
        }
        self.text_result = Some(result);
    }

    pub fn push_edit_undo_snapshot(&mut self, left: &str, right: &str) {
        let snap = (left.to_string(), right.to_string());
        if self.edit_undo_stack.last() != Some(&snap) {
            self.edit_undo_stack.push(snap);
            const MAX_UNDO: usize = 64;
            if self.edit_undo_stack.len() > MAX_UNDO {
                self.edit_undo_stack.remove(0);
            }
        }
    }

    pub fn can_undo_edit(&self) -> bool {
        !self.edit_undo_stack.is_empty()
    }

    pub fn undo_text_edit(&mut self) {
        let Some((left, right)) = self.edit_undo_stack.pop() else {
            return;
        };
        self.left_text = left;
        self.right_text = right;
        self.left_dirty = true;
        self.right_dirty = true;
        self.text_edit_pending_recompute = true;
    }

    pub fn toggle_strict_mode(&mut self) {
        self.strict_mode = !self.strict_mode;
        if self.strict_mode {
            self.ignore_case = false;
            self.ignore_whitespace = false;
        }
        self.recompute_active();
    }

    pub fn toggle_diff_map(&mut self) {
        self.show_diff_map = !self.show_diff_map;
    }

    pub fn algorithm_label(&self) -> &'static str {
        match self.algorithm {
            DiffAlgorithm::Myers => "Myers",
            DiffAlgorithm::Patience => "Patience",
            DiffAlgorithm::Lcs => "LCS",
        }
    }

    pub fn recompute_active(&mut self) {
        match self.mode {
            CompareMode::Text => self.recompute_text(),
            CompareMode::Binary => self.run_binary_compare(),
            CompareMode::Folder => self.run_folder_compare(),
            CompareMode::None => {}
        }
    }

    pub fn handle_keys(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F8) {
                self.pending_next = true;
            }
            if i.key_pressed(egui::Key::F7) {
                self.pending_prev = true;
            }
            if i.key_pressed(egui::Key::F5) {
                self.run_compare();
            }
        });
        if self.pending_next {
            self.pending_next = false;
            self.next_change();
        }
        if self.pending_prev {
            self.pending_prev = false;
            self.prev_change();
        }
    }

    pub fn next_change(&mut self) {
        match self.mode {
            CompareMode::Text => {
                let Some(result) = &self.text_result else { return };
                let count = result.change_count();
                if count == 0 {
                    return;
                }
                self.text_current_change = (self.text_current_change + 1) % count;
                self.scroll_to_current_change();
            }
            CompareMode::Binary => {
                let Some(result) = &self.binary_result else { return };
                let count = result.diff_count();
                if count == 0 {
                    return;
                }
                self.binary_current_diff = (self.binary_current_diff + 1) % count;
                let offset = result.diff_offsets[self.binary_current_diff];
                self.binary_scroll_to_row = Some((offset / 16) as usize);
            }
            _ => {}
        }
    }

    pub fn prev_change(&mut self) {
        match self.mode {
            CompareMode::Text => {
                let Some(result) = &self.text_result else { return };
                let count = result.change_count();
                if count == 0 {
                    return;
                }
                self.text_current_change =
                    (self.text_current_change + count - 1) % count;
                self.scroll_to_current_change();
            }
            CompareMode::Binary => {
                let Some(result) = &self.binary_result else { return };
                let count = result.diff_count();
                if count == 0 {
                    return;
                }
                self.binary_current_diff =
                    (self.binary_current_diff + count - 1) % count;
                let offset = result.diff_offsets[self.binary_current_diff];
                self.binary_scroll_to_row = Some((offset / 16) as usize);
            }
            _ => {}
        }
    }

    fn scroll_to_current_change(&mut self) {
        let Some(result) = self.text_result.as_ref() else { return };
        self.text_scroll_to_row = result.change_starts.get(self.text_current_change).copied();
        if let Some(row_idx) = self.text_scroll_to_row {
            if let Some(row) = result.rows.get(row_idx) {
                if let Some(line) = row.left_line.or(row.right_line) {
                    let line_height = self.font_size + 5.0;
                    self.apply_scroll_to_line(line, line_height);
                }
            }
        }
    }

    pub fn apply_scroll_to_line(&mut self, line: usize, line_height: f32) {
        self.cmp_sync_scroll_y = line as f32 * line_height;
    }

    pub fn merge_to_right(&mut self, change_id: usize) {
        let Some((left_range, right_range)) = self.text_change_block_ranges(change_id) else { return };
        let left_lines = split_lines(&self.left_text);
        let mut right_lines = split_lines(&self.right_text);
        let replacement: Vec<String> = left_lines[left_range].to_vec();
        let end = right_range.end.min(right_lines.len());
        let start = right_range.start.min(end);
        right_lines.splice(start..end, replacement);
        self.right_text = right_lines.join("\n");
        self.right_dirty = true;
        self.recompute_text();
    }

    pub fn merge_to_left(&mut self, change_id: usize) {
        let Some((left_range, right_range)) = self.text_change_block_ranges(change_id) else { return };
        let right_lines = split_lines(&self.right_text);
        let mut left_lines = split_lines(&self.left_text);
        let replacement: Vec<String> = right_lines[right_range].to_vec();
        let end = left_range.end.min(left_lines.len());
        let start = left_range.start.min(end);
        left_lines.splice(start..end, replacement);
        self.left_text = left_lines.join("\n");
        self.left_dirty = true;
        self.recompute_text();
    }

    fn text_change_block_ranges(&self, change_id: usize) -> Option<LineRangePair> {
        let result = self.text_result.as_ref()?;
        let rows: Vec<_> = result
            .rows
            .iter()
            .filter(|r| r.change_id == Some(change_id))
            .collect();
        if rows.is_empty() {
            return None;
        }
        let left_lines: Vec<usize> = rows.iter().filter_map(|r| r.left_line).collect();
        let right_lines: Vec<usize> = rows.iter().filter_map(|r| r.right_line).collect();
        let left_range = if let (Some(&s), Some(&e)) = (left_lines.iter().min(), left_lines.iter().max()) {
            s..e + 1
        } else {
            let at = rows[0].left_at;
            at..at
        };
        let right_range = if let (Some(&s), Some(&e)) = (right_lines.iter().min(), right_lines.iter().max()) {
            s..e + 1
        } else {
            let at = rows[0].right_at;
            at..at
        };
        Some((left_range, right_range))
    }

    pub fn export_html(&self) -> Option<String> {
        let result = self.text_result.as_ref()?;
        let mut html = String::from("<!DOCTYPE html><html><head><style>");
        html.push_str("body { font-family: monospace; }");
        html.push_str(".equal { background: #fff; }");
        html.push_str(".insert { background: #e4ffe4; }");
        html.push_str(".delete { background: #ffe4e4; }");
        html.push_str(".replace { background: #fffbe4; }");
        html.push_str("</style></head><body><h1>Diff Report</h1><table>");
        for line in &result.lines {
            let class = match line.tag {
                crate::diff::DiffTag::Equal => "equal",
                crate::diff::DiffTag::Insert => "insert",
                crate::diff::DiffTag::Delete => "delete",
                crate::diff::DiffTag::Replace => "replace",
            };
            html.push_str(&format!(
                "<tr class=\"{class}\"><td>{}</td></tr>",
                crate::editor::context_actions::html_escape(&line.content),
            ));
        }
        html.push_str("</table></body></html>");
        Some(html)
    }

    pub fn folder_batch_sync(&mut self, direction: SyncDirection, scope: SyncScope) {
        let Some(result) = self.folder_result.clone() else { return };
        let report = sync_folder(&result, direction, scope);
        self.folder_sync_message = if report.ok() {
            format!("Synced {} file(s)", report.copied)
        } else {
            format!(
                "Synced {} file(s), {} error(s)",
                report.copied,
                report.errors.len()
            )
        };
        self.run_folder_compare();
    }

    pub fn folder_copy_to_right(&mut self, entry_index: usize) {
        let Some(result) = self.folder_result.clone() else { return };
        let Some(entry) = result.entries.get(entry_index) else { return };
        if let (Some(src), Some(dest)) = (&entry.left_path, &entry.right_path) {
            let _ = copy_path(src, dest);
        } else if let Some(src) = &entry.left_path {
            let dest = result.right_root.join(&entry.relative_path);
            let _ = copy_path(src, &dest);
        }
        self.run_folder_compare();
    }

    pub fn folder_copy_to_left(&mut self, entry_index: usize) {
        let Some(result) = self.folder_result.clone() else { return };
        let Some(entry) = result.entries.get(entry_index) else { return };
        if let (Some(src), Some(dest)) = (&entry.right_path, &entry.left_path) {
            let _ = copy_path(src, dest);
        } else if let Some(src) = &entry.right_path {
            let dest = result.left_root.join(&entry.relative_path);
            let _ = copy_path(src, &dest);
        }
        self.run_folder_compare();
    }

    pub fn apply_dropped_path(&mut self, path: PathBuf, to_right: bool) {
        if !(path.is_dir() || path.is_file()) {
            return;
        }
        self.pending_history_record = None;
        self.text_result = None;
        self.binary_result = None;
        self.folder_result = None;
        if to_right {
            self.clear_preview_side(false);
            self.right_path_text = path.display().to_string();
        } else {
            self.clear_preview_side(true);
            self.left_path_text = path.display().to_string();
        }
        self.refresh_display();
    }
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn preview_kind_for_path(path: &Path) -> SidePreviewKind {
    if !path.exists() {
        return SidePreviewKind::None;
    }
    if path.is_dir() {
        SidePreviewKind::Folder
    } else if is_likely_binary(path) {
        SidePreviewKind::Binary
    } else {
        SidePreviewKind::Text
    }
}

const BINARY_PREVIEW_MAX: usize = 10 * 1024 * 1024;

fn read_binary_preview(path: &Path) -> Vec<u8> {
    let Ok(meta) = std::fs::metadata(path) else {
        return Vec::new();
    };
    let len = meta.len().min(BINARY_PREVIEW_MAX as u64) as usize;
    let Ok(mut file) = std::fs::File::open(path) else {
        return Vec::new();
    };
    use std::io::Read;
    let mut buf = vec![0u8; len];
    if file.read_exact(&mut buf).is_err() {
        buf.clear();
        let _ = file.read_to_end(&mut buf);
        if buf.len() > BINARY_PREVIEW_MAX {
            buf.truncate(BINARY_PREVIEW_MAX);
        }
    }
    buf
}

fn list_folder_entries(path: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(path)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.path().is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect();
    names.sort_by_key(|a| a.to_lowercase());
    names
}

fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn resolve_existing_path(text: &str) -> Option<PathBuf> {
    if text.is_empty() {
        return None;
    }
    let path = PathBuf::from(text);
    if path.exists() {
        Some(path.canonicalize().unwrap_or(path))
    } else {
        None
    }
}

fn detect_mode(left: &Path, right: &Path) -> Result<CompareMode, &'static str> {
    if !left.exists() || !right.exists() {
        return Err("One or both paths do not exist");
    }
    let left_dir = left.is_dir();
    let right_dir = right.is_dir();
    if left_dir != right_dir {
        return Err("Both sides must be files or both must be folders");
    }
    if left_dir {
        return Ok(CompareMode::Folder);
    }
    if is_likely_binary(left) || is_likely_binary(right) {
        Ok(CompareMode::Binary)
    } else {
        Ok(CompareMode::Text)
    }
}

fn split_lines(text: &str) -> Vec<String> {
    text.lines().map(|l| l.to_string()).collect()
}

fn copy_path(src: &Path, dest: &Path) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(src, dest)?;
    Ok(())
}
