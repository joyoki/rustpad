use eframe::egui;

use crate::app::RustpadApp;
use crate::ui::encoding_menu;

const FONT_SIZE_MIN: u32 = 8;
const FONT_SIZE_MAX: u32 = 72;
const FONT_SIZE_FIELD_ID: &str = "rustpad_toolbar_font_size";
/// Below this width, toolbar shows icons only; wider layouts show icon + label.
const TOOLBAR_COMPACT_WIDTH: f32 = 820.0;
/// Toolbar row height (egui default `interact_size.y` is 18.0).
pub const TOOLBAR_HEIGHT: f32 = 30.0;
const TOOLBAR_INTERACT_HEIGHT: f32 = TOOLBAR_HEIGHT - 4.0;
const TOOLBAR_FONT_SIZE: f32 = 14.0;
const TOOLBAR_BUTTON_PADDING: egui::Vec2 = egui::Vec2::new(8.0, 5.0);
const TOOLBAR_ITEM_SPACING_X: f32 = 6.0;
const TOOLBAR_FONT_FIELD_WIDTH: f32 = 40.0;

fn apply_toolbar_style(ui: &mut egui::Ui) {
    let mut style = ui.style().as_ref().clone();
    style.spacing.interact_size.y = TOOLBAR_INTERACT_HEIGHT;
    style.spacing.button_padding = TOOLBAR_BUTTON_PADDING;
    style.spacing.item_spacing.x = TOOLBAR_ITEM_SPACING_X;
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(TOOLBAR_FONT_SIZE));
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(TOOLBAR_FONT_SIZE));
    ui.set_style(style);
}

fn toolbar_label(icon: &str, text: &str, compact: bool) -> String {
    if compact {
        icon.to_string()
    } else {
        format!("{icon} {text}")
    }
}

/// Toolbar icon button with hover tooltip; disabled buttons still show the tooltip.
fn toolbar_button(ui: &mut egui::Ui, label: &str, tooltip: &str, enabled: bool) -> bool {
    let response = ui.add_enabled(enabled, egui::Button::new(label));
    if enabled {
        response.on_hover_text(tooltip).clicked()
    } else {
        ui.interact(
            response.rect,
            response.id.with("tooltip"),
            egui::Sense::hover(),
        )
        .on_hover_text(tooltip);
        false
    }
}

fn parse_font_size(text: &str) -> Option<u32> {
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

fn apply_font_size(text_buf: &mut String, font_size: &mut f32) {
    if let Some(n) = parse_font_size(text_buf) {
        let clamped = n.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX);
        *font_size = clamped as f32;
        *text_buf = format!("{clamped}");
    } else {
        *text_buf = format!("{}", *font_size as u32);
    }
}

/// Editable font-size field: click, type a number, press Enter or click away to apply.
fn show_font_size_input(
    ui: &mut egui::Ui,
    font_size: &mut f32,
    text_buf: &mut String,
    editing: &mut bool,
    editor_has_focus: &mut bool,
    tooltip: &str,
) {
    let field_id = egui::Id::new(FONT_SIZE_FIELD_ID);

    let response = ui
        .with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let resp = ui.add(
                egui::TextEdit::singleline(text_buf)
                    .id(field_id)
                    .desired_width(TOOLBAR_FONT_FIELD_WIDTH)
                    .min_size(egui::vec2(TOOLBAR_FONT_FIELD_WIDTH, TOOLBAR_INTERACT_HEIGHT))
                    .horizontal_align(egui::Align::Center)
                    .vertical_align(egui::Align::Center)
                    .margin(egui::Margin::symmetric(4.0, 0.0))
                    .lock_focus(*editing),
            );
            ui.label("px");
            resp
        })
        .inner;

    if response.gained_focus() {
        *editing = true;
        *editor_has_focus = false;
    }

    if response.has_focus() {
        *editing = true;
        *editor_has_focus = false;
    }

    // Commit only when the user was actively editing this field.
    if response.lost_focus() && *editing {
        apply_font_size(text_buf, font_size);
        *editing = false;
    } else if !response.has_focus() && !*editing {
        let expected = format!("{}", *font_size as u32);
        if text_buf != &expected {
            *text_buf = expected;
        }
    }

    response.on_hover_text(tooltip);
}

