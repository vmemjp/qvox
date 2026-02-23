use std::time::Duration;

use iced::widget::{center, column, container, progress_bar, text};
use iced::{Element, Length, Subscription, Task};

use crate::api::client::ApiClient;
use crate::message::Message;
use crate::server::manager::{ServerConfig, ServerManager};

// ─── Screen state ───────────────────────────────────────────────

#[derive(Debug, Default)]
enum Screen {
    #[default]
    Loading,
    Main,
}

// ─── Application state ─────────────────────────────────────────

#[derive(Debug)]
pub struct Qvox {
    screen: Screen,
    server: Option<ServerManager>,
    config: ServerConfig,
    elapsed_secs: u64,
    loading_status: String,
    error: Option<String>,
}

impl Default for Qvox {
    fn default() -> Self {
        Self {
            screen: Screen::Loading,
            server: None,
            config: ServerConfig::default(),
            elapsed_secs: 0,
            loading_status: "Starting server...".to_owned(),
            error: None,
        }
    }
}

impl Qvox {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self::default();
        (app, Task::done(Message::ServerSpawned))
    }

    #[allow(clippy::unused_self)]
    pub fn title(&self) -> String {
        String::from("qvox")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServerSpawned => {
                match ServerManager::spawn(&self.config) {
                    Ok(mgr) => {
                        self.server = Some(mgr);
                        "Waiting for server...".clone_into(&mut self.loading_status);
                        self.poll_health()
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                        self.loading_status = format!("Error: {e}");
                        Task::none()
                    }
                }
            }
            Message::HealthCheck(ready) => {
                if ready {
                    Task::done(Message::ServerReady)
                } else {
                    self.loading_status = format!(
                        "Loading model... ({:02}:{:02})",
                        self.elapsed_secs / 60,
                        self.elapsed_secs % 60
                    );
                    Task::none()
                }
            }
            Message::ServerReady => {
                self.screen = Screen::Main;
                "Ready".clone_into(&mut self.loading_status);
                Task::none()
            }
            Message::ServerError(e) => {
                self.error = Some(e.clone());
                self.loading_status = format!("Error: {e}");
                Task::none()
            }
            Message::Tick => {
                self.elapsed_secs += 1;
                if self.server.is_some() && self.error.is_none() {
                    self.poll_health()
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Loading => self.view_loading(),
            Screen::Main => self.view_main(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match &self.screen {
            Screen::Loading if self.error.is_none() => {
                iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
            }
            _ => Subscription::none(),
        }
    }

    // ─── Private helpers ────────────────────────────────────────

    fn poll_health(&self) -> Task<Message> {
        let base_url = self
            .server
            .as_ref()
            .map_or_else(|| "http://localhost:8000".to_owned(), ServerManager::base_url);

        Task::perform(
            async move {
                let client = ApiClient::new(&base_url);
                match client.health().await {
                    Ok(resp) => resp.voice_cloner_loaded,
                    Err(_) => false,
                }
            },
            Message::HealthCheck,
        )
    }

    // LCOV_EXCL_START
    fn view_loading(&self) -> Element<'_, Message> {
        let title = text("qvox").size(32);
        let status = text(&self.loading_status).size(16);

        let elapsed = text(format!(
            "{:02}:{:02}",
            self.elapsed_secs / 60,
            self.elapsed_secs % 60
        ))
        .size(14);

        let models_text = text(format!("Models: {}", self.config.models.join(", "))).size(12);
        let device_text = text(format!("Device: {}", self.config.device)).size(12);

        let mut col = column![title, progress_bar(0.0..=100.0, 0.0), status, elapsed,]
            .spacing(12)
            .padding(40)
            .width(Length::Fixed(400.0))
            .align_x(iced::Alignment::Center);

        col = col.push(models_text).push(device_text);

        if let Some(err) = &self.error {
            col = col.push(text(err).size(14));
        }

        center(container(col).center_x(Length::Fill)).into()
    }

    #[allow(clippy::unused_self)]
    fn view_main(&self) -> Element<'_, Message> {
        center(text("qvox — Ready").size(24)).into()
    }
    // LCOV_EXCL_STOP
}
