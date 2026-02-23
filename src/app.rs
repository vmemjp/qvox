use iced::widget::text;
use iced::{Element, Task};

use crate::message::Message;

#[derive(Debug, Default)]
pub struct Qvox;

impl Qvox {
    pub fn new() -> (Self, Task<Message>) {
        (Self, Task::none())
    }

    #[allow(clippy::unused_self)]
    pub fn title(&self) -> String {
        String::from("qvox")
    }

    #[allow(clippy::unused_self)]
    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    #[allow(clippy::unused_self)]
    pub fn view(&self) -> Element<'_, Message> {
        text("").into()
    }
}
