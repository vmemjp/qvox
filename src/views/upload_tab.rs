use std::path::PathBuf;

use iced::widget::{button, column, pick_list, progress_bar, row, text, text_input};
use iced::{Element, Length};

use crate::api::types::TaskStatus;
use crate::audio::player::PlaybackState;
use crate::audio::recorder::RecordingState;
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
    pub ref_text: Option<String>,
    pub transcribing: bool,
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
            ref_text: None,
            transcribing: false,
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
    recording: RecordingState,
    recording_elapsed: f32,
) -> Element<'a, Message> {
    let file_label = state
        .file_name
        .as_deref()
        .unwrap_or("No file selected");

    let choose_btn = button(text("Choose File")).on_press(Message::UploadPickFile);

    // Record button
    let record_btn = match recording {
        RecordingState::Idle => button(text("Record")).on_press(Message::RecordStart),
        RecordingState::Recording => button(text("Stop Recording")).on_press(Message::RecordStop),
    };

    let lang_picker = pick_list(
        languages.to_vec(),
        Some(state.selected_language.clone()),
        Message::UploadLanguageSelected,
    )
    .placeholder("Language");

    let text_field = text_input("Enter text to generate...", &state.text)
        .on_input(Message::UploadTextChanged)
        .width(Length::Fill);

    let can_generate = !state.text.is_empty()
        && state.file_bytes.is_some()
        && active_task.is_none()
        && !state.transcribing
        && recording == RecordingState::Idle;

    let mut generate_btn = button(text("Generate"));
    if can_generate {
        generate_btn = generate_btn.on_press(Message::UploadGenerate);
    }

    let mut file_row = row![choose_btn, record_btn, text(file_label).size(14)].spacing(8);

    if let Some(hash) = &state.file_hash {
        file_row = file_row.push(text(format!("SHA256: {}...", &hash[..8])).size(10));
    }

    let mut content = column![
        text("Upload & Clone").size(24),
        text("Audio File").size(14),
        file_row,
    ]
    .spacing(8)
    .padding(20)
    .width(Length::Fill);

    // Recording elapsed time
    if recording == RecordingState::Recording {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let secs = recording_elapsed as u64;
        content = content.push(
            text(format!("Recording... {:02}:{:02}", secs / 60, secs % 60)).size(12),
        );
    }

    // Transcription status
    if state.transcribing {
        content = content.push(text("Transcribing audio...").size(12));
    } else if let Some(ref_text) = &state.ref_text {
        content = content.push(
            text(format!("Transcription: {}", truncate_text(ref_text, 80))).size(12),
        );
    }

    content = content
        .push(text("Language").size(14))
        .push(lang_picker)
        .push(text("Text").size(14))
        .push(text_field)
        .push(row![generate_btn].spacing(8));

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

/// Truncate text for display, adding ellipsis if needed.
fn truncate_text(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_owned()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{truncated}...")
    }
}

// LCOV_EXCL_STOP
