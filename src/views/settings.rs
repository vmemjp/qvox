use iced::widget::{button, checkbox, column, row, text, text_input};
use iced::{Element, Length};

use crate::config::AppConfig;
use crate::message::Message;

// LCOV_EXCL_START

/// Build the settings view.
pub fn view(config: &AppConfig, dirty: bool) -> Element<'_, Message> {
    let models = &config.server.models;
    let base_check = checkbox(models.contains(&"base".to_owned()))
        .label("base")
        .on_toggle(|_| Message::SettingsModelToggled("base".to_owned()));
    let design_check = checkbox(models.contains(&"voice_design".to_owned()))
        .label("voice_design")
        .on_toggle(|_| Message::SettingsModelToggled("voice_design".to_owned()));
    let custom_check = checkbox(models.contains(&"custom_voice".to_owned()))
        .label("custom_voice")
        .on_toggle(|_| Message::SettingsModelToggled("custom_voice".to_owned()));
    let models_row = row![base_check, design_check, custom_check].spacing(16);

    let device_field = text_input("auto", &config.server.device)
        .on_input(Message::SettingsDeviceChanged)
        .width(Length::Fixed(200.0));

    let port_field = text_input("8000", &config.server.port.to_string())
        .on_input(Message::SettingsPortChanged)
        .width(Length::Fixed(100.0));

    let script_field = text_input("python/start_server.py", &config.server.script_path)
        .on_input(Message::SettingsScriptPathChanged)
        .width(Length::Fill);

    let dark_mode_toggle = checkbox(config.ui.dark_mode)
        .label("Dark Mode")
        .on_toggle(Message::SettingsDarkModeToggled);

    let mut save_btn = button(text("Save & Restart"));
    if dirty {
        save_btn = save_btn.on_press(Message::SettingsSave);
    }

    column![
        text("Settings").size(24),
        text("Models").size(14),
        models_row,
        row![
            column![text("Device").size(14), device_field].spacing(4),
            column![text("Port").size(14), port_field].spacing(4),
        ]
        .spacing(16),
        text("Server Script Path").size(14),
        script_field,
        dark_mode_toggle,
        row![save_btn].spacing(8),
    ]
    .spacing(8)
    .padding(20)
    .width(Length::Fill)
    .into()
}

// LCOV_EXCL_STOP
