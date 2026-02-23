use iced::widget::{button, column, row, scrollable, text};
use iced::Element;

use crate::api::types::GeneratedAudio;
use crate::message::Message;

// LCOV_EXCL_START

/// Build the generated audio list view.
pub fn view(items: &[GeneratedAudio]) -> Element<'_, Message> {
    if items.is_empty() {
        return column![].into();
    }

    let mut list = column![
        row![
            text("Generated Audio").size(18),
            button(text("Refresh")).on_press(Message::RefreshGeneratedList),
        ]
        .spacing(8),
    ]
    .spacing(4);

    for item in items {
        list = list.push(item_row(item));
    }

    scrollable(list).into()
}

/// Render a single generated audio item.
fn item_row(item: &GeneratedAudio) -> Element<'_, Message> {
    let label = item
        .ref_audio_name
        .as_deref()
        .unwrap_or("Unknown source");

    let truncated_text = if item.generated_text.len() > 60 {
        format!("{}...", &item.generated_text[..60])
    } else {
        item.generated_text.clone()
    };

    let time_text = item
        .generation_time_seconds
        .map_or(String::new(), |t| format!("{t:.1}s"));

    let play_btn = button(text("Play")).on_press(Message::GeneratedPlay(item.id.clone()));
    let delete_btn = button(text("Delete")).on_press(Message::GeneratedDelete(item.id.clone()));

    row![
        column![
            text(label).size(13),
            text(truncated_text).size(11),
        ]
        .spacing(2)
        .width(iced::Length::Fill),
        text(time_text).size(11),
        play_btn,
        delete_btn,
    ]
    .spacing(8)
    .into()
}

// LCOV_EXCL_STOP
