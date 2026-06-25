use super::LineEnding;

/// User-facing encoding choice (toolbar / status bar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EncodingProfile {
    /// ANSI / GBK (Simplified Chinese Windows).
    AnsiGbk,
    /// UTF-8 without BOM.
    #[default]
    Utf8,
    /// UTF-8 with BOM.
    Utf8Bom,
    /// UTF-16 big-endian with BOM on save.
    Utf16Be,
    /// UTF-16 little-endian with BOM on save.
    Utf16Le,
    /// Western European (Windows-1252 / Latin-1).
    Latin1,
}

impl EncodingProfile {
    /// Encodings shown in the main toolbar menu.
    pub const MAIN: [Self; 5] = [
        Self::AnsiGbk,
        Self::Utf8,
        Self::Utf8Bom,
        Self::Utf16Be,
        Self::Utf16Le,
    ];

    /// Additional encodings under "More…".
    pub const MORE: [Self; 1] = [Self::Latin1];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::AnsiGbk => "ANSI/GBK",
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 BOM",
            Self::Utf16Be => "UTF-16 BE",
            Self::Utf16Le => "UTF-16 LE",
            Self::Latin1 => "Western (1252)",
        }
    }

    pub fn status_label(self) -> &'static str {
        match self {
            Self::AnsiGbk => "GBK",
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 BOM",
            Self::Utf16Be => "UTF-16 BE",
            Self::Utf16Le => "UTF-16 LE",
            Self::Latin1 => "1252",
        }
    }

    pub fn writes_bom(self) -> bool {
        matches!(
            self,
            Self::Utf8Bom | Self::Utf16Be | Self::Utf16Le
        )
    }

    /// Stable id suffix for native menu items (`enc.open.{id}` / `enc.convert.{id}`).
    pub fn menu_id(self) -> &'static str {
        match self {
            Self::AnsiGbk => "AnsiGbk",
            Self::Utf8 => "Utf8",
            Self::Utf8Bom => "Utf8Bom",
            Self::Utf16Be => "Utf16Be",
            Self::Utf16Le => "Utf16Le",
            Self::Latin1 => "Latin1",
        }
    }

    pub fn from_menu_id(id: &str) -> Option<Self> {
        match id {
            "AnsiGbk" => Some(Self::AnsiGbk),
            "Utf8" => Some(Self::Utf8),
            "Utf8Bom" => Some(Self::Utf8Bom),
            "Utf16Be" => Some(Self::Utf16Be),
            "Utf16Le" => Some(Self::Utf16Le),
            "Latin1" => Some(Self::Latin1),
            _ => None,
        }
    }

    /// All profiles shown in encoding menus.
    pub fn menu_profiles() -> impl Iterator<Item = Self> {
        Self::MAIN.iter().copied().chain(Self::MORE.iter().copied())
    }

    /// Decode raw file bytes using the selected profile (for "open with encoding").
    pub fn decode_bytes(self, bytes: &[u8]) -> String {
        match self {
            Self::Utf8 | Self::Utf8Bom => {
                let start = utf8_bom_skip(bytes);
                String::from_utf8_lossy(&bytes[start..]).into_owned()
            }
            Self::Utf16Le => decode_utf16(bytes, true),
            Self::Utf16Be => decode_utf16(bytes, false),
            Self::AnsiGbk => {
                let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
                decoded.into_owned()
            }
            Self::Latin1 => {
                let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
                decoded.into_owned()
            }
        }
    }

    /// Encode in-memory UTF-8 text to on-disk bytes.
    pub fn encode_text(self, content: &str, line_ending: LineEnding) -> Vec<u8> {
        let normalized = normalize_line_endings(content, line_ending);
        let mut bytes = Vec::new();

        if self.writes_bom() {
            match self {
                Self::Utf8Bom => bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]),
                Self::Utf16Le => bytes.extend_from_slice(&[0xFF, 0xFE]),
                Self::Utf16Be => bytes.extend_from_slice(&[0xFE, 0xFF]),
                _ => {}
            }
        }

        match self {
            Self::Utf8 | Self::Utf8Bom => bytes.extend_from_slice(normalized.as_bytes()),
            Self::Utf16Le => {
                bytes.extend_from_slice(&encode_utf16_units(&normalized, true));
            }
            Self::Utf16Be => {
                bytes.extend_from_slice(&encode_utf16_units(&normalized, false));
            }
            Self::AnsiGbk => {
                let (encoded, _, _) = encoding_rs::GBK.encode(&normalized);
                bytes.extend_from_slice(&encoded);
            }
            Self::Latin1 => {
                let (encoded, _, _) = encoding_rs::WINDOWS_1252.encode(&normalized);
                bytes.extend_from_slice(&encoded);
            }
        }

        bytes
    }
}

