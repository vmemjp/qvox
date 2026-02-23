use iced::widget::{button, column, pick_list, progress_bar, row, text, text_input};
use iced::{Element, Length};

use crate::api::types::ReferenceAudio;
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
        ref_picker,
        text("Language").size(14),
        lang_picker,
        text("Text").size(14),
        text_field,
        row![generate_btn].spacing(8),
    ]
    .spacing(8)
    .padding(20)
    .width(Length::Fill);

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
    }

    content.into()
}

// LCOV_EXCL_STOP
