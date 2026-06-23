/// Windows-specific platform code.
use std::path::PathBuf;

/// Get the default application icon path for Windows.
pub fn app_icon_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("assets/icon.ico"),
        PathBuf::from("icons/icon.ico"),
    ];
    candidates.iter().find(|p| p.exists()).cloned()
}

/// Configure Windows-specific eframe options.
pub fn configure_native_options(options: &mut eframe::NativeOptions) {
    // Windows: DPI awareness is handled by eframe automatically
    log::info!("Configuring Windows-specific options");
}

/// Generate a registry script for file associations.
/// Returns the content of a .reg file that registers RustPad as an editor.
pub fn generate_file_association_script() -> String {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| r"C:\Program Files\RustPad\rustpad.exe".to_string());

    format!(
        r#"Windows Registry Editor Version 5.00

[HKEY_CLASSES_ROOT\*\shell\OpenWithRustPad]
@="Open with RustPad"
"Icon"="{exe}"

[HKEY_CLASSES_ROOT\*\shell\OpenWithRustPad\command]
@="\"{exe}\" \"%1\""

[HKEY_CLASSES_ROOT\.rs\OpenWithList\RustPad]
@=""
"#,
        exe = exe_path
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_association_script() {
        let script = generate_file_association_script();
        assert!(script.contains("RustPad"));
        assert!(script.contains("Windows Registry Editor"));
    }
}
