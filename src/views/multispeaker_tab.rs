use iced::widget::{button, column, pick_list, progress_bar, row, text, text_input};
use iced::{Element, Length};

use crate::api::types::{ReferenceAudio, TaskStatus};
use crate::audio::player::PlaybackState;
use crate::message::{ActiveTask, Message};

/// A single segment in the multi-speaker list.
#[derive(Debug, Clone)]
pub struct SegmentState {
    pub selected_ref: Option<String>,
    pub text: String,
    pub selected_language: String,
}

impl Default for SegmentState {
    fn default() -> Self {
        Self {
            selected_ref: None,
            text: String::new(),
            selected_language: "auto".to_owned(),
        }
    }
}

/// State specific to the Multi-Speaker tab.
#[derive(Debug, Clone)]
pub struct MultiSpeakerTabState {
    pub segments: Vec<SegmentState>,
}

impl Default for MultiSpeakerTabState {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiSpeakerTabState {
    pub fn new() -> Self {
        Self {
            segments: vec![SegmentState::default(), SegmentState::default()],
        }
    }
}

// LCOV_EXCL_START

/// Build the Multi-Speaker tab view.
pub fn view<'a>(
    state: &'a MultiSpeakerTabState,
    references: &'a [ReferenceAudio],
    languages: &'a [String],
    active_task: Option<&'a ActiveTask>,
    playback: PlaybackState,
    model_available: bool,
) -> Element<'a, Message> {
    let ref_names: Vec<String> = references
        .iter()
        .map(|r| {
            r.name
                .clone()
                .unwrap_or_else(|| r.original_name.clone())
        })
        .collect();

    let mut content = column![text("Multi-Speaker").size(24),]
        .spacing(8)
        .padding(20)
        .width(Length::Fill);

    for (i, segment) in state.segments.iter().enumerate() {
        let segment_col = segment_view(i, segment, &ref_names, languages, state.segments.len());
        content = content.push(segment_col);
    }

    let add_btn = button(text("+ Add Segment")).on_press(Message::MultiAddSegment);
    content = content.push(add_btn);

    let is_generating = active_task
        .as_ref()
        .is_some_and(|t| t.status == TaskStatus::Processing);
    let can_generate = state.segments.iter().all(|s| {
        !s.text.is_empty() && s.selected_ref.is_some()
    }) && !state.segments.is_empty()
        && !is_generating
        && model_available;

    let mut generate_btn = button(text("Generate"));
    if can_generate {
        generate_btn = generate_btn.on_press(Message::MultiGenerate);
    }
    content = content.push(row![generate_btn].spacing(8));

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

/// Build the view for a single segment.
fn segment_view<'a>(
    index: usize,
    segment: &'a SegmentState,
    ref_names: &[String],
    languages: &[String],
    total_segments: usize,
) -> Element<'a, Message> {
    let ref_picker = pick_list(
        ref_names.to_vec(),
        segment.selected_ref.clone(),
        move |name| Message::MultiRefSelected(index, name),
    )
    .placeholder("Select reference audio...");

    let lang_picker = pick_list(
        languages.to_vec(),
        Some(segment.selected_language.clone()),
        move |lang| Message::MultiLanguageSelected(index, lang),
    )
    .placeholder("Language");

    let text_field = text_input("Enter text for this segment...", &segment.text)
        .on_input(move |t| Message::MultiTextChanged(index, t))
        .width(Length::Fill);

    let header_text = format!("Segment {}", index + 1);

    let mut header_row = row![text(header_text).size(16)].spacing(8);
    if total_segments > 1 {
        header_row = header_row
            .push(button(text("Remove")).on_press(Message::MultiRemoveSegment(index)));
    }

    column![
        header_row,
        row![ref_picker, lang_picker].spacing(8),
        text_field,
    ]
    .spacing(4)
    .into()
}

// LCOV_EXCL_STOP
