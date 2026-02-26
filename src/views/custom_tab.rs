use iced::widget::{button, column, pick_list, progress_bar, row, text, text_input};
use iced::{Element, Length};

use crate::api::types::TaskStatus;
use crate::audio::player::PlaybackState;
use crate::message::{ActiveTask, Message};

/// State specific to the Custom Voice tab.
#[derive(Debug, Clone, Default)]
pub struct CustomTabState {
    pub text: String,
    pub selected_speaker: Option<String>,
    pub selected_language: String,
    pub instruct: String,
}

impl CustomTabState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            selected_speaker: None,
            selected_language: "auto".to_owned(),
            instruct: String::new(),
        }
    }
}

// LCOV_EXCL_START

/// Build the Custom Voice tab view.
pub fn view<'a>(
    state: &'a CustomTabState,
    speakers: &'a [String],
    languages: &'a [String],
    active_task: Option<&'a ActiveTask>,
    playback: PlaybackState,
    model_available: bool,
) -> Element<'a, Message> {
    let speaker_picker = pick_list(
        speakers.to_vec(),
        state.selected_speaker.clone(),
        Message::CustomSpeakerSelected,
    )
    .placeholder("Select speaker...");

    let lang_picker = pick_list(
        languages.to_vec(),
        Some(state.selected_language.clone()),
        Message::CustomLanguageSelected,
    )
    .placeholder("Language");

    let text_field = text_input("Enter text to generate...", &state.text)
        .on_input(Message::CustomTextChanged)
        .width(Length::Fill);

    let instruct_field =
        text_input("Style instructions (optional, e.g. \"Speak slowly and calmly\")", &state.instruct)
            .on_input(Message::CustomInstructChanged)
            .width(Length::Fill);

    let is_generating = active_task
        .as_ref()
        .is_some_and(|t| t.status == TaskStatus::Processing);
    let can_generate =
        !state.text.is_empty() && state.selected_speaker.is_some() && !is_generating && model_available;

    let mut generate_btn = button(text("Generate"));
    if can_generate {
        generate_btn = generate_btn.on_press(Message::CustomGenerate);
    }

    let mut content = column![
        text("Custom Voice").size(24),
        text("Speaker").size(14),
        speaker_picker,
        text("Language").size(14),
        lang_picker,
        text("Text").size(14),
        text_field,
        text("Style Instructions").size(14),
        instruct_field,
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
    } else if playback != PlaybackState::Stopped {
        content = content.push(super::clone_tab::playback_controls(playback));
    }

    content.into()
}

// LCOV_EXCL_STOP
