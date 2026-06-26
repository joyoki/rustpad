//! Binary (hex) file comparison — notepad-- CompareHexWin style.

use std::path::{Path, PathBuf};

/// Maximum file size for in-memory binary compare (10 MB, same as notepad--).
pub const MAX_BINARY_COMPARE_SIZE: u64 = 10 * 1024 * 1024;

const BYTES_PER_ROW: usize = 16;

/// One byte cell on left or right side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryHexCell {
    pub left: Option<u8>,
    pub right: Option<u8>,
}

impl BinaryHexCell {
    pub fn differs(self) -> bool {
        self.left != self.right
    }
}

/// One row of hex dump (up to 16 bytes).
#[derive(Debug, Clone)]
pub struct BinaryHexRow {
    pub offset: u64,
    pub cells: Vec<BinaryHexCell>,
}

/// Complete binary comparison result.
#[derive(Debug, Clone)]
pub struct BinaryDiffResult {
    pub left_path: PathBuf,
    pub right_path: PathBuf,
    pub left_size: u64,
    pub right_size: u64,
    /// True when either file exceeds [`MAX_BINARY_COMPARE_SIZE`].
    pub truncated: bool,
    pub rows: Vec<BinaryHexRow>,
    /// Byte offsets where left/right differ (for prev/next navigation).
    pub diff_offsets: Vec<u64>,
    pub identical_bytes: u64,
    pub different_bytes: u64,
}

impl BinaryDiffResult {
    pub fn diff_count(&self) -> usize {
        self.diff_offsets.len()
    }
}

/// Compare two files byte-by-byte and build aligned hex rows.
pub fn compare_binary_files(left: &Path, right: &Path) -> std::io::Result<BinaryDiffResult> {
    let left_meta = std::fs::metadata(left)?;
    let right_meta = std::fs::metadata(right)?;
    let left_size = left_meta.len();
    let right_size = right_meta.len();

    let truncated = left_size > MAX_BINARY_COMPARE_SIZE || right_size > MAX_BINARY_COMPARE_SIZE;
    let read_left = left_size.min(MAX_BINARY_COMPARE_SIZE) as usize;
    let read_right = right_size.min(MAX_BINARY_COMPARE_SIZE) as usize;
    let compare_len = read_left.max(read_right);

    let left_data = read_prefix(left, read_left)?;
    let right_data = read_prefix(right, read_right)?;

    let mut rows = Vec::new();
    let mut diff_offsets = Vec::new();
    let mut identical_bytes = 0u64;
    let mut different_bytes = 0u64;

    let mut offset = 0u64;
    while (offset as usize) < compare_len {
        let mut cells = Vec::with_capacity(BYTES_PER_ROW);
        for i in 0..BYTES_PER_ROW {
            let idx = offset as usize + i;
            if idx >= compare_len {
                break;
            }
            let l = left_data.get(idx).copied();
            let r = right_data.get(idx).copied();
            if l == r {
                if l.is_some() {
                    identical_bytes += 1;
                }
            } else {
                different_bytes += 1;
                diff_offsets.push(offset + i as u64);
            }
            cells.push(BinaryHexCell { left: l, right: r });
        }
        rows.push(BinaryHexRow { offset, cells });
        offset += BYTES_PER_ROW as u64;
    }

    Ok(BinaryDiffResult {
        left_path: left.to_path_buf(),
        right_path: right.to_path_buf(),
        left_size,
        right_size,
        truncated,
        rows,
        diff_offsets,
        identical_bytes,
        different_bytes,
    })
}

fn read_prefix(path: &Path, len: usize) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut buf = vec![0u8; len];
    if len > 0 {
        file.read_exact(&mut buf)?;
    }
    Ok(buf)
}

/// Heuristic: NUL byte or high ratio of non-text control chars → treat as binary.
pub fn is_likely_binary(path: &Path) -> bool {
    const SAMPLE: usize = 8192;
    let Ok(data) = std::fs::read(path) else {
        return true;
    };
    if data.is_empty() {
        return false;
    }
    if data.contains(&0) {
        return true;
    }
    let sample = &data[..data.len().min(SAMPLE)];
    let non_text = sample
        .iter()
        .filter(|&&b| b < 0x09 || (b > 0x0d && b < 0x20))
        .count();
    non_text * 10 > sample.len()
}

/// Format one byte as two hex digits, or `--` if missing.
pub fn format_hex_byte(b: Option<u8>) -> String {
    match b {
        Some(v) => format!("{v:02X}"),
        None => "--".to_string(),
    }
}

/// Printable ASCII or `.` for non-printable.
pub fn format_ascii_byte(b: Option<u8>) -> char {
    match b {
        Some(v) if (0x20..=0x7e).contains(&v) => v as char,
        Some(_) => '.',
        None => ' ',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_compare_binary_identical() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        fs::write(&a, [0x48, 0x65, 0x6c, 0x6c, 0x6f]).unwrap();
        fs::copy(&a, &b).unwrap();

        let result = compare_binary_files(&a, &b).unwrap();
        assert_eq!(result.different_bytes, 0);
        assert_eq!(result.identical_bytes, 5);
    }

    #[test]
    fn test_compare_binary_diff() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        fs::write(&a, [0x01, 0x02, 0x03]).unwrap();
        fs::write(&b, [0x01, 0xFF, 0x03]).unwrap();

        let result = compare_binary_files(&a, &b).unwrap();
        assert_eq!(result.different_bytes, 1);
        assert_eq!(result.diff_offsets, vec![1]);
    }

    #[test]
    fn test_is_likely_binary_nul() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("x.bin");
        fs::write(&p, [0x00, 0x01]).unwrap();
        assert!(is_likely_binary(&p));
    }
}