/// Render the toolbar with common action buttons.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    let t = app.tr();
    egui::TopBottomPanel::top("toolbar")
        .exact_height(TOOLBAR_HEIGHT)
        .show(ctx, |ui| {
        apply_toolbar_style(ui);
        let compact = ui.available_width() < TOOLBAR_COMPACT_WIDTH;
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.horizontal_wrapped(|ui| {
            if toolbar_button(
                ui,
                &toolbar_label(t.tb_new, t.tb_label_new, compact),
                t.tip_new,
                true,
            ) {
                app.tab_manager.new_tab();
            }
            if toolbar_button(
                ui,
                &toolbar_label(t.tb_open, t.tb_label_open, compact),
                t.tip_open,
                true,
            ) {
                app.pending_open_file = true;
            }
            if toolbar_button(
                ui,
                &toolbar_label(t.tb_save, t.tb_label_save, compact),
                t.tip_save,
                true,
            ) {
                app.save_current_tab();
            }
            ui.separator();
            let can_undo = app.tab_manager.active().buffer.can_undo();
            if toolbar_button(
                ui,
                &toolbar_label(t.tb_undo, t.tb_label_undo, compact),
                t.tip_undo,
                can_undo,
            ) {
                app.tab_manager.active_mut().buffer.undo();
            }
            let can_redo = app.tab_manager.active().buffer.can_redo();
            if toolbar_button(
                ui,
                &toolbar_label(t.tb_redo, t.tb_label_redo, compact),
                t.tip_redo,
                can_redo,
            ) {
                app.tab_manager.active_mut().buffer.redo();
            }
            ui.separator();
            if toolbar_button(
                ui,
                &toolbar_label(t.tb_find, t.tb_label_find, compact),
                t.tip_find,
                true,
            ) {
                app.open_find(false);
            }
            if toolbar_button(
                ui,
                &toolbar_label(t.tb_compare, t.tb_label_compare, compact),
                t.tip_compare,
                true,
            ) {
                app.open_compare_window();
            }
            let encoding_label = toolbar_label(t.tb_encoding, t.tb_label_encoding, compact);
            ui.menu_button(encoding_label, |ui| {
                let enc_action = encoding_menu::show_menu(ui, app);
                crate::ui::menu_actions::apply_encoding_action(app, enc_action);
            })
            .response
            .on_hover_text(t.tip_encoding);
            ui.separator();
            if ui.button("A-").on_hover_text(t.tip_font_dec).clicked() {
                app.config.editor.font_size = (app.config.editor.font_size - 1.0).max(8.0);
                app.toolbar_font_size_text =
                    format!("{}", app.config.editor.font_size as u32);
                app.toolbar_font_size_editing = false;
            }
            show_font_size_input(
                ui,
                &mut app.config.editor.font_size,
                &mut app.toolbar_font_size_text,
                &mut app.toolbar_font_size_editing,
                &mut app.editor_has_focus,
                t.tip_font_size,
            );
            if ui.button("A+").on_hover_text(t.tip_font_inc).clicked() {
                app.config.editor.font_size = (app.config.editor.font_size + 1.0).min(72.0);
                app.toolbar_font_size_text =
                    format!("{}", app.config.editor.font_size as u32);
                app.toolbar_font_size_editing = false;
            }
        });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_font_size_accepts_digits_only() {
        assert_eq!(parse_font_size("16"), Some(16));
        assert_eq!(parse_font_size("16px"), Some(16));
        assert_eq!(parse_font_size(""), None);
    }

    #[test]
    fn apply_font_size_clamps_and_updates_buffer() {
        let mut text = "99".to_string();
        let mut size = 16.0;
        apply_font_size(&mut text, &mut size);
        assert_eq!(size, 72.0);
        assert_eq!(text, "72");

        text = "abc".to_string();
        size = 20.0;
        apply_font_size(&mut text, &mut size);
        assert_eq!(size, 20.0);
        assert_eq!(text, "20");
    }

    #[test]
    fn toolbar_label_compact_vs_full() {
        assert_eq!(toolbar_label("📄", "New", true), "📄");
        assert_eq!(toolbar_label("📄", "New", false), "📄 New");
    }
}
