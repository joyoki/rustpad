//! Encoding submenu — open with / convert to encoding.

use eframe::egui;

use crate::app::RustpadApp;
use crate::editor::EncodingProfile;

/// Render the encoding dropdown; returns actions to apply after the menu closes.
pub fn show_menu(ui: &mut egui::Ui, app: &RustpadApp) -> EncodingMenuAction {
    let t = app.tr();
    let current = app.tab_manager.active().encoding;
    let has_file = app.tab_manager.active().file_path.is_some();
    let mut action = EncodingMenuAction::None;
    let mut more_open: Option<EncodingProfile> = None;
    let mut more_convert: Option<EncodingProfile> = None;

    ui.label(egui::RichText::new(t.enc_open_section).strong());
    for profile in EncodingProfile::MAIN {
        let label = t.enc_open_with(profile);
        let selected = current == profile;
        if ui
            .add_enabled(has_file, egui::SelectableLabel::new(selected, label))
            .clicked()
        {
            action = EncodingMenuAction::OpenWith(profile);
            ui.close_menu();
        }
    }
    ui.menu_button(t.enc_more, |ui| {
        for profile in EncodingProfile::MORE {
            let label = t.enc_open_with(profile);
            let selected = current == profile;
            if ui
                .add_enabled(has_file, egui::SelectableLabel::new(selected, label))
                .clicked()
            {
                more_open = Some(profile);
                ui.close_menu();
            }
        }
    });
    if let Some(profile) = more_open {
        action = EncodingMenuAction::OpenWith(profile);
    }

    ui.separator();
    ui.label(egui::RichText::new(t.enc_convert_section).strong());
    for profile in EncodingProfile::MAIN {
        let label = t.enc_convert_to(profile);
        if ui.button(label).clicked() {
            action = EncodingMenuAction::ConvertTo(profile);
            ui.close_menu();
        }
    }
    ui.menu_button(t.enc_convert_more, |ui| {
        for profile in EncodingProfile::MORE {
            let label = t.enc_convert_to(profile);
            if ui.button(label).clicked() {
                more_convert = Some(profile);
                ui.close_menu();
            }
        }
    });
    if let Some(profile) = more_convert {
        action = EncodingMenuAction::ConvertTo(profile);
    }
    if ui
        .add_enabled(has_file, egui::Button::new(t.enc_batch_convert))
        .clicked()
    {
        action = EncodingMenuAction::BatchConvert;
        ui.close_menu();
    }

    action
}

/// Apply a menu action after the popup closes.
pub fn apply_action(app: &mut RustpadApp, action: EncodingMenuAction) {
    match action {
        EncodingMenuAction::OpenWith(profile) => app.open_with_encoding(profile),
        EncodingMenuAction::ConvertTo(profile) => app.convert_to_encoding(profile),
        EncodingMenuAction::BatchConvert => app.show_batch_encoding = true,
        EncodingMenuAction::None => {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingMenuAction {
    None,
    OpenWith(EncodingProfile),
    ConvertTo(EncodingProfile),
    BatchConvert,
}
