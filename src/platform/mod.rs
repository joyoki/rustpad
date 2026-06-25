/// Platform-specific abstractions.
pub mod shell;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub mod macos_menu;

#[cfg(target_os = "windows")]
pub mod windows;

use std::path::PathBuf;

/// Get the crash log file path for the current platform.
pub fn crash_log_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rustpad")
        .join("crash.log")
}

/// Setup a panic hook that writes to crash.log.
pub fn setup_panic_hook() {
    let crash_path = crash_log_path();
    std::panic::set_hook(Box::new(move |panic_info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");

        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<dyn Any>".to_string()
        };

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        let message = format!(
            "PANIC at {} in thread '{}' at {}\nBacktrace:\n{}\n",
            payload, thread_name, location, backtrace
        );

        eprintln!("{}", message);

        if let Some(parent) = crash_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&crash_path)
        {
            use std::io::Write;
            let _ = writeln!(f, "{}", message);
        }
    }));
}

/// Initialize platform-specific logging based on RUSTPAD_LOG env var.
pub fn init_logging() {
    use std::io::Write;

    let env = env_logger::Env::default().filter_or("RUSTPAD_LOG", "info");
    env_logger::Builder::from_env(env)
        .format_timestamp_millis()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [RustPad] {:5} {}",
                buf.timestamp(),
                record.level(),
                record.args()
            )
        })
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_log_path() {
        let path = crash_log_path();
        assert!(path.to_string_lossy().contains("crash.log"));
    }
}
