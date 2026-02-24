use serde::{Deserialize, Serialize};

// ─── Server Management ─────────────────────────────────────────

/// Response from `GET /health`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
    pub voice_cloner_loaded: bool,
    pub loaded_models: Vec<String>,
}

/// Response from `GET /capabilities`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilitiesResponse {
    pub models: Vec<String>,
    pub speakers: Vec<String>,
}

/// Response from `GET /languages`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanguagesResponse {
    pub languages: Vec<String>,
}

// ─── Reference Audio ────────────────────────────────────────────

/// Reference audio metadata, returned by `POST /upload-reference` and `GET /references`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferenceAudio {
    pub id: String,
    pub filename: String,
    pub original_name: String,
    pub name: Option<String>,
    pub ref_text: Option<String>,
    pub created_at: String,
}

/// Request body for `PUT /references/{audio_id}/name`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenameRequest {
    pub name: String,
}

/// Response from `PUT /references/{audio_id}/name`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenameResponse {
    pub message: String,
    pub name: String,
}

// ─── Voice Generation Requests ──────────────────────────────────

/// Request body for `POST /clone`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloneRequest {
    pub text: String,
    pub ref_audio_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_text: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
}

/// A single segment in a `POST /clone-multi-speaker` request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultiSpeakerSegment {
    pub text: String,
    pub ref_audio_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_text: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
}

/// Request body for `POST /clone-multi-speaker`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultiSpeakerRequest {
    pub segments: Vec<MultiSpeakerSegment>,
}

/// Request body for `POST /voice-design`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceDesignRequest {
    pub text: String,
    pub instruct: String,
    #[serde(default = "default_language")]
    pub language: String,
}

/// Request body for `POST /custom-voice`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomVoiceRequest {
    pub text: String,
    pub speaker: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruct: Option<String>,
}

// ─── Voice Generation Response ──────────────────────────────────

/// Shared response for all voice generation endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloneResponse {
    pub task_id: String,
    pub status: String,
    pub output_path: Option<String>,
    pub message: String,
    pub estimated_time: Option<f64>,
}

// ─── Task Management ────────────────────────────────────────────

/// Task status values
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Response from `GET /tasks/{task_id}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskStatusResponse {
    pub status: TaskStatus,
    pub progress: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_audio_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_time_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    // Multi-speaker fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_multi_speaker: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_segments: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_segment: Option<u32>,
}

/// Response from `POST /tasks/{task_id}/cancel`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CancelResponse {
    pub message: String,
}

// ─── Generated Audio ────────────────────────────────────────────

/// An item from `GET /generated`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneratedAudio {
    pub id: String,
    pub filename: String,
    pub ref_audio_id: Option<String>,
    pub ref_audio_name: Option<String>,
    pub generated_text: String,
    pub created_at: String,
    pub generation_time_seconds: Option<f64>,
}

/// Response from `DELETE /references/{id}` or `DELETE /generated/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeleteResponse {
    pub message: String,
}

// ─── Helpers ────────────────────────────────────────────────────

