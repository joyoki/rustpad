use std::path::PathBuf;

use super::{FileEncoding, LineEnding};

/// BOM (Byte Order Mark) detection result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomType {
    Utf8,
    Utf16Le,
    Utf16Be,
    None,
}

/// File read result with metadata.
#[derive(Debug)]
pub struct FileReadResult {
    pub content: String,
    pub encoding: FileEncoding,
    pub line_ending: LineEnding,
    pub has_bom: bool,
    pub file_size: u64,
    pub is_large_file: bool,
}

/// Async file reader with encoding detection.
pub struct FileReader;

impl FileReader {
    /// Read a file asynchronously with automatic encoding detection.
    pub async fn read(path: &PathBuf) -> anyhow::Result<FileReadResult> {
        let bytes = tokio::fs::read(path).await?;
        let file_size = bytes.len() as u64;
        let is_large_file = file_size > 10 * 1024 * 1024; // 10MB

        let (bom_type, encoding) = detect_bom_and_encoding(&bytes);
        let has_bom = bom_type != BomType::None;
        let content = decode_bytes(&bytes, encoding, has_bom)?;
        let line_ending = detect_line_ending(&content);

        Ok(FileReadResult {
            content,
            encoding,
            line_ending,
            has_bom,
            file_size,
            is_large_file,
        })
    }

    /// Read a large file in chunks (for files > 10MB).
    pub async fn read_chunked(
        path: &PathBuf,
        start_line: usize,
        line_count: usize,
    ) -> anyhow::Result<(String, FileEncoding, LineEnding)> {
        let bytes = tokio::fs::read(path).await?;
        let (bom_type, encoding) = detect_bom_and_encoding(&bytes);
        let has_bom = bom_type != BomType::None;
        let content = decode_bytes(&bytes, encoding, has_bom)?;

        let lines: Vec<&str> = content.lines().collect();
        let end_line = (start_line + line_count).min(lines.len());
        let chunk: String = lines[start_line..end_line].join("\n");

        let line_ending = detect_line_ending(&content);
        Ok((chunk, encoding, line_ending))
    }
}

/// Async file writer with encoding support.
pub struct FileWriter;

impl FileWriter {
    /// Write content to a file asynchronously.
    pub async fn write(
        path: &PathBuf,
        content: &str,
        encoding: FileEncoding,
        line_ending: LineEnding,
        write_bom: bool,
    ) -> anyhow::Result<()> {
        // Convert line endings
        let normalized = normalize_line_endings(content, line_ending);

        // Encode content
        let bytes = encode_string(&normalized, encoding, write_bom)?;

        // Write to file
        tokio::fs::write(path, &bytes).await?;
        Ok(())
    }
}

/// Detect BOM and encoding from raw bytes.
fn detect_bom_and_encoding(bytes: &[u8]) -> (BomType, FileEncoding) {
    if bytes.len() >= 3 && bytes[..3] == [0xEF, 0xBB, 0xBF] {
        return (BomType::Utf8, FileEncoding::Utf8);
    }
    if bytes.len() >= 2 && bytes[..2] == [0xFF, 0xFE] {
        return (BomType::Utf16Le, FileEncoding::Utf16Le);
    }
    if bytes.len() >= 2 && bytes[..2] == [0xFE, 0xFF] {
        return (BomType::Utf16Be, FileEncoding::Utf16Be);
    }

    // Try UTF-8 validation
    if std::str::from_utf8(bytes).is_ok() {
        return (BomType::None, FileEncoding::Utf8);
    }

    let (_, _, gbk_errors) = encoding_rs::GBK.decode(bytes);
    if !gbk_errors {
        return (BomType::None, FileEncoding::Gbk);
    }

    let (_, _, latin_errors) = encoding_rs::WINDOWS_1252.decode(bytes);
    if !latin_errors {
        return (BomType::None, FileEncoding::Latin1);
    }

    if cfg!(windows) {
        (BomType::None, FileEncoding::Gbk)
    } else {
        (BomType::None, FileEncoding::Latin1)
    }
}

