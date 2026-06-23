use std::path::PathBuf;

pub mod autocomplete;
pub mod buffer;
pub mod cursor;
pub mod file_io;
pub mod indent;
pub mod macro_recorder;
pub mod tab;
pub mod view;

pub use buffer::TextBuffer;
pub use cursor::{Cursor, Selection};
pub use tab::TabManager;

/// Represents the encoding of a file.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FileEncoding {
    #[default]
    Utf8,
    Utf16Le,
    Utf16Be,
    Latin1,
}

/// Represents a single undo/redo action (Command pattern).
#[derive(Debug, Clone)]
pub enum EditAction {
    Insert {
        char_pos: usize,
        text: String,
    },
    Delete {
        char_pos: usize,
        text: String,
    },
    Replace {
        char_pos: usize,
        old_text: String,
        new_text: String,
    },
}

/// The line ending style used by a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
    Cr,
}

impl Default for LineEnding {
    fn default() -> Self {
        if cfg!(windows) {
            Self::CrLf
        } else {
            Self::Lf
        }
    }
}

#[allow(dead_code)]
impl LineEnding {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
            Self::Cr => "\r",
        }
    }
}

/// Detect line ending style from text content.
pub fn detect_line_ending(text: &str) -> LineEnding {
    if text.contains("\r\n") {
        LineEnding::CrLf
    } else if text.contains('\r') {
        LineEnding::Cr
    } else {
        LineEnding::Lf
    }
}

/// Detect file encoding from raw bytes.
pub fn detect_encoding(bytes: &[u8]) -> FileEncoding {
    if bytes.len() >= 3 && bytes[..3] == [0xEF, 0xBB, 0xBF] {
        return FileEncoding::Utf8;
    }
    if bytes.len() >= 2 && bytes[..2] == [0xFF, 0xFE] {
        return FileEncoding::Utf16Le;
    }
    if bytes.len() >= 2 && bytes[..2] == [0xFE, 0xFF] {
        return FileEncoding::Utf16Be;
    }
    if std::str::from_utf8(bytes).is_ok() {
        return FileEncoding::Utf8;
    }
    FileEncoding::Latin1
}

/// Read a file into a UTF-8 string, handling various encodings.
pub fn read_file_to_string(path: &PathBuf) -> anyhow::Result<(String, FileEncoding, LineEnding)> {
    let bytes = std::fs::read(path)?;
    let encoding = detect_encoding(&bytes);
    let text = match encoding {
        FileEncoding::Utf8 => {
            let start = if bytes.len() >= 3 && bytes[..3] == [0xEF, 0xBB, 0xBF] {
                3
            } else {
                0
            };
            String::from_utf8_lossy(&bytes[start..]).into_owned()
        }
        FileEncoding::Utf16Le => {
            let (decoded, _, _) = encoding_rs::UTF_16LE.decode(&bytes);
            decoded.into_owned()
        }
        FileEncoding::Utf16Be => {
            let (decoded, _, _) = encoding_rs::UTF_16BE.decode(&bytes);
            decoded.into_owned()
        }
        FileEncoding::Latin1 => {
            let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
            decoded.into_owned()
        }
    };
    let line_ending = detect_line_ending(&text);
    Ok((text, encoding, line_ending))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_line_ending_lf() {
        assert_eq!(detect_line_ending("hello\nworld\n"), LineEnding::Lf);
    }

    #[test]
    fn test_detect_line_ending_crlf() {
        assert_eq!(detect_line_ending("hello\r\nworld\r\n"), LineEnding::CrLf);
    }

    #[test]
    fn test_detect_encoding_utf8() {
        assert_eq!(detect_encoding(b"hello world"), FileEncoding::Utf8);
    }

    #[test]
    fn test_detect_encoding_utf8_bom() {
        let bytes = [0xEF, 0xBB, 0xBF, b'h', b'i'];
        assert_eq!(detect_encoding(&bytes), FileEncoding::Utf8);
    }
}
