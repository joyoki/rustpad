use eframe::egui;

use crate::app::RustpadApp;
use crate::i18n::UiLanguage;
use crate::ui::keybindings::{Command, KeyBinding, KeyBindings, KeyScheme};

/// Keyboard shortcuts management dialog.
pub fn show(app: &mut RustpadApp, ctx: &egui::Context) {
    if !app.show_keybindings {
        return;
    }

    let t = app.tr();
    let zh = UiLanguage::parse(&app.config.ui.ui_language) == UiLanguage::Zh;
    let mut open = app.show_keybindings;
    let mut should_save = false;
    let mut should_reset = false;
    let mut should_cancel = false;
    let mut scheme_change: Option<KeyScheme> = None;
    let mut start_recording: Option<Command> = None;

    egui::Window::new(t.dlg_keybindings)
        .default_size([520.0, 480.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(t.kb_scheme);
                if ui
                    .selectable_label(app.keybindings_edit.scheme == KeyScheme::NotepadPP, "Notepad++")
                    .clicked()
                {
                    scheme_change = Some(KeyScheme::NotepadPP);
                }
                if ui
                    .selectable_label(app.keybindings_edit.scheme == KeyScheme::VSCode, "VS Code")
                    .clicked()
                {
                    scheme_change = Some(KeyScheme::VSCode);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t.kb_reset).clicked() {
                        should_reset = true;
                    }
                });
            });

            if let Some(cmd) = app.keybindings_recording {
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "{}: {}",
                        cmd.label(zh),
                        t.kb_press_key
                    ))
                    .strong()
                    .color(egui::Color32::from_rgb(0, 100, 180)),
                );
            }

            let conflicts = app.keybindings_edit.find_conflicts();
            if !conflicts.is_empty() {
                ui.colored_label(egui::Color32::from_rgb(180, 60, 0), t.kb_conflicts);
                for (a, b, binding) in &conflicts {
                    ui.label(format!(
                        "• {} / {} → {}",
                        a.label(zh),
                        b.label(zh),
                        binding.display()
                    ));
                }
            }

            if !app.keybindings_status.is_empty() {
                ui.label(
                    egui::RichText::new(&app.keybindings_status)
                        .italics()
                        .color(egui::Color32::from_rgb(0, 120, 60)),
                );
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.strong(t.kb_command_col);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.strong(t.kb_shortcut_col);
                });
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for &command in Command::all() {
                        ui.horizontal(|ui| {
                            ui.label(command.label(zh));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let recording = app.keybindings_recording == Some(command);
                                let label = if recording {
                                    t.kb_press_key.to_string()
                                } else {
                                    app.keybindings_edit.primary_display(&command)
                                };
                                if ui
                                    .add_enabled(
                                        app.keybindings_recording.is_none(),
                                        egui::Button::new(label),
                                    )
                                    .clicked()
                                {
                                    start_recording = Some(command);
                                }
                            });
                        });
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(t.btn_save).clicked() {
                    should_save = true;
                }
                if ui.button(t.btn_cancel).clicked() {
                    should_cancel = true;
                }
            });
        });

    if should_cancel {
        open = false;
        app.keybindings_recording = None;
        app.keybindings_status.clear();
    }
    if let Some(scheme) = scheme_change {
        app.keybindings_edit.apply_scheme(scheme);
        app.keybindings_status.clear();
    }
    if should_reset {
        app.keybindings_edit = KeyBindings::default();
        app.keybindings_recording = None;
        app.keybindings_status.clear();
    }
    if let Some(cmd) = start_recording {
        app.keybindings_recording = Some(cmd);
        app.keybindings_status.clear();
    }

    if app.keybindings_recording.is_some() {
        if let Some(binding) = ctx.input(KeyBinding::capture_from_input) {
            if let Some(cmd) = app.keybindings_recording.take() {
                app.keybindings_edit.set_binding(cmd, binding);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            app.keybindings_recording = None;
        }
    }

    if should_save {
        let _ = app.keybindings_edit.save();
        app.keybindings = app.keybindings_edit.clone();
        app.keybindings_status = app.tr().kb_saved.to_string();
        open = false;
        app.keybindings_recording = None;
    }

    app.show_keybindings = open;
}

pub fn open_editor(app: &mut RustpadApp) {
    app.keybindings_edit = app.keybindings.clone();
    app.keybindings_recording = None;
    app.keybindings_status.clear();
    app.show_keybindings = true;
}
