use iced::widget::{button, column, pick_list, progress_bar, row, text, text_input};
use iced::{Element, Length};

use crate::api::types::TaskStatus;
use crate::audio::player::PlaybackState;
use crate::message::{ActiveTask, Message};

/// State specific to the Voice Design tab.
#[derive(Debug, Clone, Default)]
pub struct DesignTabState {
    pub text: String,
    pub instruct: String,
    pub selected_language: String,
}

impl DesignTabState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            instruct: String::new(),
            selected_language: "auto".to_owned(),
        }
    }
}

// LCOV_EXCL_START

/// Build the Voice Design tab view.
pub fn view<'a>(
    state: &'a DesignTabState,
    languages: &'a [String],
    active_task: Option<&'a ActiveTask>,
    playback: PlaybackState,
) -> Element<'a, Message> {
    let instruct_field =
        text_input("Describe the voice (e.g. \"A warm, friendly female voice\")", &state.instruct)
            .on_input(Message::DesignInstructChanged)
            .width(Length::Fill);

    let lang_picker = pick_list(
        languages.to_vec(),
        Some(state.selected_language.clone()),
        Message::DesignLanguageSelected,
    )
    .placeholder("Language");

    let text_field = text_input("Enter text to generate...", &state.text)
        .on_input(Message::DesignTextChanged)
        .width(Length::Fill);

    let can_generate =
        !state.text.is_empty() && !state.instruct.is_empty() && active_task.is_none();

    let mut generate_btn = button(text("Generate"));
    if can_generate {
        generate_btn = generate_btn.on_press(Message::DesignGenerate);
    }

    let mut content = column![
        text("Voice Design").size(24),
        text("Voice Description").size(14),
        instruct_field,
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
