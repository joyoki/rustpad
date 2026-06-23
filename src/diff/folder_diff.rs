use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File comparison status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    Identical,
    Different,
    LeftOnly,
    RightOnly,
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
}

/// Result of folder comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderDiffResult {
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
    ignore_patterns: Vec<String>,
}

impl FolderDiff {
    pub fn new() -> Self {
        Self {
            ignore_patterns: vec![
                ".git".to_string(),
                ".DS_Store".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
            ],
        }
    }

    pub fn add_ignore_pattern(&mut self, pattern: String) {
        self.ignore_patterns.push(pattern);
    }

    pub fn diff_folders(&self, left: &PathBuf, right: &PathBuf) -> anyhow::Result<FolderDiffResult> {
        let mut entries = Vec::new();
        let mut stats = FolderDiffStats::default();

        let left_files = self.collect_files(left)?;
        let right_files = self.collect_files(right)?;

        let mut left_map: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
        let mut right_map: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();

        for (path, full_path) in &left_files {
            left_map.insert(path.clone(), full_path.clone());
        }
        for (path, full_path) in &right_files {
            right_map.insert(path.clone(), full_path.clone());
        }

        for (rel_path, left_path) in &left_map {
            if !right_map.contains_key(rel_path) {
                stats.left_only += 1;
                let left_size = std::fs::metadata(left_path).map(|m| m.len()).unwrap_or(0);
                entries.push(FolderDiffEntry {
                    relative_path: rel_path.clone(),
                    left_path: Some(left_path.clone()),
                    right_path: None,
                    status: FileStatus::LeftOnly,
                    left_size,
                    right_size: 0,
                });
            }
        }

        for (rel_path, right_path) in &right_map {
            if !left_map.contains_key(rel_path) {
                stats.right_only += 1;
                let right_size = std::fs::metadata(right_path).map(|m| m.len()).unwrap_or(0);
                entries.push(FolderDiffEntry {
                    relative_path: rel_path.clone(),
                    left_path: None,
                    right_path: Some(right_path.clone()),
                    status: FileStatus::RightOnly,
                    left_size: 0,
                    right_size,
                });
            }
        }

        for (rel_path, left_path) in &left_map {
            if let Some(right_path) = right_map.get(rel_path) {
                let left_size = std::fs::metadata(left_path).map(|m| m.len()).unwrap_or(0);
                let right_size = std::fs::metadata(right_path).map(|m| m.len()).unwrap_or(0);

                let status = if self.files_equal(left_path, right_path) {
                    stats.identical += 1;
                    FileStatus::Identical
                } else {
                    stats.different += 1;
                    FileStatus::Different
                };

                entries.push(FolderDiffEntry {
                    relative_path: rel_path.clone(),
                    left_path: Some(left_path.clone()),
                    right_path: Some(right_path.clone()),
                    status,
                    left_size,
                    right_size,
                });
            }
        }

        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

        Ok(FolderDiffResult { entries, stats })
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

            if self.should_ignore(&name) {
                continue;
            }

            if path.is_dir() {
                self.collect_files_recursive(base, &path, files)?;
            } else {
                let relative = path.strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                files.push((relative, path));
            }
        }

        Ok(())
    }

    fn should_ignore(&self, name: &str) -> bool {
        self.ignore_patterns.iter().any(|p| name.contains(p.as_str()))
    }

    fn files_equal(&self, left: &PathBuf, right: &PathBuf) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_diff_new() {
        let diff = FolderDiff::new();
        assert!(!diff.ignore_patterns.is_empty());
    }

    #[test]
    fn test_should_ignore() {
        let diff = FolderDiff::new();
        assert!(diff.should_ignore(".git"));
        assert!(diff.should_ignore("target"));
        assert!(!diff.should_ignore("src"));
    }
}
