#![allow(dead_code)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::needless_update)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::needless_range_loop)]

mod app;
mod config;
mod diff;
mod editor;
mod highlight;
mod platform;
mod search;
mod session;
mod ui;

use eframe::egui;

/// Initialize the tokio multi-thread runtime and start the application.
fn main() -> eframe::Result {
    // Initialize platform-specific logging (respects RUSTPAD_LOG env var)
    platform::init_logging();

    // Install crash handler
    platform::setup_panic_hook();

    log::info!("Starting RustPad v{}", env!("CARGO_PKG_VERSION"));

    // Build tokio runtime for async operations (file watching, etc.)
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    // Keep runtime alive for the entire application lifetime
    let _guard = runtime.enter();

    // Configure native window options (Glow/OpenGL for broad macOS compatibility)
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("RustPad"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    // Platform-specific configuration
    #[cfg(target_os = "macos")]
    {
        platform::macos::configure_native_options(&mut options);

        // Disable winit's default macOS menu. Its "Quit" item binds Cmd+Q and is
        // handled by AppKit directly, which terminates the process before eframe's
        // `close_requested()` fires (eframe issue #7115). Disabling it lets the
        // Cmd+Q key event reach egui so we can prompt to save unsaved changes.
        use winit::platform::macos::EventLoopBuilderExtMacOS;
        options.event_loop_builder = Some(Box::new(|builder| {
            builder.with_default_menu(false);
        }));
    }

    #[cfg(target_os = "windows")]
    platform::windows::configure_native_options(&mut options);

    log::info!("Launching native window...");

    if let Err(e) = eframe::run_native(
        "RustPad",
        options,
        Box::new(|cc| {
            log::info!("eframe initialized, creating app...");
            Ok(Box::new(app::RustpadApp::new(cc)))
        }),
    ) {
        eprintln!("启动失败: {e}");
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main_compiles() {
        assert!(true);
    }
}
