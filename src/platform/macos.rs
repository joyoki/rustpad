/// macOS-specific platform code.
use std::path::PathBuf;

/// Get the default application icon path for macOS.
pub fn app_icon_path() -> Option<PathBuf> {
    // Look for .icns file in common locations
    let candidates = [
        PathBuf::from("assets/icon.icns"),
        PathBuf::from("icons/icon.icns"),
    ];
    candidates.iter().find(|p| p.exists()).cloned()
}

/// Configure macOS-specific eframe options.
pub fn configure_native_options(_options: &mut eframe::NativeOptions) {
    // Native menu bar is installed via muda in main.rs after NSApp is ready.
    // macOS automatically places menu items in the notch safe area on Apple Silicon MacBooks.
    log::info!("Configuring macOS-specific options");
}

/// Check if the application should handle terminate event.
pub fn should_confirm_quit(has_unsaved_changes: bool) -> bool {
    has_unsaved_changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_quit() {
        assert!(should_confirm_quit(true));
        assert!(!should_confirm_quit(false));
    }
}
