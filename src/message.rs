use crate::api::types::{
    CapabilitiesResponse, GeneratedAudio, LanguagesResponse, ReferenceAudio, TaskStatus,
    TaskStatusResponse,
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    // ─── Server lifecycle ───────────────────────────────────────
    /// Server process has been spawned; begin health polling.
    ServerSpawned,
    /// Result of a health check poll.
    HealthCheck(bool),
    /// Server is ready (voice cloner loaded).
    ServerReady,
    /// Server failed to start or crashed.
    ServerError(String),
    /// Elapsed-time tick while loading (every 1 second).
    Tick,

    // ─── Data loading ───────────────────────────────────────────
    /// Capabilities fetched from server.
    CapabilitiesLoaded(Result<CapabilitiesResponse, String>),
    /// Reference audio list fetched.
    ReferencesLoaded(Result<Vec<ReferenceAudio>, String>),
    /// Languages list fetched.
    LanguagesLoaded(Result<LanguagesResponse, String>),

    // ─── Tab navigation ─────────────────────────────────────────
    /// User switched tabs.
    TabSelected(TabId),

    // ─── Clone tab inputs ───────────────────────────────────────
    /// Text input changed.
    CloneTextChanged(String),
    /// Reference audio selected from dropdown.
    CloneRefSelected(String),
    /// Language selected.
    CloneLanguageSelected(String),
    /// Generate button pressed.
    CloneGenerate,

    // ─── Design tab inputs ─────────────────────────────────────
    /// Text input changed on design tab.
    DesignTextChanged(String),
    /// Voice description (instruct) changed.
    DesignInstructChanged(String),
    /// Language selected on design tab.
    DesignLanguageSelected(String),
    /// Generate button pressed on design tab.
    DesignGenerate,

    // ─── Custom Voice tab inputs ────────────────────────────────
    /// Text input changed on custom voice tab.
    CustomTextChanged(String),
    /// Speaker selected from dropdown.
    CustomSpeakerSelected(String),
    /// Language selected on custom voice tab.
    CustomLanguageSelected(String),
    /// Style instruct changed on custom voice tab.
    CustomInstructChanged(String),
    /// Generate button pressed on custom voice tab.
    CustomGenerate,

    // ─── Multi-Speaker tab inputs ──────────────────────────────
    /// Add a new segment.
    MultiAddSegment,
    /// Remove segment at index.
    MultiRemoveSegment(usize),
    /// Reference audio selected for segment at index.
    MultiRefSelected(usize, String),
    /// Text changed for segment at index.
    MultiTextChanged(usize, String),
    /// Language selected for segment at index.
    MultiLanguageSelected(usize, String),
    /// Generate button pressed on multi-speaker tab.
    MultiGenerate,

    // ─── Task lifecycle ─────────────────────────────────────────
    /// Generation task created, received `task_id`.
    TaskCreated(Result<String, String>),
    /// Task status poll result.
    TaskProgress(Result<TaskStatusResponse, String>),
    /// Task polling tick (every 1 second during generation).
    TaskPollTick,
    /// Audio data fetched for completed task.
    TaskAudioLoaded(Result<Vec<u8>, String>),

    // ─── Playback ───────────────────────────────────────────────
    /// Play generated audio (from active task).
    PlayGenerated,
    /// Play reference audio preview.
    PlayReference(String),
    /// Reference audio bytes fetched for preview.
    ReferenceAudioFetched(Result<Vec<u8>, String>),
    /// Pause playback.
    PlaybackPause,
    /// Resume playback.
    PlaybackResume,
    /// Stop playback.
    PlaybackStop,

    // ─── Upload tab inputs ────────────────────────────────────────
    /// User clicked "Choose File" — open native file dialog.
    UploadPickFile,
    /// File selected from dialog (path, bytes, filename).
    UploadFileSelected(std::path::PathBuf, Vec<u8>, String),
    /// Text input changed on upload tab.
    UploadTextChanged(String),
    /// Language selected on upload tab.
    UploadLanguageSelected(String),
    /// Generate button pressed on upload tab.
    UploadGenerate,

    // ─── Recording ────────────────────────────────────────────────
    /// Start microphone recording.
    RecordStart,
    /// Stop recording; produces WAV bytes.
    RecordStop,
    /// Recording tick (update elapsed time display).
    RecordTick,

    // ─── Transcription ────────────────────────────────────────────
    /// Whisper model download progress (downloaded, total).
    ModelDownloadProgress(u64, u64),
    /// Model download finished.
    ModelDownloaded(Result<std::path::PathBuf, String>),
    /// Transcription result for uploaded audio.
    TranscriptionDone(Result<String, String>),

    // ─── Generated list ─────────────────────────────────────────
    /// Generated audio list fetched.
    GeneratedListLoaded(Result<Vec<GeneratedAudio>, String>),
    /// Refresh the generated audio list.
    RefreshGeneratedList,
    /// Play a generated audio item by ID.
    GeneratedPlay(String),
    /// Audio bytes fetched for a generated item.
    GeneratedAudioFetched(Result<Vec<u8>, String>),
    /// Delete a generated audio item by ID.
    GeneratedDelete(String),
    /// Deletion result.
    GeneratedDeleted(Result<String, String>),

    // ─── Settings ─────────────────────────────────────────────────
    /// Models field changed.
    SettingsModelsChanged(String),
    /// Device field changed.
    SettingsDeviceChanged(String),
    /// Port field changed.
    SettingsPortChanged(String),
    /// Script path field changed.
    SettingsScriptPathChanged(String),
    /// Dark mode toggled.
    SettingsDarkModeToggled(bool),
    /// Save settings and restart server.
    SettingsSave,

    // ─── Error ────────────────────────────────────────────────────
    /// Dismiss the error banner.
    ErrorDismiss,
}

