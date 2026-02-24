use iced::widget::{button, column, pick_list, progress_bar, row, text, text_input};
use iced::{Element, Length};

use crate::api::types::{ReferenceAudio, TaskStatus};
use crate::audio::player::PlaybackState;
use crate::message::{ActiveTask, Message};

/// State specific to the Voice Clone tab.
#[derive(Debug, Clone, Default)]
pub struct CloneTabState {
    pub text: String,
    pub selected_ref: Option<String>,
    pub selected_language: String,
}

impl CloneTabState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            selected_ref: None,
            selected_language: "auto".to_owned(),
        }
    }
}

// LCOV_EXCL_START

/// Build the Voice Clone tab view.
pub fn view<'a>(
    state: &'a CloneTabState,
    references: &'a [ReferenceAudio],
    languages: &'a [String],
    active_task: Option<&'a ActiveTask>,
    playback: PlaybackState,
) -> Element<'a, Message> {
    let ref_names: Vec<String> = references
        .iter()
        .map(|r| {
            r.name
                .clone()
                .unwrap_or_else(|| r.original_name.clone())
        })
        .collect();

    let ref_picker = pick_list(
        ref_names,
        state.selected_ref.clone(),
        Message::CloneRefSelected,
    )
    .placeholder("Select reference audio...");

    // Preview button for the selected reference audio
    let mut ref_row = row![ref_picker].spacing(8);
    if let Some(ref_name) = &state.selected_ref {
        // Find the reference audio ID for the selected name
        let ref_audio = references
            .iter()
            .find(|r| r.name.as_deref().unwrap_or(&r.original_name) == ref_name.as_str());
        if let Some(audio) = ref_audio {
            let mut preview_btn = button(text("Preview"));
            if playback == PlaybackState::Stopped {
                preview_btn =
                    preview_btn.on_press(Message::PlayReference(audio.id.clone()));
            }
            ref_row = ref_row.push(preview_btn);
        }
    }

    let lang_picker = pick_list(
        languages.to_vec(),
        Some(state.selected_language.clone()),
        Message::CloneLanguageSelected,
    )
    .placeholder("Language");

    let text_field = text_input("Enter text to generate...", &state.text)
        .on_input(Message::CloneTextChanged)
        .width(Length::Fill);

    let can_generate =
        !state.text.is_empty() && state.selected_ref.is_some() && active_task.is_none();

    let mut generate_btn = button(text("Generate"));
    if can_generate {
        generate_btn = generate_btn.on_press(Message::CloneGenerate);
    }

    let mut content = column![
        text("Voice Clone").size(24),
        text("Reference Audio").size(14),
        ref_row,
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

        // Playback controls for completed task with audio data
        if task.status == TaskStatus::Completed && task.audio_data.is_some() {
            content = content.push(playback_controls(playback));
        }
    } else if playback != PlaybackState::Stopped {
        // Playback controls for reference preview (no active task)
        content = content.push(playback_controls(playback));
    }

    content.into()
}

/// Render play/pause/stop buttons based on current playback state.
pub fn playback_controls(playback: PlaybackState) -> Element<'static, Message> {
    let mut controls = row![].spacing(8);

    match playback {
        PlaybackState::Stopped => {
            controls = controls.push(button(text("Play")).on_press(Message::PlayGenerated));
        }
        PlaybackState::Playing => {
            controls = controls.push(button(text("Pause")).on_press(Message::PlaybackPause));
            controls = controls.push(button(text("Stop")).on_press(Message::PlaybackStop));
        }
        PlaybackState::Paused => {
            controls = controls.push(button(text("Resume")).on_press(Message::PlaybackResume));
            controls = controls.push(button(text("Stop")).on_press(Message::PlaybackStop));
        }
    }

    controls.into()
}

// LCOV_EXCL_STOP
