use std::path::Path;

/// Reveal a file in the system file manager (select/highlight when supported).
pub fn reveal_file_in_folder(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg("-R").arg(path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
        }
    }
}

/// Open a terminal window with the working directory set to `path`'s parent folder.
pub fn open_terminal_in_directory(path: &Path) {
    let dir = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| path.to_path_buf());

    #[cfg(target_os = "windows")]
    {
        let dir_arg = dir.to_string_lossy();
        let _ = std::process::Command::new("cmd")
            .args([
                "/C",
                "start",
                "",
                "/D",
                &dir_arg,
                "cmd",
                "/K",
                &format!("cd /d {} && title RustPad", dir_arg),
            ])
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let dir_escaped = applescript_escape_path(&dir);
        let script = format!(
            r#"tell application "Terminal"
    activate
    do script ("cd " & quoted form of "{dir_escaped}")
end tell"#
        );
        let _ = std::process::Command::new("osascript").arg("-e").arg(script).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let dir_s = dir.to_string_lossy();
        if std::process::Command::new("gnome-terminal")
            .args(["--working-directory", &dir_s])
            .spawn()
            .is_ok()
        {
            return;
        }
        if std::process::Command::new("konsole")
            .args(["--workdir", &dir_s])
            .spawn()
            .is_ok()
        {
            return;
        }
        let script = format!("cd {} && exec $SHELL", dir.display());
        let _ = std::process::Command::new("xterm")
            .args(["-e", &script])
            .spawn();
    }
}

#[cfg(target_os = "macos")]
fn applescript_escape_path(path: &std::path::Path) -> String {
    path.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_missing_path_does_not_panic() {
        let p = Path::new("/nonexistent/rustpad-test-file.txt");
        reveal_file_in_folder(p);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn applescript_escape_path_handles_quotes_and_backslashes() {
        let p = Path::new("/tmp/foo \"bar\"\\baz");
        let escaped = applescript_escape_path(p);
        assert!(escaped.contains("\\\""));
        assert!(escaped.contains("\\\\"));
    }
}