/// Decode bytes to UTF-8 string based on encoding.
fn decode_bytes(bytes: &[u8], encoding: FileEncoding, has_bom: bool) -> anyhow::Result<String> {
    match encoding {
        FileEncoding::Utf8 => {
            let start = if has_bom { 3 } else { 0 };
            Ok(String::from_utf8_lossy(&bytes[start..]).into_owned())
        }
        FileEncoding::Utf16Le => {
            let (decoded, _, _) = encoding_rs::UTF_16LE.decode(bytes);
            Ok(decoded.into_owned())
        }
        FileEncoding::Utf16Be => {
            let (decoded, _, _) = encoding_rs::UTF_16BE.decode(bytes);
            Ok(decoded.into_owned())
        }
        FileEncoding::Latin1 => {
            let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
            Ok(decoded.into_owned())
        }
        FileEncoding::Gbk => {
            let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
            Ok(decoded.into_owned())
        }
    }
}

/// Encode string to bytes based on encoding.
fn encode_string(
    content: &str,
    encoding: FileEncoding,
    write_bom: bool,
) -> anyhow::Result<Vec<u8>> {
    let mut bytes = Vec::new();

    // Write BOM if requested
    if write_bom {
        match encoding {
            FileEncoding::Utf8 => {
                bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            }
            FileEncoding::Utf16Le => {
                bytes.extend_from_slice(&[0xFF, 0xFE]);
            }
            FileEncoding::Utf16Be => {
                bytes.extend_from_slice(&[0xFE, 0xFF]);
            }
            _ => {}
        }
    }

    match encoding {
        FileEncoding::Utf8 => {
            bytes.extend_from_slice(content.as_bytes());
        }
        FileEncoding::Utf16Le => {
            let (encoded, _, _) = encoding_rs::UTF_16LE.encode(content);
            bytes.extend_from_slice(&encoded);
        }
        FileEncoding::Utf16Be => {
            let (encoded, _, _) = encoding_rs::UTF_16BE.encode(content);
            bytes.extend_from_slice(&encoded);
        }
        FileEncoding::Latin1 => {
            let (encoded, _, _) = encoding_rs::WINDOWS_1252.encode(content);
            bytes.extend_from_slice(&encoded);
        }
        FileEncoding::Gbk => {
            let (encoded, _, _) = encoding_rs::GBK.encode(content);
            bytes.extend_from_slice(&encoded);
        }
    }

    Ok(bytes)
}

/// Detect line ending style from text content.
fn detect_line_ending(text: &str) -> LineEnding {
    if text.contains("\r\n") {
        LineEnding::CrLf
    } else if text.contains('\r') {
        LineEnding::Cr
    } else {
        LineEnding::Lf
    }
}

/// Normalize line endings to the specified style.
fn normalize_line_endings(content: &str, line_ending: LineEnding) -> String {
    // First normalize to LF
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");

    // Then convert to target line ending
    match line_ending {
        LineEnding::Lf => normalized,
        LineEnding::CrLf => normalized.replace('\n', "\r\n"),
        LineEnding::Cr => normalized.replace('\n', "\r"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_bom_utf8() {
        let bytes = [0xEF, 0xBB, 0xBF, b'h', b'i'];
        let (bom, encoding) = detect_bom_and_encoding(&bytes);
        assert_eq!(bom, BomType::Utf8);
        assert_eq!(encoding, FileEncoding::Utf8);
    }

    #[test]
    fn test_detect_bom_utf16le() {
        let bytes = [0xFF, 0xFE, b'h', 0];
        let (bom, encoding) = detect_bom_and_encoding(&bytes);
        assert_eq!(bom, BomType::Utf16Le);
        assert_eq!(encoding, FileEncoding::Utf16Le);
    }

    #[test]
    fn test_detect_bom_none() {
        let bytes = [b'h', b'e', b'l', b'l', b'o'];
        let (bom, encoding) = detect_bom_and_encoding(&bytes);
        assert_eq!(bom, BomType::None);
        assert_eq!(encoding, FileEncoding::Utf8);
    }

    #[test]
    fn test_normalize_line_endings() {
        assert_eq!(normalize_line_endings("a\r\nb\rc", LineEnding::Lf), "a\nb\nc");
        assert_eq!(normalize_line_endings("a\nb\nc", LineEnding::CrLf), "a\r\nb\r\nc");
    }

    #[test]
    fn test_encode_decode_utf8() {
        let content = "Hello, World!";
        let bytes = encode_string(content, FileEncoding::Utf8, false).unwrap();
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), content);
    }
}
