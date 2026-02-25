use std::time::Duration;

use iced::widget::{button, center, column, container, progress_bar, row, scrollable, text};
use iced::{Element, Length, Subscription, Task, Theme};

use crate::api::client::ApiClient;
use crate::api::types::{
    CloneRequest, CustomVoiceRequest, GeneratedAudio, MultiSpeakerRequest, MultiSpeakerSegment,
    ReferenceAudio, TaskStatus, VoiceDesignRequest,
};
use crate::audio::player::{AudioPlayer, PlaybackState};
use crate::audio::recorder::{Recorder, RecordingState};
use crate::config::AppConfig;
use crate::message::{ActiveTask, Message, TabId};
use crate::server::manager::ServerManager;
use crate::views::clone_tab::CloneTabState;
use crate::views::custom_tab::CustomTabState;
use crate::views::design_tab::DesignTabState;
use crate::views::multispeaker_tab::MultiSpeakerTabState;
use crate::views::upload_tab::UploadTabState;

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
    app_config: AppConfig,
    edit_config: AppConfig,
    settings_dirty: bool,
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

    // ─── Upload tab ───────────────────────────────────────
    upload_tab: UploadTabState,

    // ─── Design tab ──────────────────────────────────────
    design_tab: DesignTabState,

    // ─── Custom Voice tab ────────────────────────────────
    custom_tab: CustomTabState,
    speakers: Vec<String>,

    // ─── Multi-Speaker tab ───────────────────────────────
    multi_tab: MultiSpeakerTabState,

    // ─── Generated list ──────────────────────────────────
    generated_list: Vec<GeneratedAudio>,

    // ─── Audio playback / recording ─────────────────────
    player: Option<AudioPlayer>,
    recorder: Option<Recorder>,
}

