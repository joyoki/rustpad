use std::path::PathBuf;

pub mod autocomplete;
pub mod buffer;
pub mod column_selection;
pub mod context_actions;
pub mod cursor;
pub mod encoding;
pub mod file_io;
pub mod fold;
pub mod indent;
pub mod macro_recorder;
pub mod tab;
pub mod view;

pub use buffer::TextBuffer;
pub use cursor::{Cursor, Selection};
pub use encoding::EncodingProfile;
pub use tab::TabManager;

/// Legacy encoding enum (used by async file_io).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FileEncoding {
    #[default]
    Utf8,
    Utf16Le,
    Utf16Be,
    Latin1,
    Gbk,
}

impl FileEncoding {
    pub fn from_profile(profile: EncodingProfile) -> Self {
        match profile {
            EncodingProfile::AnsiGbk => Self::Gbk,
            EncodingProfile::Utf8 | EncodingProfile::Utf8Bom => Self::Utf8,
            EncodingProfile::Utf16Le => Self::Utf16Le,
            EncodingProfile::Utf16Be => Self::Utf16Be,
            EncodingProfile::Latin1 => Self::Latin1,
        }
    }

    pub fn to_profile(self, had_utf8_bom: bool) -> EncodingProfile {
        match self {
            Self::Gbk => EncodingProfile::AnsiGbk,
            Self::Utf16Le => EncodingProfile::Utf16Le,
            Self::Utf16Be => EncodingProfile::Utf16Be,
            Self::Latin1 => EncodingProfile::Latin1,
            Self::Utf8 if had_utf8_bom => EncodingProfile::Utf8Bom,
            Self::Utf8 => EncodingProfile::Utf8,
        }
    }
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

/// Read a file into a UTF-8 string, handling various encodings.
pub fn read_file_to_string(
    path: &PathBuf,
) -> anyhow::Result<(String, EncodingProfile, LineEnding)> {
    let bytes = std::fs::read(path)?;
    let profile = encoding::detect_encoding_profile(&bytes);
    let text = profile.decode_bytes(&bytes);
    let line_ending = detect_line_ending(&text);
    Ok((text, profile, line_ending))
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
        assert_eq!(
            encoding::detect_encoding_profile(b"hello world"),
            EncodingProfile::Utf8
        );
    }

    #[test]
    fn test_detect_encoding_utf8_bom() {
        let bytes = [0xEF, 0xBB, 0xBF, b'h', b'i'];
        assert_eq!(
            encoding::detect_encoding_profile(&bytes),
            EncodingProfile::Utf8Bom
        );
    }
}
