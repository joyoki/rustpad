//! Application logo and window icon (embedded from `assets/logo.png`).

use eframe::egui;

const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

/// Build the native window / taskbar icon from the embedded logo.
pub fn load_window_icon() -> egui::IconData {
    let image = image::load_from_memory(LOGO_PNG).expect("valid logo png");
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    }
}

/// Load the logo as an egui texture for in-app display (About dialog, startup splash).
pub fn load_logo_texture(ctx: &egui::Context) -> egui::TextureHandle {
    let image = image::load_from_memory(LOGO_PNG).expect("valid logo png");
    let rgba = image.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.as_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
    ctx.load_texture(
        "rustpad_logo",
        color_image,
        egui::TextureOptions::LINEAR,
    )
}

/// Brief centered splash shown when the application opens.
pub fn paint_startup_splash(ctx: &egui::Context, logo: &egui::TextureHandle) {
    let screen = ctx.screen_rect();
    ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("startup_splash"),
    ))
    .rect_filled(screen, 0.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 230));

    let logo_size = 128.0_f32.min(screen.width() * 0.28);

    egui::Area::new(egui::Id::new("startup_splash_content"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, -18.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.image((logo.id(), egui::vec2(logo_size, logo_size)));
                ui.add_space(8.0);
                ui.heading("RustPad");
                ui.label(
                    egui::RichText::new(env!("CARGO_PKG_VERSION"))
                        .size(13.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
}