impl Default for Qvox {
    fn default() -> Self {
        let config = crate::config::load();
        Self {
            screen: Screen::Loading,
            server: None,
            edit_config: config.clone(),
            app_config: config,
            settings_dirty: false,
            elapsed_secs: 0,
            loading_status: "Starting server...".to_owned(),
            error: None,
            active_tab: TabId::Clone,
            references: Vec::new(),
            languages: vec!["auto".to_owned()],
            available_models: Vec::new(),
            clone_tab: CloneTabState::new(),
            active_task: None,
            upload_tab: UploadTabState::new(),
            design_tab: DesignTabState::new(),
            custom_tab: CustomTabState::new(),
            speakers: Vec::new(),
            multi_tab: MultiSpeakerTabState::new(),
            generated_list: Vec::new(),
            player: None,
            recorder: None,
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

    pub fn theme(&self) -> Theme {
        if self.app_config.ui.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
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

            // ─── Design tab inputs ────────────────────────────
            Message::DesignTextChanged(_)
            | Message::DesignInstructChanged(_)
            | Message::DesignLanguageSelected(_)
            | Message::DesignGenerate => self.update_design(message),

            // ─── Custom Voice tab inputs ─────────────────────
            Message::CustomTextChanged(_)
            | Message::CustomSpeakerSelected(_)
            | Message::CustomLanguageSelected(_)
            | Message::CustomInstructChanged(_)
            | Message::CustomGenerate => self.update_custom(message),

            // ─── Multi-Speaker tab inputs ─────────────────────
            Message::MultiAddSegment
            | Message::MultiRemoveSegment(_)
            | Message::MultiRefSelected(_, _)
            | Message::MultiTextChanged(_, _)
            | Message::MultiLanguageSelected(_, _)
            | Message::MultiGenerate => self.update_multi(message),

            // ─── Task lifecycle ─────────────────────────────
            Message::TaskCreated(_)
            | Message::TaskPollTick
            | Message::TaskProgress(_)
            | Message::TaskAudioLoaded(_) => self.update_task(message),

            // ─── Upload tab inputs ─────────────────────────
            Message::UploadPickFile
            | Message::UploadFileSelected(_, _, _)
            | Message::UploadTextChanged(_)
            | Message::UploadLanguageSelected(_)
            | Message::UploadGenerate
            | Message::RecordStart
            | Message::RecordStop
            | Message::RecordTick
            | Message::ModelDownloadProgress(_, _)
            | Message::ModelDownloaded(_)
            | Message::TranscriptionDone(_) => self.update_upload(message),

            // ─── Playback ─────────────────────────────────
            Message::PlayGenerated
            | Message::PlayReference(_)
            | Message::ReferenceAudioFetched(_)
            | Message::PlaybackPause
            | Message::PlaybackResume
            | Message::PlaybackStop
            | Message::PlaybackTick => self.update_playback(message),

            // ─── Generated list ─────────────────────────────
            Message::GeneratedListLoaded(_)
            | Message::RefreshGeneratedList
            | Message::GeneratedPlay(_)
            | Message::GeneratedAudioFetched(_)
            | Message::GeneratedDelete(_)
            | Message::GeneratedDeleted(_) => self.update_generated(message),

            // ─── Settings ──────────────────────────────────────
            Message::SettingsModelToggled(_)
            | Message::SettingsDeviceChanged(_)
            | Message::SettingsPortChanged(_)
            | Message::SettingsScriptPathChanged(_)
            | Message::SettingsDarkModeToggled(_)
            | Message::SettingsSave => self.update_settings(message),

            // ─── Error ─────────────────────────────────────────
            Message::ErrorDismiss => {
                self.error = None;
                Task::none()
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
        let is_loading = matches!(&self.screen, Screen::Loading) && self.error.is_none();
        let is_task_polling = self
            .active_task
            .as_ref()
            .is_some_and(|t| t.status == TaskStatus::Processing);
        let is_recording = self.recording_state() == RecordingState::Recording;
        let is_playing = self.playback_state() == PlaybackState::Playing;

        let mut subs = Vec::new();

        if is_loading {
            subs.push(iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick));
        }
        if is_task_polling {
            subs.push(iced::time::every(Duration::from_secs(1)).map(|_| Message::TaskPollTick));
        }
        if is_recording {
            subs.push(iced::time::every(Duration::from_millis(200)).map(|_| Message::RecordTick));
        }
        if is_playing {
            subs.push(iced::time::every(Duration::from_millis(250)).map(|_| Message::PlaybackTick));
        }

        Subscription::batch(subs)
    }

    // ─── Update sub-handlers ────────────────────────────────────

    fn update_server(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServerSpawned => match ServerManager::spawn(&self.app_config.to_server_config()) {
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
                if let Some(ref mut mgr) = self.server {
                    if self.error.is_none() {
                        if mgr.is_running() {
                            self.poll_health()
                        } else {
                            self.error = Some(
                                "Server process exited unexpectedly. Check the terminal for details.".to_owned(),
                            );
                            self.loading_status = "Error: server crashed".to_owned();
                            Task::none()
                        }
                    } else {
                        Task::none()
                    }
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
                self.speakers = caps.speakers;
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

    fn update_design(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DesignTextChanged(t) => {
                self.design_tab.text = t;
                Task::none()
            }
            Message::DesignInstructChanged(t) => {
                self.design_tab.instruct = t;
                Task::none()
            }
            Message::DesignLanguageSelected(lang) => {
                self.design_tab.selected_language = lang;
                Task::none()
            }
            Message::DesignGenerate => self.start_design_generation(),
            _ => Task::none(),
        }
    }

    fn update_custom(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CustomTextChanged(t) => {
                self.custom_tab.text = t;
                Task::none()
            }
            Message::CustomSpeakerSelected(s) => {
                self.custom_tab.selected_speaker = Some(s);
                Task::none()
            }
            Message::CustomLanguageSelected(lang) => {
                self.custom_tab.selected_language = lang;
                Task::none()
            }
            Message::CustomInstructChanged(t) => {
                self.custom_tab.instruct = t;
                Task::none()
            }
            Message::CustomGenerate => self.start_custom_generation(),
            _ => Task::none(),
        }
    }

    fn update_multi(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MultiAddSegment => {
                self.multi_tab
                    .segments
                    .push(crate::views::multispeaker_tab::SegmentState::default());
                Task::none()
            }
            Message::MultiRemoveSegment(i) => {
                if i < self.multi_tab.segments.len() && self.multi_tab.segments.len() > 1 {
                    self.multi_tab.segments.remove(i);
                }
                Task::none()
            }
            Message::MultiRefSelected(i, name) => {
                if let Some(seg) = self.multi_tab.segments.get_mut(i) {
                    seg.selected_ref = Some(name);
                }
                Task::none()
            }
            Message::MultiTextChanged(i, t) => {
                if let Some(seg) = self.multi_tab.segments.get_mut(i) {
                    seg.text = t;
                }
                Task::none()
            }
            Message::MultiLanguageSelected(i, lang) => {
                if let Some(seg) = self.multi_tab.segments.get_mut(i) {
                    seg.selected_language = lang;
                }
                Task::none()
            }
            Message::MultiGenerate => self.start_multi_generation(),
            _ => Task::none(),
        }
    }

    fn update_upload(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UploadPickFile => Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("Audio", &["wav", "mp3", "flac", "ogg", "m4a"])
                        .set_title("Select audio file")
                        .pick_file()
                        .await;

                    match handle {
                        Some(file) => {
                            let name = file.file_name();
                            let path = file.path().to_path_buf();
                            let bytes = file.read().await;
                            Ok((path, bytes, name))
                        }
                        None => Err("No file selected".to_owned()),
                    }
                },
                |result: Result<(std::path::PathBuf, Vec<u8>, String), String>| match result {
                    Ok((path, bytes, name)) => Message::UploadFileSelected(path, bytes, name),
                    Err(_) => Message::UploadPickFile, // silently ignore cancel
                },
            ),
            Message::UploadFileSelected(path, bytes, name) => {
                let hash = crate::audio::hash::bytes_sha256(&bytes);

                // Check transcription cache
                let cached = crate::transcribe::whisper::cached_transcription(&hash);

                self.upload_tab.selected_file = Some(path);
                self.upload_tab.file_bytes = Some(bytes.clone());
                self.upload_tab.file_name = Some(name);
                self.upload_tab.file_hash = Some(hash.clone());

                if let Some(text) = cached {
                    self.upload_tab.ref_text = Some(text);
                    Task::none()
                } else {
                    // Start transcription in background
                    self.upload_tab.transcribing = true;
                    self.upload_tab.ref_text = None;
                    self.start_transcription(bytes, hash)
                }
            }
            Message::TranscriptionDone(result) => {
                self.upload_tab.transcribing = false;
                match result {
                    Ok(text) => self.upload_tab.ref_text = Some(text),
                    Err(e) => self.error = Some(format!("Transcription failed: {e}")),
                }
                Task::none()
            }
            Message::ModelDownloadProgress(_, _) | Message::ModelDownloaded(_) => {
                // Model download progress is handled silently for now;
                // the transcription task chains download → transcribe.
                Task::none()
            }
            Message::UploadTextChanged(t) => {
                self.upload_tab.text = t;
                Task::none()
            }
            Message::UploadLanguageSelected(lang) => {
                self.upload_tab.selected_language = lang;
                Task::none()
            }
            Message::UploadGenerate => self.start_upload_generation(),
            Message::RecordStart => {
                self.ensure_recorder();
                if let Some(rec) = &mut self.recorder
                    && let Err(e) = rec.start()
                {
                    self.error = Some(format!("Recording error: {e}"));
                }
                Task::none()
            }
            Message::RecordStop => {
                if let Some(rec) = &mut self.recorder {
                    let samples = rec.stop();
                    let sample_rate = rec.sample_rate();
                    if !samples.is_empty() {
                        match crate::audio::recorder::samples_to_wav(&samples, sample_rate) {
                            Ok(wav_bytes) => {
                                let name = "recording.wav".to_owned();
                                return Task::done(Message::UploadFileSelected(
                                    std::path::PathBuf::from(&name),
                                    wav_bytes,
                                    name,
                                ));
                            }
                            Err(e) => self.error = Some(format!("WAV encode error: {e}")),
                        }
                    }
                }
                Task::none()
            }
            Message::RecordTick => {
                // Just triggers a view refresh via subscription
                Task::none()
            }
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
                    match &result {
                        Ok(data) => task.audio_data = Some(data.clone()),
                        Err(e) => task.error = Some(e.clone()),
                    }
                }
                if result.is_ok() {
                    self.fetch_generated_list()
                } else {
                    Task::none()
                }
            }
            _ => Task::none(),
        }
    }

    fn update_playback(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PlayGenerated => {
                if let Some(data) = self
                    .active_task
                    .as_ref()
                    .and_then(|t| t.audio_data.clone())
                {
                    self.play_audio(data);
                }
                Task::none()
            }
            Message::PlayReference(ref_id) => {
                let base_url = self.api_base_url();
                Task::perform(
                    async move {
                        ApiClient::new(&base_url)
                            .reference_audio(&ref_id)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::ReferenceAudioFetched,
                )
            }
            Message::ReferenceAudioFetched(Ok(data)) => {
                self.play_audio(data);
                Task::none()
            }
            Message::ReferenceAudioFetched(Err(e)) => {
                self.error = Some(e);
                Task::none()
            }
            Message::PlaybackPause => {
                if let Some(player) = &mut self.player {
                    player.pause();
                }
                Task::none()
            }
            Message::PlaybackResume => {
                if let Some(player) = &mut self.player {
                    player.resume();
                }
                Task::none()
            }
            Message::PlaybackStop => {
                if let Some(player) = &mut self.player {
                    player.stop();
                }
                Task::none()
            }
            Message::PlaybackTick => {
                // Triggers a view refresh; playback_state() detects when audio finished.
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn update_generated(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GeneratedListLoaded(Ok(list)) => {
                self.generated_list = list;
                Task::none()
            }
            Message::GeneratedListLoaded(Err(e)) => {
                self.error = Some(format!("Failed to load generated list: {e}"));
                Task::none()
            }
            Message::RefreshGeneratedList => self.fetch_generated_list(),
            Message::GeneratedPlay(audio_id) => {
                let base_url = self.api_base_url();
                Task::perform(
                    async move {
                        ApiClient::new(&base_url)
                            .task_audio(&audio_id)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::GeneratedAudioFetched,
                )
            }
            Message::GeneratedAudioFetched(Ok(data)) => {
                self.play_audio(data);
                Task::none()
            }
            Message::GeneratedAudioFetched(Err(e)) => {
                self.error = Some(format!("Failed to fetch audio: {e}"));
                Task::none()
            }
            Message::GeneratedDelete(audio_id) => {
                let base_url = self.api_base_url();
                let id = audio_id.clone();
                Task::perform(
                    async move {
                        ApiClient::new(&base_url)
                            .delete_generated(&id)
                            .await
                            .map(|_| audio_id)
                            .map_err(|e| e.to_string())
                    },
                    Message::GeneratedDeleted,
                )
            }
            Message::GeneratedDeleted(Ok(audio_id)) => {
                self.generated_list.retain(|g| g.id != audio_id);
                Task::none()
            }
            Message::GeneratedDeleted(Err(e)) => {
                self.error = Some(format!("Failed to delete: {e}"));
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn update_settings(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SettingsModelToggled(model) => {
                let models = &mut self.edit_config.server.models;
                if let Some(pos) = models.iter().position(|m| m == &model) {
                    models.remove(pos);
                } else {
                    models.push(model);
                }
                self.settings_dirty = self.edit_config != self.app_config;
                Task::none()
            }
            Message::SettingsDeviceChanged(s) => {
                self.edit_config.server.device = s;
                self.settings_dirty = self.edit_config != self.app_config;
                Task::none()
            }
            Message::SettingsPortChanged(s) => {
                if let Ok(port) = s.parse::<u16>() {
                    self.edit_config.server.port = port;
                }
                self.settings_dirty = self.edit_config != self.app_config;
                Task::none()
            }
            Message::SettingsScriptPathChanged(s) => {
                self.edit_config.server.script_path = s;
                self.settings_dirty = self.edit_config != self.app_config;
                Task::none()
            }
            Message::SettingsDarkModeToggled(enabled) => {
                self.edit_config.ui.dark_mode = enabled;
                // Apply dark mode immediately
                self.app_config.ui.dark_mode = enabled;
                self.settings_dirty = self.edit_config != self.app_config;
                let _ = crate::config::save(&self.app_config);
                Task::none()
            }
            Message::SettingsSave => {
                self.app_config = self.edit_config.clone();
                self.settings_dirty = false;
                let _ = crate::config::save(&self.app_config);
                // Kill existing server and restart
                if let Some(server) = &mut self.server {
                    server.kill();
                }
                self.server = None;
                self.screen = Screen::Loading;
                self.elapsed_secs = 0;
                self.error = None;
                "Restarting server...".clone_into(&mut self.loading_status);
                Task::done(Message::ServerSpawned)
            }
            _ => Task::none(),
        }
    }

    // ─── Private helpers ────────────────────────────────────────

    fn ensure_recorder(&mut self) {
        if self.recorder.is_none() {
            match Recorder::new() {
                Ok(r) => self.recorder = Some(r),
                Err(e) => self.error = Some(format!("Microphone error: {e}")),
            }
        }
    }

    fn recording_state(&self) -> RecordingState {
        self.recorder
            .as_ref()
            .map_or(RecordingState::Idle, Recorder::state)
    }

    fn ensure_player(&mut self) -> Option<&mut AudioPlayer> {
        if self.player.is_none() {
            match AudioPlayer::new() {
                Ok(p) => self.player = Some(p),
                Err(e) => {
                    self.error = Some(format!("Audio device error: {e}"));
                    return None;
                }
            }
        }
        self.player.as_mut()
    }

    fn play_audio(&mut self, data: Vec<u8>) {
        if let Some(player) = self.ensure_player()
            && let Err(e) = player.play_bytes(data)
        {
            self.error = Some(format!("Playback error: {e}"));
        }
    }

    fn playback_state(&self) -> PlaybackState {
        self.player
            .as_ref()
            .map_or(PlaybackState::Stopped, AudioPlayer::state)
    }

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
        let url4 = url.clone();

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
            Task::perform(
                async move {
                    ApiClient::new(&url4)
                        .generated_list()
                        .await
                        .map_err(|e| e.to_string())
                },
                Message::GeneratedListLoaded,
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

    fn start_upload_generation(&mut self) -> Task<Message> {
        let Some(file_bytes) = self.upload_tab.file_bytes.clone() else {
            return Task::none();
        };
        let Some(file_name) = self.upload_tab.file_name.clone() else {
            return Task::none();
        };

        let text = self.upload_tab.text.clone();
        let language = self.upload_tab.selected_language.clone();
        let ref_text = self.upload_tab.ref_text.clone();
        let base_url = self.api_base_url();

        Task::perform(
            async move {
                ApiClient::new(&base_url)
                    .clone_with_upload(
                        file_bytes,
                        file_name,
                        &text,
                        ref_text.as_deref(),
                        Some(&language),
                    )
                    .await
                    .map(|resp| resp.task_id)
                    .map_err(|e| e.to_string())
            },
            Message::TaskCreated,
        )
    }

    fn start_design_generation(&mut self) -> Task<Message> {
        let request = VoiceDesignRequest {
            text: self.design_tab.text.clone(),
            instruct: self.design_tab.instruct.clone(),
            language: self.design_tab.selected_language.clone(),
        };

        let base_url = self.api_base_url();

        Task::perform(
            async move {
                ApiClient::new(&base_url)
                    .voice_design(&request)
                    .await
                    .map(|resp| resp.task_id)
                    .map_err(|e| e.to_string())
            },
            Message::TaskCreated,
        )
    }

    fn start_custom_generation(&mut self) -> Task<Message> {
        let Some(speaker) = self.custom_tab.selected_speaker.clone() else {
            return Task::none();
        };

        let instruct = if self.custom_tab.instruct.is_empty() {
            None
        } else {
            Some(self.custom_tab.instruct.clone())
        };

        let request = CustomVoiceRequest {
            text: self.custom_tab.text.clone(),
            speaker,
            language: self.custom_tab.selected_language.clone(),
            instruct,
        };

        let base_url = self.api_base_url();

        Task::perform(
            async move {
                ApiClient::new(&base_url)
                    .custom_voice(&request)
                    .await
                    .map(|resp| resp.task_id)
                    .map_err(|e| e.to_string())
            },
            Message::TaskCreated,
        )
    }

    fn start_multi_generation(&mut self) -> Task<Message> {
        let segments: Vec<MultiSpeakerSegment> = self
            .multi_tab
            .segments
            .iter()
            .filter_map(|seg| {
                let ref_name = seg.selected_ref.as_ref()?;
                let ref_audio = self.references.iter().find(|r| {
                    r.name.as_deref().unwrap_or(&r.original_name) == ref_name.as_str()
                })?;
                Some(MultiSpeakerSegment {
                    text: seg.text.clone(),
                    ref_audio_id: ref_audio.id.clone(),
                    ref_text: ref_audio.ref_text.clone(),
                    language: seg.selected_language.clone(),
                })
            })
            .collect();

        if segments.len() != self.multi_tab.segments.len() {
            return Task::none();
        }

        let request = MultiSpeakerRequest { segments };
        let base_url = self.api_base_url();

        Task::perform(
            async move {
                ApiClient::new(&base_url)
                    .clone_multi_speaker(&request)
                    .await
                    .map(|resp| resp.task_id)
                    .map_err(|e| e.to_string())
            },
            Message::TaskCreated,
        )
    }

    #[allow(clippy::unused_self)]
    fn start_transcription(&self, wav_bytes: Vec<u8>, hash: String) -> Task<Message> {
        Task::perform(
            async move {
                // Ensure model is downloaded
                if !crate::transcribe::whisper::model_exists() {
                    crate::transcribe::whisper::download_model(|_, _| {})
                        .await
                        .map_err(|e| e.to_string())?;
                }

                // Run transcription in a blocking thread
                tokio::task::spawn_blocking(move || {
                    let result = crate::transcribe::whisper::transcribe(&wav_bytes)
                        .map_err(|e| e.to_string())?;

                    // Cache the result
                    let _ = crate::transcribe::whisper::save_transcription_cache(&hash, &result);

                    Ok(result)
                })
                .await
                .map_err(|e| e.to_string())?
            },
            Message::TranscriptionDone,
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

    fn fetch_generated_list(&self) -> Task<Message> {
        let base_url = self.api_base_url();
        Task::perform(
            async move {
                ApiClient::new(&base_url)
                    .generated_list()
                    .await
                    .map_err(|e| e.to_string())
            },
            Message::GeneratedListLoaded,
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

        let models_text =
            text(format!("Models: {}", self.app_config.server.models.join(", "))).size(12);
        let device_text = text(format!("Device: {}", self.app_config.server.device)).size(12);

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
        let tab_bar = self.view_tab_bar();

        let tab_content = match self.active_tab {
            TabId::Clone => crate::views::clone_tab::view(
                &self.clone_tab,
                &self.references,
                &self.languages,
                self.active_task.as_ref(),
                self.playback_state(),
            ),
            TabId::Upload => crate::views::upload_tab::view(
                &self.upload_tab,
                &self.languages,
                self.active_task.as_ref(),
                self.playback_state(),
                self.recording_state(),
                self.recorder.as_ref().map_or(0.0, Recorder::elapsed_secs),
            ),
            TabId::VoiceDesign => crate::views::design_tab::view(
                &self.design_tab,
                &self.languages,
                self.active_task.as_ref(),
                self.playback_state(),
            ),
            TabId::CustomVoice => crate::views::custom_tab::view(
                &self.custom_tab,
                &self.speakers,
                &self.languages,
                self.active_task.as_ref(),
                self.playback_state(),
            ),
            TabId::MultiSpeaker => crate::views::multispeaker_tab::view(
                &self.multi_tab,
                &self.references,
                &self.languages,
                self.active_task.as_ref(),
                self.playback_state(),
            ),
            TabId::Settings => crate::views::settings::view(
                &self.edit_config,
                self.settings_dirty,
            ),
        };

        let generated = crate::views::generated_list::view(&self.generated_list);

        let mut main_col = column![tab_bar].spacing(0).width(Length::Fill);

        // Error banner
        if let Some(err) = &self.error {
            main_col = main_col.push(
                row![
                    text(err).size(13),
                    button(text("Dismiss")).on_press(Message::ErrorDismiss),
                ]
                .spacing(8)
                .padding(8),
            );
        }

        main_col = main_col.push(
            scrollable(
                column![tab_content, generated]
                    .spacing(16)
                    .width(Length::Fill),
            )
            .height(Length::Fill),
        );

        main_col.into()
    }

    fn view_tab_bar(&self) -> Element<'_, Message> {
        let tabs = [
            ("Clone", TabId::Clone),
            ("Upload", TabId::Upload),
            ("Multi-Speaker", TabId::MultiSpeaker),
            ("Voice Design", TabId::VoiceDesign),
            ("Custom Voice", TabId::CustomVoice),
            ("Settings", TabId::Settings),
        ];

        let mut tab_row = row![].spacing(0);
        for (label, id) in tabs {
            let btn = if self.active_tab == id {
                button(text(label).size(13))
            } else {
                button(text(label).size(13)).on_press(Message::TabSelected(id))
            };
            tab_row = tab_row.push(btn);
        }

        tab_row.padding(4).into()
    }
    // LCOV_EXCL_STOP
}