fn default_language() -> String {
    String::from("auto")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_response_round_trip() {
        let original = HealthResponse {
            status: "healthy".to_owned(),
            voice_cloner_loaded: true,
            loaded_models: vec!["base".to_owned(), "voice_design".to_owned()],
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: HealthResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn capabilities_response_round_trip() {
        let original = CapabilitiesResponse {
            models: vec!["base".to_owned(), "custom_voice".to_owned()],
            speakers: vec!["Vivian".to_owned(), "Dylan".to_owned()],
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: CapabilitiesResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn languages_response_round_trip() {
        let original = LanguagesResponse {
            languages: vec!["auto".to_owned(), "English".to_owned(), "Japanese".to_owned()],
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: LanguagesResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn reference_audio_round_trip() {
        let original = ReferenceAudio {
            id: "abc-123".to_owned(),
            filename: "abc-123.wav".to_owned(),
            original_name: "sample.wav".to_owned(),
            name: None,
            ref_text: Some("hello world".to_owned()),
            created_at: "1234567890.123".to_owned(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: ReferenceAudio = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn reference_audio_with_name() {
        let original = ReferenceAudio {
            id: "abc-123".to_owned(),
            filename: "abc-123.wav".to_owned(),
            original_name: "sample.wav".to_owned(),
            name: Some("My Voice".to_owned()),
            ref_text: None,
            created_at: "1234567890.123".to_owned(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: ReferenceAudio = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn clone_request_round_trip() {
        let original = CloneRequest {
            text: "Hello".to_owned(),
            ref_audio_id: "uuid-1".to_owned(),
            ref_text: Some("reference".to_owned()),
            language: "auto".to_owned(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: CloneRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn clone_request_omits_none_ref_text() {
        let req = CloneRequest {
            text: "Hello".to_owned(),
            ref_audio_id: "uuid-1".to_owned(),
            ref_text: None,
            language: "auto".to_owned(),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("ref_text"));
    }

    #[test]
    fn clone_request_default_language() {
        let json = r#"{"text":"hi","ref_audio_id":"id1"}"#;
        let req: CloneRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.language, "auto");
        assert_eq!(req.ref_text, None);
    }

    #[test]
    fn clone_response_round_trip() {
        let original = CloneResponse {
            task_id: "task-1".to_owned(),
            status: "processing".to_owned(),
            output_path: None,
            message: "Voice cloning started".to_owned(),
            estimated_time: None,
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: CloneResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn multi_speaker_request_round_trip() {
        let original = MultiSpeakerRequest {
            segments: vec![
                MultiSpeakerSegment {
                    text: "Line 1".to_owned(),
                    ref_audio_id: "uuid1".to_owned(),
                    ref_text: None,
                    language: "auto".to_owned(),
                },
                MultiSpeakerSegment {
                    text: "Line 2".to_owned(),
                    ref_audio_id: "uuid2".to_owned(),
                    ref_text: Some("ref".to_owned()),
                    language: "English".to_owned(),
                },
            ],
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: MultiSpeakerRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn voice_design_request_round_trip() {
        let original = VoiceDesignRequest {
            text: "Hello".to_owned(),
            instruct: "A warm voice".to_owned(),
            language: "English".to_owned(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: VoiceDesignRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn custom_voice_request_round_trip() {
        let original = CustomVoiceRequest {
            text: "Hello".to_owned(),
            speaker: "Vivian".to_owned(),
            language: "auto".to_owned(),
            instruct: Some("Speak slowly".to_owned()),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: CustomVoiceRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn custom_voice_request_omits_none_instruct() {
        let req = CustomVoiceRequest {
            text: "Hello".to_owned(),
            speaker: "Dylan".to_owned(),
            language: "auto".to_owned(),
            instruct: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(!json.contains("instruct"));
    }

    #[test]
    fn task_status_deserialize() {
        assert_eq!(
            serde_json::from_str::<TaskStatus>(r#""processing""#).expect("deserialize"),
            TaskStatus::Processing,
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>(r#""completed""#).expect("deserialize"),
            TaskStatus::Completed,
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>(r#""failed""#).expect("deserialize"),
            TaskStatus::Failed,
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>(r#""cancelled""#).expect("deserialize"),
            TaskStatus::Cancelled,
        );
    }

    #[test]
    fn task_status_response_processing() {
        let json = r#"{"status":"processing","progress":50,"ref_audio_id":"uuid"}"#;
        let resp: TaskStatusResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.status, TaskStatus::Processing);
        assert_eq!(resp.progress, 50);
        assert_eq!(resp.ref_audio_id.as_deref(), Some("uuid"));
        assert!(resp.output_path.is_none());
    }

    #[test]
    fn task_status_response_completed() {
        let json = r#"{
            "status": "completed",
            "progress": 100,
            "output_path": "output/cloned_uuid.wav",
            "ref_audio_id": "uuid",
            "generation_time_seconds": 12.34
        }"#;
        let resp: TaskStatusResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.status, TaskStatus::Completed);
        assert_eq!(resp.progress, 100);
        assert_eq!(resp.output_path.as_deref(), Some("output/cloned_uuid.wav"));
        assert_eq!(resp.generation_time_seconds, Some(12.34));
    }

    #[test]
    fn task_status_response_failed() {
        let json = r#"{"status":"failed","progress":50,"error":"Cloning task failed"}"#;
        let resp: TaskStatusResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.status, TaskStatus::Failed);
        assert_eq!(resp.error.as_deref(), Some("Cloning task failed"));
    }

    #[test]
    fn task_status_response_multi_speaker() {
        let json = r#"{
            "status": "processing",
            "progress": 45,
            "is_multi_speaker": true,
            "total_segments": 3,
            "current_segment": 2
        }"#;
        let resp: TaskStatusResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.is_multi_speaker, Some(true));
        assert_eq!(resp.total_segments, Some(3));
        assert_eq!(resp.current_segment, Some(2));
    }

    #[test]
    fn generated_audio_round_trip() {
        let original = GeneratedAudio {
            id: "task-uuid".to_owned(),
            filename: "cloned_uuid.wav".to_owned(),
            ref_audio_id: Some("ref-uuid".to_owned()),
            ref_audio_name: Some("sample.wav".to_owned()),
            generated_text: "Generated text".to_owned(),
            created_at: "1234567890.123".to_owned(),
            generation_time_seconds: Some(12.34),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: GeneratedAudio = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn rename_request_round_trip() {
        let original = RenameRequest {
            name: "New Name".to_owned(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: RenameRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn delete_response_round_trip() {
        let original = DeleteResponse {
            message: "Deleted successfully".to_owned(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: DeleteResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn cancel_response_round_trip() {
        let original = CancelResponse {
            message: "Task cancelled successfully".to_owned(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: CancelResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }
}
