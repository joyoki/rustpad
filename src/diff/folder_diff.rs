use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// File comparison status (notepad-- / Beyond Compare style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    Identical,
    Different,
    LeftOnly,
    RightOnly,
}

/// Quick mode compares size + mtime; deep mode reads file bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FolderCompareMode {
    #[default]
    Quick,
    Deep,
}

/// Result list filter (notepad--: all / diff / only diff / unique).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FolderDiffFilter {
    #[default]
    All,
    /// Different + left-only + right-only (hide identical).
    Diff,
    /// Only files present on both sides but not equal.
    DifferentOnly,
    /// Only left-only or right-only entries.
    UniqueOnly,
    Identical,
}

impl FolderDiffFilter {
    pub fn matches(self, status: FileStatus) -> bool {
        match self {
            Self::All => true,
            Self::Diff => status != FileStatus::Identical,
            Self::DifferentOnly => status == FileStatus::Different,
            Self::UniqueOnly => matches!(status, FileStatus::LeftOnly | FileStatus::RightOnly),
            Self::Identical => status == FileStatus::Identical,
        }
    }
}

/// Options controlling directory traversal and comparison (notepad-- DirCmpExtWin).
#[derive(Debug, Clone)]
pub struct FolderDiffOptions {
    pub mode: FolderCompareMode,
    /// Include hidden files and directories (names starting with `.`).
    pub compare_hidden: bool,
    /// Directory name segments to skip entirely (e.g. `.git`, `target`).
    pub skip_dir_names: Vec<String>,
    /// File extensions to skip (e.g. `.sln`, `.vcxproj`).
    pub skip_extensions: Vec<String>,
}

impl Default for FolderDiffOptions {
    fn default() -> Self {
        Self {
            mode: FolderCompareMode::Quick,
            compare_hidden: false,
            skip_dir_names: vec![
                ".git".into(),
                "node_modules".into(),
                "target".into(),
                ".svn".into(),
                ".vs".into(),
                "debug".into(),
                "Release".into(),
            ],
            skip_extensions: vec![".sln".into(), ".vcxproj".into()],
        }
    }
}

/// A file entry in the folder diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderDiffEntry {
    pub relative_path: String,
    pub left_path: Option<PathBuf>,
    pub right_path: Option<PathBuf>,
    pub status: FileStatus,
    pub left_size: u64,
    pub right_size: u64,
    pub left_mtime: Option<SystemTime>,
    pub right_mtime: Option<SystemTime>,
}

/// Result of folder comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderDiffResult {
    pub left_root: PathBuf,
    pub right_root: PathBuf,
    pub entries: Vec<FolderDiffEntry>,
    pub stats: FolderDiffStats,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FolderDiffStats {
    pub identical: usize,
    pub different: usize,
    pub left_only: usize,
    pub right_only: usize,
}

/// Folder diff engine.
pub struct FolderDiff {
    options: FolderDiffOptions,
}

impl FolderDiff {
    pub fn new() -> Self {
        Self {
            options: FolderDiffOptions::default(),
        }
    }

    pub fn with_options(options: FolderDiffOptions) -> Self {
        Self { options }
    }

    pub fn options(&self) -> &FolderDiffOptions {
        &self.options
    }

    pub fn options_mut(&mut self) -> &mut FolderDiffOptions {
        &mut self.options
    }

    pub fn diff_folders(
        &self,
        left: &PathBuf,
        right: &PathBuf,
    ) -> anyhow::Result<FolderDiffResult> {
        let left_files = self.collect_files(left)?;
        let right_files = self.collect_files(right)?;

        let left_map: HashMap<String, PathBuf> = left_files.into_iter().collect();
        let right_map: HashMap<String, PathBuf> = right_files.into_iter().collect();

        let mut entries = Vec::new();
        let mut stats = FolderDiffStats::default();

        let mut all_paths: Vec<String> = left_map.keys().chain(right_map.keys()).cloned().collect();
        all_paths.sort();
        all_paths.dedup();

        for rel_path in all_paths {
            let left_path = left_map.get(&rel_path);
            let right_path = right_map.get(&rel_path);

            match (left_path, right_path) {
                (Some(lp), Some(rp)) => {
                    let left_meta = std::fs::metadata(lp).ok();
                    let right_meta = std::fs::metadata(rp).ok();
                    let left_size = left_meta.as_ref().map(|m| m.len()).unwrap_or(0);
                    let right_size = right_meta.as_ref().map(|m| m.len()).unwrap_or(0);
                    let left_mtime = left_meta.and_then(|m| m.modified().ok());
                    let right_mtime = right_meta.and_then(|m| m.modified().ok());

                    let status = self.compare_pair(lp, rp, left_mtime, right_mtime, left_size, right_size);
                    match status {
                        FileStatus::Identical => stats.identical += 1,
                        FileStatus::Different => stats.different += 1,
                        _ => {}
                    }

                    entries.push(FolderDiffEntry {
                        relative_path: rel_path,
                        left_path: Some(lp.clone()),
                        right_path: Some(rp.clone()),
                        status,
                        left_size,
                        right_size,
                        left_mtime,
                        right_mtime,
                    });
                }
                (Some(lp), None) => {
                    stats.left_only += 1;
                    let meta = std::fs::metadata(lp).ok();
                    entries.push(FolderDiffEntry {
                        relative_path: rel_path,
                        left_path: Some(lp.clone()),
                        right_path: None,
                        status: FileStatus::LeftOnly,
                        left_size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                        right_size: 0,
                        left_mtime: meta.and_then(|m| m.modified().ok()),
                        right_mtime: None,
                    });
                }
                (None, Some(rp)) => {
                    stats.right_only += 1;
                    let meta = std::fs::metadata(rp).ok();
                    entries.push(FolderDiffEntry {
                        relative_path: rel_path,
                        left_path: None,
                        right_path: Some(rp.clone()),
                        status: FileStatus::RightOnly,
                        left_size: 0,
                        right_size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                        left_mtime: None,
                        right_mtime: meta.and_then(|m| m.modified().ok()),
                    });
                }
                (None, None) => {}
            }
        }

        Ok(FolderDiffResult {
            left_root: left.clone(),
            right_root: right.clone(),
            entries,
            stats,
        })
    }

