use iced::widget::{button, checkbox, column, row, text, text_input};
use iced::{Element, Length};

use crate::config::AppConfig;
use crate::message::Message;

// LCOV_EXCL_START

/// Build the settings view.
pub fn view(config: &AppConfig, dirty: bool) -> Element<'_, Message> {
    let models_str = config.server.models.join(", ");

    let models_field = text_input("base, voice_design, custom_voice", &models_str)
        .on_input(Message::SettingsModelsChanged)
        .width(Length::Fill);

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
        text("Models (comma-separated)").size(14),
        models_field,
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
