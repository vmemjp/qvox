use std::path::PathBuf;

use iced::widget::{button, column, pick_list, progress_bar, row, text, text_input};
use iced::{Element, Length};

use crate::api::types::TaskStatus;
use crate::audio::player::PlaybackState;
use crate::message::{ActiveTask, Message};

/// State specific to the Upload & Clone tab.
#[derive(Debug, Clone, Default)]
pub struct UploadTabState {
    pub selected_file: Option<PathBuf>,
    pub file_bytes: Option<Vec<u8>>,
    pub file_name: Option<String>,
    pub file_hash: Option<String>,
    pub text: String,
    pub selected_language: String,
}

impl UploadTabState {
    pub fn new() -> Self {
        Self {
            selected_file: None,
            file_bytes: None,
            file_name: None,
            file_hash: None,
            text: String::new(),
            selected_language: "auto".to_owned(),
        }
    }
}

// LCOV_EXCL_START

/// Build the Upload & Clone tab view.
pub fn view<'a>(
    state: &'a UploadTabState,
    languages: &'a [String],
    active_task: Option<&'a ActiveTask>,
    playback: PlaybackState,
) -> Element<'a, Message> {
    let file_label = state
        .file_name
        .as_deref()
        .unwrap_or("No file selected");

    let choose_btn = button(text("Choose File")).on_press(Message::UploadPickFile);

    let lang_picker = pick_list(
        languages.to_vec(),
        Some(state.selected_language.clone()),
        Message::UploadLanguageSelected,
    )
    .placeholder("Language");

    let text_field = text_input("Enter text to generate...", &state.text)
        .on_input(Message::UploadTextChanged)
        .width(Length::Fill);

    let can_generate =
        !state.text.is_empty() && state.file_bytes.is_some() && active_task.is_none();

    let mut generate_btn = button(text("Generate"));
    if can_generate {
        generate_btn = generate_btn.on_press(Message::UploadGenerate);
    }

    let mut file_info = row![choose_btn, text(file_label).size(14)].spacing(8);

    if let Some(hash) = &state.file_hash {
        file_info = file_info.push(text(format!("SHA256: {}...", &hash[..8])).size(10));
    }

    let mut content = column![
        text("Upload & Clone").size(24),
        text("Audio File").size(14),
        file_info,
        text("Language").size(14),
        lang_picker,
        text("Text").size(14),
        text_field,
        row![generate_btn].spacing(8),
    ]
    .spacing(8)
    .padding(20)
    .width(Length::Fill);

    // Progress section
    if let Some(task) = active_task {
        #[allow(clippy::cast_precision_loss)]
        let progress_value = task.progress as f32;
        content = content
            .push(progress_bar(0.0..=100.0, progress_value))
            .push(text(&task.status_text).size(14))
            .push(
                text(format!(
                    "Elapsed: {:02}:{:02}",
                    task.elapsed_secs / 60,
                    task.elapsed_secs % 60
                ))
                .size(12),
            );

        if let Some(err) = &task.error {
            content = content.push(text(err).size(14));
        }

        if task.status == TaskStatus::Completed && task.audio_data.is_some() {
            content = content.push(super::clone_tab::playback_controls(playback));
        }
    }

    if playback != PlaybackState::Stopped {
        content = content.push(super::clone_tab::playback_controls(playback));
    }

    content.into()
}

// LCOV_EXCL_STOP