    fn compare_pair(
        &self,
        left: &Path,
        right: &Path,
        left_mtime: Option<SystemTime>,
        right_mtime: Option<SystemTime>,
        left_size: u64,
        right_size: u64,
    ) -> FileStatus {
        match self.options.mode {
            FolderCompareMode::Quick => {
                if left_size == right_size && left_mtime == right_mtime {
                    FileStatus::Identical
                } else {
                    FileStatus::Different
                }
            }
            FolderCompareMode::Deep => {
                if self.files_equal_bytes(left, right) {
                    FileStatus::Identical
                } else {
                    FileStatus::Different
                }
            }
        }
    }

    fn collect_files(&self, dir: &PathBuf) -> anyhow::Result<Vec<(String, PathBuf)>> {
        let mut files = Vec::new();
        self.collect_files_recursive(dir, dir, &mut files)?;
        Ok(files)
    }

    fn collect_files_recursive(
        &self,
        base: &PathBuf,
        current: &PathBuf,
        files: &mut Vec<(String, PathBuf)>,
    ) -> anyhow::Result<()> {
        if !current.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if !self.options.compare_hidden && name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                if self.should_skip_dir(&name) {
                    continue;
                }
                self.collect_files_recursive(base, &path, files)?;
            } else if !self.should_skip_file(&name) {
                let relative = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((relative, path));
            }
        }

        Ok(())
    }

    fn should_skip_dir(&self, name: &str) -> bool {
        self.options.skip_dir_names.iter().any(|p| p == name)
    }

    fn should_skip_file(&self, name: &str) -> bool {
        if name == ".DS_Store" {
            return true;
        }
        let lower = name.to_ascii_lowercase();
        self.options
            .skip_extensions
            .iter()
            .any(|ext| lower.ends_with(&ext.to_ascii_lowercase()))
    }

    fn files_equal_bytes(&self, left: &Path, right: &Path) -> bool {
        match (std::fs::read(left), std::fs::read(right)) {
            (Ok(l), Ok(r)) => l == r,
            _ => false,
        }
    }
}

impl Default for FolderDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Format file size for the folder compare table.
pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Format modification time for display (HH:MM UTC offset from epoch day).
pub fn format_mtime(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return "—".to_string();
    };
    let Ok(duration) = time.duration_since(std::time::UNIX_EPOCH) else {
        return "?".to_string();
    };
    let secs = duration.as_secs();
    let day_secs = secs % 86_400;
    let hours = day_secs / 3600;
    let mins = (day_secs % 3600) / 60;
    format!("{hours:02}:{mins:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_quick_mode_size_mtime() {
        let left = tempdir().unwrap();
        let right = tempdir().unwrap();
        let src = left.path().join("a.txt");
        fs::write(&src, "hello").unwrap();
        fs::copy(&src, right.path().join("a.txt")).unwrap();

        let engine = FolderDiff::new();
        let result = engine
            .diff_folders(&left.path().to_path_buf(), &right.path().to_path_buf())
            .unwrap();
        assert_eq!(result.stats.identical, 1);
    }

    #[test]
    fn test_deep_mode_content() {
        let left = tempdir().unwrap();
        let right = tempdir().unwrap();
        fs::write(left.path().join("a.txt"), "hello").unwrap();
        fs::write(right.path().join("a.txt"), "world").unwrap();

        let mut engine = FolderDiff::new();
        engine.options_mut().mode = FolderCompareMode::Deep;
        let result = engine
            .diff_folders(&left.path().to_path_buf(), &right.path().to_path_buf())
            .unwrap();
        assert_eq!(result.stats.different, 1);
    }

    #[test]
    fn test_skip_dir_and_unique() {
        let left = tempdir().unwrap();
        let right = tempdir().unwrap();
        fs::create_dir_all(left.path().join("keep")).unwrap();
        fs::write(left.path().join("keep/a.txt"), "x").unwrap();
        fs::write(left.path().join("only_left.txt"), "x").unwrap();
        fs::create_dir_all(left.path().join(".git")).unwrap();
        fs::write(left.path().join(".git/config"), "x").unwrap();

        let engine = FolderDiff::new();
        let result = engine
            .diff_folders(&left.path().to_path_buf(), &right.path().to_path_buf())
            .unwrap();
        assert_eq!(result.stats.left_only, 2);
        assert!(result.entries.iter().all(|e| !e.relative_path.contains(".git")));
    }

    #[test]
    fn test_filter_matches() {
        assert!(FolderDiffFilter::Diff.matches(FileStatus::Different));
        assert!(!FolderDiffFilter::Diff.matches(FileStatus::Identical));
        assert!(FolderDiffFilter::UniqueOnly.matches(FileStatus::LeftOnly));
    }
}