/// Auto-detect encoding from raw bytes.
///
/// Order: BOM markers, valid UTF-8, clean GBK (common on Chinese Windows),
/// clean Windows-1252, then a platform-appropriate ANSI fallback.
pub fn detect_encoding_profile(bytes: &[u8]) -> EncodingProfile {
    if bytes.len() >= 3 && bytes[..3] == [0xEF, 0xBB, 0xBF] {
        return EncodingProfile::Utf8Bom;
    }
    if bytes.len() >= 2 && bytes[..2] == [0xFF, 0xFE] {
        return EncodingProfile::Utf16Le;
    }
    if bytes.len() >= 2 && bytes[..2] == [0xFE, 0xFF] {
        return EncodingProfile::Utf16Be;
    }
    if std::str::from_utf8(bytes).is_ok() {
        return EncodingProfile::Utf8;
    }

    let (_, _, gbk_errors) = encoding_rs::GBK.decode(bytes);
    if !gbk_errors {
        return EncodingProfile::AnsiGbk;
    }

    let (_, _, latin_errors) = encoding_rs::WINDOWS_1252.decode(bytes);
    if !latin_errors {
        return EncodingProfile::Latin1;
    }

    if cfg!(windows) {
        EncodingProfile::AnsiGbk
    } else {
        EncodingProfile::Utf8
    }
}

fn utf8_bom_skip(bytes: &[u8]) -> usize {
    if bytes.len() >= 3 && bytes[..3] == [0xEF, 0xBB, 0xBF] {
        3
    } else {
        0
    }
}

fn encode_utf16_units(text: &str, little_endian: bool) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(text.len() * 2);
    for unit in text.encode_utf16() {
        if little_endian {
            bytes.extend_from_slice(&unit.to_le_bytes());
        } else {
            bytes.extend_from_slice(&unit.to_be_bytes());
        }
    }
    bytes
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> String {
    let mut start = 0usize;
    if bytes.len() >= 2 {
        let bom = if little_endian {
            [0xFF, 0xFE]
        } else {
            [0xFE, 0xFF]
        };
        if bytes[..2] == bom {
            start = 2;
        }
    }
    let data = &bytes[start..];
    if !data.len().is_multiple_of(2) {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let units: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| {
            if little_endian {
                u16::from_le_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_be_bytes([chunk[0], chunk[1]])
            }
        })
        .collect();
    String::from_utf16_lossy(&units)
}

fn normalize_line_endings(content: &str, line_ending: LineEnding) -> String {
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
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
    fn test_detect_utf8_bom() {
        let bytes = [0xEF, 0xBB, 0xBF, b'h', b'i'];
        assert_eq!(detect_encoding_profile(&bytes), EncodingProfile::Utf8Bom);
    }

    #[test]
    fn test_detect_utf8_plain() {
        assert_eq!(detect_encoding_profile(b"hello"), EncodingProfile::Utf8);
    }

    #[test]
    fn test_detect_gbk_without_bom() {
        let bytes = EncodingProfile::AnsiGbk.encode_text("中文测试", LineEnding::Lf);
        assert_eq!(detect_encoding_profile(&bytes), EncodingProfile::AnsiGbk);
    }

    #[test]
    fn test_detect_latin1_without_bom() {
        let bytes = EncodingProfile::Latin1.encode_text("café", LineEnding::Lf);
        assert_eq!(detect_encoding_profile(&bytes), EncodingProfile::Latin1);
    }

    #[test]
    fn test_encode_utf8_bom() {
        let bytes = EncodingProfile::Utf8Bom.encode_text("hi", LineEnding::Lf);
        assert_eq!(&bytes[..3], &[0xEF, 0xBB, 0xBF]);
        assert_eq!(&bytes[3..], b"hi");
    }

    #[test]
    fn test_gbk_roundtrip() {
        let text = "中文测试";
        let bytes = EncodingProfile::AnsiGbk.encode_text(text, LineEnding::Lf);
        let decoded = EncodingProfile::AnsiGbk.decode_bytes(&bytes);
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_utf16le_roundtrip() {
        let text = "Hello";
        let bytes = EncodingProfile::Utf16Le.encode_text(text, LineEnding::Lf);
        assert_eq!(&bytes[..2], &[0xFF, 0xFE]);
        let decoded = EncodingProfile::Utf16Le.decode_bytes(&bytes);
        assert_eq!(decoded, text);
    }
}
