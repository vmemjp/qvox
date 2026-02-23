use std::time::Duration;

use iced::widget::{center, column, container, progress_bar, text};
use iced::{Element, Length, Subscription, Task};

use crate::api::client::ApiClient;
use crate::api::types::{CloneRequest, ReferenceAudio, TaskStatus};
use crate::message::{ActiveTask, Message, TabId};
use crate::server::manager::{ServerConfig, ServerManager};
use crate::views::clone_tab::CloneTabState;

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

    // ─── Main screen state ──────────────────────────────────
    active_tab: TabId,
    references: Vec<ReferenceAudio>,
    languages: Vec<String>,
    available_models: Vec<String>,

    // ─── Clone tab ──────────────────────────────────────────
    clone_tab: CloneTabState,
    active_task: Option<ActiveTask>,
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
            active_tab: TabId::Clone,
            references: Vec::new(),
            languages: vec!["auto".to_owned()],
            available_models: Vec::new(),
            clone_tab: CloneTabState::new(),
            active_task: None,
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
            // ─── Server lifecycle ───────────────────────────
            Message::ServerSpawned
            | Message::HealthCheck(_)
            | Message::ServerReady
            | Message::ServerError(_)
            | Message::Tick => self.update_server(message),

            // ─── Data loading ───────────────────────────────
            Message::CapabilitiesLoaded(_)
            | Message::ReferencesLoaded(_)
            | Message::LanguagesLoaded(_) => self.update_data(message),

            // ─── Tab navigation ─────────────────────────────
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                Task::none()
            }

            // ─── Clone tab inputs ───────────────────────────
            Message::CloneTextChanged(_)
            | Message::CloneRefSelected(_)
            | Message::CloneLanguageSelected(_)
            | Message::CloneGenerate => self.update_clone(message),

            // ─── Task lifecycle ─────────────────────────────
            Message::TaskCreated(_)
            | Message::TaskPollTick
            | Message::TaskProgress(_)
            | Message::TaskAudioLoaded(_) => self.update_task(message),

            // ─── Generated list ─────────────────────────────
            Message::GeneratedListLoaded(_) => Task::none(),
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
            Screen::Main
                if self
                    .active_task
                    .as_ref()
                    .is_some_and(|t| t.status == TaskStatus::Processing) =>
            {
                iced::time::every(Duration::from_secs(1)).map(|_| Message::TaskPollTick)
            }
            _ => Subscription::none(),
        }
    }

    // ─── Update sub-handlers ────────────────────────────────────

    fn update_server(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServerSpawned => match ServerManager::spawn(&self.config) {
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
            },
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
                self.load_initial_data()
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
            _ => Task::none(),
        }
    }

    fn update_data(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CapabilitiesLoaded(Ok(caps)) => {
                self.available_models = caps.models;
            }
            Message::ReferencesLoaded(Ok(refs)) => {
                self.references = refs;
            }
            Message::LanguagesLoaded(Ok(langs)) => {
                self.languages = langs.languages;
            }
            _ => {}
        }
        Task::none()
    }

    fn update_clone(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloneTextChanged(t) => {
                self.clone_tab.text = t;
                Task::none()
            }
            Message::CloneRefSelected(name) => {
                self.clone_tab.selected_ref = Some(name);
                Task::none()
            }
            Message::CloneLanguageSelected(lang) => {
                self.clone_tab.selected_language = lang;
                Task::none()
            }
            Message::CloneGenerate => self.start_clone_generation(),
            _ => Task::none(),
        }
    }

    fn update_task(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TaskCreated(result) => {
                match result {
                    Ok(task_id) => self.active_task = Some(ActiveTask::new(task_id)),
                    Err(e) => self.error = Some(e),
                }
                Task::none()
            }
            Message::TaskPollTick => {
                if let Some(task) = &mut self.active_task {
                    task.elapsed_secs += 1;
                }
                self.poll_task()
            }
            Message::TaskProgress(result) => match result {
                Ok(resp) => {
                    if let Some(task) = &mut self.active_task {
                        task.update_progress(&resp);
                        if resp.status == TaskStatus::Completed {
                            return self.fetch_task_audio();
                        }
                    }
                    Task::none()
                }
                Err(e) => {
                    if let Some(task) = &mut self.active_task {
                        task.error = Some(e);
                    }
                    Task::none()
                }
            },
            Message::TaskAudioLoaded(result) => {
                if let Some(task) = &mut self.active_task {
                    match result {
                        Ok(data) => task.audio_data = Some(data),
                        Err(e) => task.error = Some(e),
                    }
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    // ─── Private helpers ────────────────────────────────────────

    fn api_base_url(&self) -> String {
        self.server
            .as_ref()
            .map_or_else(|| "http://localhost:8000".to_owned(), ServerManager::base_url)
    }

    fn poll_health(&self) -> Task<Message> {
        let base_url = self.api_base_url();
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

    fn load_initial_data(&self) -> Task<Message> {
        let url = self.api_base_url();
        let url2 = url.clone();
        let url3 = url.clone();

        Task::batch([
            Task::perform(
                async move {
                    ApiClient::new(&url)
                        .capabilities()
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::CapabilitiesLoaded,
            ),
            Task::perform(
                async move {
                    ApiClient::new(&url2)
                        .references()
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::ReferencesLoaded,
            ),
            Task::perform(
                async move {
                    ApiClient::new(&url3)
                        .languages()
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::LanguagesLoaded,
            ),
        ])
    }

    fn start_clone_generation(&mut self) -> Task<Message> {
        let Some(ref_name) = &self.clone_tab.selected_ref else {
            return Task::none();
        };

        let ref_audio = self
            .references
            .iter()
            .find(|r| r.name.as_deref().unwrap_or(&r.original_name) == ref_name.as_str());

        let Some(ref_audio) = ref_audio else {
            return Task::none();
        };

        let request = CloneRequest {
            text: self.clone_tab.text.clone(),
            ref_audio_id: ref_audio.id.clone(),
            ref_text: ref_audio.ref_text.clone(),
            language: self.clone_tab.selected_language.clone(),
        };

        let base_url = self.api_base_url();

        Task::perform(
            async move {
                ApiClient::new(&base_url)
                    .clone_voice(&request)
                    .await
                    .map(|resp| resp.task_id)
                    .map_err(|e| e.to_string())
            },
            Message::TaskCreated,
        )
    }

    fn poll_task(&self) -> Task<Message> {
        let Some(task) = &self.active_task else {
            return Task::none();
        };

        let base_url = self.api_base_url();
        let task_id = task.task_id.clone();

        Task::perform(
            async move {
                ApiClient::new(&base_url)
                    .task_status(&task_id)
                    .await
                    .map_err(|e| e.to_string())
            },
            Message::TaskProgress,
        )
    }

    fn fetch_task_audio(&self) -> Task<Message> {
        let Some(task) = &self.active_task else {
            return Task::none();
        };

        let base_url = self.api_base_url();
        let task_id = task.task_id.clone();

        Task::perform(
            async move {
                ApiClient::new(&base_url)
                    .task_audio(&task_id)
                    .await
                    .map_err(|e| e.to_string())
            },
            Message::TaskAudioLoaded,
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

    fn view_main(&self) -> Element<'_, Message> {
        match self.active_tab {
            TabId::Clone => crate::views::clone_tab::view(
                &self.clone_tab,
                &self.references,
                &self.languages,
                self.active_task.as_ref(),
            ),
            _ => center(text("Coming soon...").size(16)).into(),
        }
    }
    // LCOV_EXCL_STOP
}