/// Tab identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    Clone,
    Upload,
    MultiSpeaker,
    VoiceDesign,
    CustomVoice,
    Settings,
}

/// State of an active generation task.
#[derive(Debug, Clone)]
pub struct ActiveTask {
    pub task_id: String,
    pub status: TaskStatus,
    pub progress: u32,
    pub elapsed_secs: u64,
    pub status_text: String,
    pub error: Option<String>,
    pub audio_data: Option<Vec<u8>>,
}

impl ActiveTask {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            status: TaskStatus::Processing,
            progress: 0,
            elapsed_secs: 0,
            status_text: "Initializing voice cloner...".to_owned(),
            error: None,
            audio_data: None,
        }
    }

    pub fn update_progress(&mut self, resp: &TaskStatusResponse) {
        self.status = resp.status;
        self.progress = resp.progress;
        self.status_text = progress_text(resp);

        if let Some(err) = &resp.error {
            self.error = Some(err.clone());
        }
    }
}

/// Map task progress to user-facing status text.
fn progress_text(resp: &TaskStatusResponse) -> String {
    match resp.status {
        TaskStatus::Failed => resp
            .error
            .as_deref()
            .unwrap_or("Generation failed")
            .to_owned(),
        TaskStatus::Cancelled => "Generation cancelled".to_owned(),
        TaskStatus::Completed => "Complete!".to_owned(),
        TaskStatus::Processing => {
            if resp.is_multi_speaker == Some(true) {
                multi_speaker_progress_text(resp)
            } else {
                normal_progress_text(resp.progress)
            }
        }
    }
}

fn normal_progress_text(progress: u32) -> String {
    match progress {
        0..25 => "Initializing voice cloner...".to_owned(),
        25..50 => "Processing reference audio...".to_owned(),
        50..75 => "Generating cloned voice...".to_owned(),
        _ => "Finalizing audio...".to_owned(),
    }
}

fn multi_speaker_progress_text(resp: &TaskStatusResponse) -> String {
    let current = resp.current_segment.unwrap_or(0);
    let total = resp.total_segments.unwrap_or(0);

    match resp.progress {
        0..5 => "Initializing multi-speaker generation...".to_owned(),
        5..90 => format!("Generating segment {current} of {total}..."),
        90..95 => "Concatenating audio segments...".to_owned(),
        _ => "Finalizing audio...".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_task_new() {
        let task = ActiveTask::new("task-1".to_owned());
        assert_eq!(task.task_id, "task-1");
        assert_eq!(task.status, TaskStatus::Processing);
        assert_eq!(task.progress, 0);
        assert!(task.error.is_none());
        assert!(task.audio_data.is_none());
    }

    #[test]
    fn normal_progress_text_ranges() {
        assert!(normal_progress_text(0).contains("Initializing"));
        assert!(normal_progress_text(24).contains("Initializing"));
        assert!(normal_progress_text(25).contains("Processing"));
        assert!(normal_progress_text(49).contains("Processing"));
        assert!(normal_progress_text(50).contains("Generating"));
        assert!(normal_progress_text(74).contains("Generating"));
        assert!(normal_progress_text(75).contains("Finalizing"));
        assert!(normal_progress_text(100).contains("Finalizing"));
    }

    #[test]
    fn progress_text_completed() {
        let resp = TaskStatusResponse {
            status: TaskStatus::Completed,
            progress: 100,
            output_path: None,
            ref_audio_id: None,
            generation_time_seconds: None,
            error: None,
            is_multi_speaker: None,
            total_segments: None,
            current_segment: None,
        };
        assert_eq!(progress_text(&resp), "Complete!");
    }

    #[test]
    fn progress_text_failed() {
        let resp = TaskStatusResponse {
            status: TaskStatus::Failed,
            progress: 50,
            output_path: None,
            ref_audio_id: None,
            generation_time_seconds: None,
            error: Some("out of memory".to_owned()),
            is_multi_speaker: None,
            total_segments: None,
            current_segment: None,
        };
        assert_eq!(progress_text(&resp), "out of memory");
    }

    #[test]
    fn progress_text_multi_speaker() {
        let resp = TaskStatusResponse {
            status: TaskStatus::Processing,
            progress: 45,
            output_path: None,
            ref_audio_id: None,
            generation_time_seconds: None,
            error: None,
            is_multi_speaker: Some(true),
            total_segments: Some(3),
            current_segment: Some(2),
        };
        assert_eq!(progress_text(&resp), "Generating segment 2 of 3...");
    }

    #[test]
    fn active_task_update_progress() {
        let mut task = ActiveTask::new("t1".to_owned());
        let resp = TaskStatusResponse {
            status: TaskStatus::Processing,
            progress: 60,
            output_path: None,
            ref_audio_id: None,
            generation_time_seconds: None,
            error: None,
            is_multi_speaker: None,
            total_segments: None,
            current_segment: None,
        };
        task.update_progress(&resp);
        assert_eq!(task.progress, 60);
        assert!(task.status_text.contains("Generating"));
    }
}
