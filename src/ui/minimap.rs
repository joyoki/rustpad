use eframe::egui;
use eframe::epaint::Color32;

/// Minimap state.
#[derive(Debug)]
pub struct Minimap {
    pub enabled: bool,
    pub width: f32,
    pub line_height: f32,
    pub scroll_offset: f32,
}

impl Minimap {
    pub fn new() -> Self {
        Self {
            enabled: true,
            width: 100.0,
            line_height: 2.0,
            scroll_offset: 0.0,
        }
    }

    /// Toggle minimap visibility.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl Default for Minimap {
    fn default() -> Self {
        Self::new()
    }
}

/// Show the minimap panel.
pub fn show(app: &mut crate::app::RustpadApp, ctx: &egui::Context) {
    if !app.minimap.enabled {
        return;
    }

    // Get the data we need before the closure
    let text = app.tab_manager.active().buffer.text();
    let current_line = app.tab_manager.active().cursor.line;
    let minimap_width = app.minimap.width;
    let minimap_line_height = app.minimap.line_height;

    let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let total_lines = lines.len();

    egui::SidePanel::right("minimap_panel")
        .default_width(minimap_width)
        .show(ctx, |ui| {
            let available_height = ui.available_height();
            let scale = available_height / (total_lines as f32 * minimap_line_height).max(1.0);

            let response = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.cursor().left_top(),
                    egui::vec2(minimap_width, available_height),
                ),
                egui::Sense::click_and_drag(),
            );

            let painter = ui.painter();
            let start_pos = response.rect.left_top();

            // Draw each line as a colored block
            for (i, line) in lines.iter().enumerate() {
                let y = start_pos.y + (i as f32 * minimap_line_height * scale);
                let width = (line.len() as f32 * 0.5).min(minimap_width - 4.0);

                let color = if line.trim().is_empty() {
                    Color32::TRANSPARENT
                } else {
                    Color32::from_gray(120)
                };

                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(start_pos.x + 2.0, y),
                        egui::vec2(width, minimap_line_height),
                    ),
                    0.0,
                    color,
                );
            }

            // Highlight current line
            let y = start_pos.y + (current_line as f32 * minimap_line_height * scale);
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(start_pos.x, y),
                    egui::vec2(minimap_width, minimap_line_height * 3.0),
                ),
                0.0,
                Color32::from_rgba_premultiplied(100, 150, 255, 40),
            );

            // Handle click to jump to line
            if response.clicked() {
                if let Some(click_pos) = response.interact_pointer_pos() {
                    let rel_y = click_pos.y - start_pos.y;
                    let line = (rel_y / (minimap_line_height * scale)) as usize;
                    if line < total_lines {
                        app.tab_manager.active_mut().cursor.line = line;
                    }
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimap_default() {
        let minimap = Minimap::default();
        assert!(minimap.enabled);
        assert_eq!(minimap.width, 100.0);
    }

    #[test]
    fn test_minimap_toggle() {
        let mut minimap = Minimap::new();
        assert!(minimap.enabled);
        minimap.toggle();
        assert!(!minimap.enabled);
    }
}
