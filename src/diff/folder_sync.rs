//! Batch directory sync operations for folder compare results (notepad-- style).

use std::path::{Path, PathBuf};

use super::folder_diff::{FileStatus, FolderDiffResult};

/// Copy direction for batch sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    /// Copy from left tree to right tree.
    ToRight,
    /// Copy from right tree to left tree.
    ToLeft,
}

/// Which entries participate in a batch operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncScope {
    /// Files that exist on both sides but differ.
    DifferentOnly,
    /// Files only on the left → copy to right.
    LeftOnlyToRight,
    /// Files only on the right → copy to left.
    RightOnlyToLeft,
    /// Overwrite different + copy left-only (mirror left → right).
    MirrorLeftToRight,
    /// Overwrite different + copy right-only (mirror right → left).
    MirrorRightToLeft,
}

/// Outcome of a batch sync run.
#[derive(Debug, Clone, Default)]
pub struct FolderSyncReport {
    pub copied: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

impl FolderSyncReport {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Run a recursive batch sync over folder compare entries.
pub fn sync_folder(
    result: &FolderDiffResult,
    direction: SyncDirection,
    scope: SyncScope,
) -> FolderSyncReport {
    let mut report = FolderSyncReport::default();

    for entry in &result.entries {
        let (src, dest, should_run) = match (direction, scope, entry.status) {
            (SyncDirection::ToRight, SyncScope::DifferentOnly, FileStatus::Different) => {
                match (&entry.left_path, &entry.right_path) {
                    (Some(s), Some(_)) => (s.clone(), result.right_root.join(&entry.relative_path), true),
                    _ => (PathBuf::new(), PathBuf::new(), false),
                }
            }
            (SyncDirection::ToLeft, SyncScope::DifferentOnly, FileStatus::Different) => {
                match (&entry.right_path, &entry.left_path) {
                    (Some(s), Some(_)) => (s.clone(), result.left_root.join(&entry.relative_path), true),
                    _ => (PathBuf::new(), PathBuf::new(), false),
                }
            }
            (SyncDirection::ToRight, SyncScope::LeftOnlyToRight, FileStatus::LeftOnly) => {
                match &entry.left_path {
                    Some(s) => (s.clone(), result.right_root.join(&entry.relative_path), true),
                    None => (PathBuf::new(), PathBuf::new(), false),
                }
            }
            (SyncDirection::ToLeft, SyncScope::RightOnlyToLeft, FileStatus::RightOnly) => {
                match &entry.right_path {
                    Some(s) => (s.clone(), result.left_root.join(&entry.relative_path), true),
                    None => (PathBuf::new(), PathBuf::new(), false),
                }
            }
            (SyncDirection::ToRight, SyncScope::MirrorLeftToRight, FileStatus::Different) => {
                match &entry.left_path {
                    Some(s) => (s.clone(), result.right_root.join(&entry.relative_path), true),
                    None => (PathBuf::new(), PathBuf::new(), false),
                }
            }
            (SyncDirection::ToRight, SyncScope::MirrorLeftToRight, FileStatus::LeftOnly) => {
                match &entry.left_path {
                    Some(s) => (s.clone(), result.right_root.join(&entry.relative_path), true),
                    None => (PathBuf::new(), PathBuf::new(), false),
                }
            }
            (SyncDirection::ToLeft, SyncScope::MirrorRightToLeft, FileStatus::Different) => {
                match &entry.right_path {
                    Some(s) => (s.clone(), result.left_root.join(&entry.relative_path), true),
                    None => (PathBuf::new(), PathBuf::new(), false),
                }
            }
            (SyncDirection::ToLeft, SyncScope::MirrorRightToLeft, FileStatus::RightOnly) => {
                match &entry.right_path {
                    Some(s) => (s.clone(), result.left_root.join(&entry.relative_path), true),
                    None => (PathBuf::new(), PathBuf::new(), false),
                }
            }
            _ => (PathBuf::new(), PathBuf::new(), false),
        };

        if !should_run {
            continue;
        }

        match copy_file(&src, &dest) {
            Ok(()) => report.copied += 1,
            Err(e) => report.errors.push(format!(
                "{} → {}: {e}",
                src.display(),
                dest.display()
            )),
        }
    }

    report
}

/// Delete files that exist only on one side (optional cleanup after sync).
pub fn delete_unique(
    result: &FolderDiffResult,
    side: SyncDirection,
) -> FolderSyncReport {
    let mut report = FolderSyncReport::default();

    for entry in &result.entries {
        let path = match (side, entry.status) {
            (SyncDirection::ToRight, FileStatus::LeftOnly) => entry.left_path.clone(),
            (SyncDirection::ToLeft, FileStatus::RightOnly) => entry.right_path.clone(),
            _ => None,
        };
        let Some(path) = path else { continue };

        match std::fs::remove_file(&path) {
            Ok(()) => report.copied += 1,
            Err(e) => report.errors.push(format!("{}: {e}", path.display())),
        }
    }

    report
}

fn copy_file(src: &Path, dest: &Path) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(src, dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::folder_diff::FolderDiff;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_mirror_left_to_right() {
        let left = tempdir().unwrap();
        let right = tempdir().unwrap();
        fs::write(left.path().join("a.txt"), "left").unwrap();
        fs::write(left.path().join("only.txt"), "only").unwrap();
        fs::write(right.path().join("a.txt"), "right").unwrap();

        let engine = FolderDiff::new();
        let diff = engine
            .diff_folders(&left.path().to_path_buf(), &right.path().to_path_buf())
            .unwrap();

        let report = sync_folder(&diff, SyncDirection::ToRight, SyncScope::MirrorLeftToRight);
        assert_eq!(report.copied, 2);
        assert!(report.ok());
        assert_eq!(fs::read_to_string(right.path().join("a.txt")).unwrap(), "left");
        assert_eq!(fs::read_to_string(right.path().join("only.txt")).unwrap(), "only");
    }
}
