//! Shared text layout helpers for the UI.

/// Truncate `text` with an ellipsis in the middle when longer than `max_chars`.
pub fn ellipsis_middle(text: &str, max_chars: usize) -> String {
    if max_chars < 4 {
        return "...".chars().take(max_chars).collect();
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        return text.to_string();
    }
    let head = (max_chars - 3) / 2;
    let tail = max_chars - 3 - head;
    let start: String = chars.iter().take(head).collect();
    let end: String = chars.iter().skip(chars.len() - tail).collect();
    format!("{start}...{end}")
}

/// Estimate a middle-ellipsis path that fits the available sidebar width.
pub fn ellipsis_path_for_width(path: &str, available_width: f32) -> String {
    let max_chars = (available_width / 6.5).floor() as usize;
    ellipsis_middle(path, max_chars.clamp(12, 64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ellipsis_middle_keeps_short_text() {
        assert_eq!(ellipsis_middle("short.txt", 20), "short.txt");
    }

    #[test]
    fn ellipsis_middle_truncates_long_path() {
        let path = "/Users/me/Documents/projects/rustpad/src/main.rs";
        let out = ellipsis_middle(path, 24);
        assert!(out.contains("..."));
        assert_eq!(out.chars().count(), 24);
        assert!(out.starts_with('/'));
        assert!(out.ends_with(".rs"));
    }

    #[test]
    fn ellipsis_middle_preserves_special_chars() {
        let text = "a<b>&\"cdefghijklmnop";
        let out = ellipsis_middle(text, 10);
        assert_eq!(out, "a<b...mnop");
    }
}
