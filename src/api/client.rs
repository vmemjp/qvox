use anyhow::{Context, Result};
use reqwest::multipart;

use super::types::{
    CancelResponse, CapabilitiesResponse, CloneRequest, CloneResponse, CustomVoiceRequest,
    DeleteResponse, GeneratedAudio, HealthResponse, LanguagesResponse, MultiSpeakerRequest,
    ReferenceAudio, RenameRequest, RenameResponse, TaskStatusResponse, VoiceDesignRequest,
};

/// HTTP client for the Qwen3-TTS Python backend.
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    // ─── Server Management ──────────────────────────────────────

    pub async fn health(&self) -> Result<HealthResponse> {
        self.client
            .get(self.url("/health"))
            .send()
            .await
            .context("health request failed")?
            .error_for_status()
            .context("health returned error status")?
            .json()
            .await
            .context("failed to parse health response")
    }

    pub async fn capabilities(&self) -> Result<CapabilitiesResponse> {
        self.client
            .get(self.url("/capabilities"))
            .send()
            .await
            .context("capabilities request failed")?
            .error_for_status()
            .context("capabilities returned error status")?
            .json()
            .await
            .context("failed to parse capabilities response")
    }

    pub async fn languages(&self) -> Result<LanguagesResponse> {
        self.client
            .get(self.url("/languages"))
            .send()
            .await
            .context("languages request failed")?
            .error_for_status()
            .context("languages returned error status")?
            .json()
            .await
            .context("failed to parse languages response")
    }

    // ─── Reference Audio ────────────────────────────────────────

    pub async fn references(&self) -> Result<Vec<ReferenceAudio>> {
        self.client
            .get(self.url("/references"))
            .send()
            .await
            .context("references request failed")?
            .error_for_status()
            .context("references returned error status")?
            .json()
            .await
            .context("failed to parse references response")
    }

    pub async fn upload_reference(
        &self,
        file_bytes: Vec<u8>,
        filename: String,
        ref_text: Option<&str>,
    ) -> Result<ReferenceAudio> {
        let file_part = multipart::Part::bytes(file_bytes)
            .file_name(filename)
            .mime_str("audio/wav")
            .context("invalid mime type")?;

        let mut form = multipart::Form::new().part("file", file_part);
        if let Some(text) = ref_text {
            form = form.text("ref_text", text.to_owned());
        }

        self.client
            .post(self.url("/upload-reference"))
            .multipart(form)
            .send()
            .await
            .context("upload-reference request failed")?
            .error_for_status()
            .context("upload-reference returned error status")?
            .json()
            .await
            .context("failed to parse upload-reference response")
    }

    pub async fn reference_audio(&self, audio_id: &str) -> Result<Vec<u8>> {
        self.client
            .get(self.url(&format!("/references/{audio_id}/audio")))
            .send()
            .await
            .context("reference audio request failed")?
            .error_for_status()
            .context("reference audio returned error status")?
            .bytes()
            .await
            .context("failed to read reference audio bytes")
            .map(|b| b.to_vec())
    }

    pub async fn delete_reference(&self, audio_id: &str) -> Result<DeleteResponse> {
        self.client
            .delete(self.url(&format!("/references/{audio_id}")))
            .send()
            .await
            .context("delete reference request failed")?
            .error_for_status()
            .context("delete reference returned error status")?
            .json()
            .await
            .context("failed to parse delete reference response")
    }

    pub async fn rename_reference(
        &self,
        audio_id: &str,
        name: &str,
    ) -> Result<RenameResponse> {
        self.client
            .put(self.url(&format!("/references/{audio_id}/name")))
            .json(&RenameRequest {
                name: name.to_owned(),
            })
            .send()
            .await
            .context("rename reference request failed")?
            .error_for_status()
            .context("rename reference returned error status")?
            .json()
            .await
            .context("failed to parse rename reference response")
    }

    // ─── Voice Generation ───────────────────────────────────────

    pub async fn clone_voice(&self, request: &CloneRequest) -> Result<CloneResponse> {
        self.client
            .post(self.url("/clone"))
            .json(request)
            .send()
            .await
            .context("clone request failed")?
            .error_for_status()
            .context("clone returned error status")?
            .json()
            .await
            .context("failed to parse clone response")
    }

    pub async fn clone_with_upload(
        &self,
        file_bytes: Vec<u8>,
        filename: String,
        text: &str,
        ref_text: Option<&str>,
        language: Option<&str>,
    ) -> Result<CloneResponse> {
        let file_part = multipart::Part::bytes(file_bytes)
            .file_name(filename)
            .mime_str("audio/wav")
            .context("invalid mime type")?;

        let mut form = multipart::Form::new()
            .part("file", file_part)
            .text("text", text.to_owned());

        if let Some(rt) = ref_text {
            form = form.text("ref_text", rt.to_owned());
        }
        if let Some(lang) = language {
            form = form.text("language", lang.to_owned());
        }

        self.client
            .post(self.url("/clone-with-upload"))
            .multipart(form)
            .send()
            .await
            .context("clone-with-upload request failed")?
            .error_for_status()
            .context("clone-with-upload returned error status")?
            .json()
            .await
            .context("failed to parse clone-with-upload response")
    }

    pub async fn clone_multi_speaker(
        &self,
        request: &MultiSpeakerRequest,
    ) -> Result<CloneResponse> {
        self.client
            .post(self.url("/clone-multi-speaker"))
            .json(request)
            .send()
            .await
            .context("clone-multi-speaker request failed")?
            .error_for_status()
            .context("clone-multi-speaker returned error status")?
            .json()
            .await
            .context("failed to parse clone-multi-speaker response")
    }

    pub async fn voice_design(&self, request: &VoiceDesignRequest) -> Result<CloneResponse> {
        self.client
            .post(self.url("/voice-design"))
            .json(request)
            .send()
            .await
            .context("voice-design request failed")?
            .error_for_status()
            .context("voice-design returned error status")?
            .json()
            .await
            .context("failed to parse voice-design response")
    }

    pub async fn custom_voice(&self, request: &CustomVoiceRequest) -> Result<CloneResponse> {
        self.client
            .post(self.url("/custom-voice"))
            .json(request)
            .send()
            .await
            .context("custom-voice request failed")?
            .error_for_status()
            .context("custom-voice returned error status")?
            .json()
            .await
            .context("failed to parse custom-voice response")
    }

    // ─── Task Management ────────────────────────────────────────

    pub async fn task_status(&self, task_id: &str) -> Result<TaskStatusResponse> {
        self.client
            .get(self.url(&format!("/tasks/{task_id}")))
            .send()
            .await
            .context("task status request failed")?
            .error_for_status()
            .context("task status returned error status")?
            .json()
            .await
            .context("failed to parse task status response")
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<CancelResponse> {
        self.client
            .post(self.url(&format!("/tasks/{task_id}/cancel")))
            .send()
            .await
            .context("cancel task request failed")?
            .error_for_status()
            .context("cancel task returned error status")?
            .json()
            .await
            .context("failed to parse cancel task response")
    }

    pub async fn task_audio(&self, task_id: &str) -> Result<Vec<u8>> {
        self.client
            .get(self.url(&format!("/tasks/{task_id}/audio")))
            .send()
            .await
            .context("task audio request failed")?
            .error_for_status()
            .context("task audio returned error status")?
            .bytes()
            .await
            .context("failed to read task audio bytes")
            .map(|b| b.to_vec())
    }

    // ─── Generated Audio ────────────────────────────────────────

    pub async fn generated_list(&self) -> Result<Vec<GeneratedAudio>> {
        self.client
            .get(self.url("/generated"))
            .send()
            .await
            .context("generated list request failed")?
            .error_for_status()
            .context("generated list returned error status")?
            .json()
            .await
            .context("failed to parse generated list response")
    }

    pub async fn delete_generated(&self, audio_id: &str) -> Result<DeleteResponse> {
        self.client
            .delete(self.url(&format!("/generated/{audio_id}")))
            .send()
            .await
            .context("delete generated request failed")?
            .error_for_status()
            .context("delete generated returned error status")?
            .json()
            .await
            .context("failed to parse delete generated response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn health_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "healthy",
                "voice_cloner_loaded": true,
                "loaded_models": ["base"]
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client.health().await.expect("health should succeed");
        assert_eq!(resp.status, "healthy");
        assert!(resp.voice_cloner_loaded);
        assert_eq!(resp.loaded_models, vec!["base"]);
    }

    #[tokio::test]
    async fn health_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        assert!(client.health().await.is_err());
    }

    #[tokio::test]
    async fn capabilities_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/capabilities"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "models": ["base", "voice_design"],
                "speakers": ["Chelsie"]
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client.capabilities().await.expect("should succeed");
        assert_eq!(resp.models, vec!["base", "voice_design"]);
        assert_eq!(resp.speakers, vec!["Chelsie"]);
    }

    #[tokio::test]
    async fn languages_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/languages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "languages": ["auto", "English", "Japanese"]
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client.languages().await.expect("should succeed");
        assert_eq!(resp.languages, vec!["auto", "English", "Japanese"]);
    }

    #[tokio::test]
    async fn references_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/references"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": "uuid-1",
                    "filename": "uuid-1.wav",
                    "original_name": "sample.wav",
                    "name": null,
                    "ref_text": "hello",
                    "created_at": "1234567890.123"
                }
            ])))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let refs = client.references().await.expect("should succeed");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id, "uuid-1");
    }

    #[tokio::test]
    async fn clone_voice_success() {
        let server = MockServer::start().await;
        let request = CloneRequest {
            text: "Hello".to_owned(),
            ref_audio_id: "uuid-1".to_owned(),
            ref_text: None,
            language: "auto".to_owned(),
        };

        Mock::given(method("POST"))
            .and(path("/clone"))
            .and(body_json(serde_json::json!({
                "text": "Hello",
                "ref_audio_id": "uuid-1",
                "language": "auto"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "task_id": "task-1",
                "status": "processing",
                "output_path": null,
                "message": "Voice cloning started",
                "estimated_time": null
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client.clone_voice(&request).await.expect("should succeed");
        assert_eq!(resp.task_id, "task-1");
        assert_eq!(resp.status, "processing");
    }

    #[tokio::test]
    async fn task_status_completed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/tasks/task-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "completed",
                "progress": 100,
                "output_path": "output/cloned.wav",
                "ref_audio_id": "uuid-1",
                "generation_time_seconds": 5.5
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client.task_status("task-1").await.expect("should succeed");
        assert_eq!(resp.status, super::super::types::TaskStatus::Completed);
        assert_eq!(resp.progress, 100);
        assert_eq!(resp.generation_time_seconds, Some(5.5));
    }

    #[tokio::test]
    async fn cancel_task_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/tasks/task-1/cancel"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": "Task cancelled successfully"
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client.cancel_task("task-1").await.expect("should succeed");
        assert_eq!(resp.message, "Task cancelled successfully");
    }

    #[tokio::test]
    async fn task_audio_success() {
        let server = MockServer::start().await;
        let wav_bytes = b"RIFF fake wav data";
        Mock::given(method("GET"))
            .and(path("/tasks/task-1/audio"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(wav_bytes.to_vec()))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let data = client.task_audio("task-1").await.expect("should succeed");
        assert_eq!(&data[..], wav_bytes);
    }

    #[tokio::test]
    async fn generated_list_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/generated"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": "task-1",
                    "filename": "cloned_uuid.wav",
                    "ref_audio_id": "ref-1",
                    "ref_audio_name": "sample.wav",
                    "generated_text": "Hello",
                    "created_at": "1234567890.123",
                    "generation_time_seconds": 3.2
                }
            ])))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let list = client.generated_list().await.expect("should succeed");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "task-1");
    }

    #[tokio::test]
    async fn delete_reference_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/references/uuid-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": "Reference audio deleted successfully"
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client.delete_reference("uuid-1").await.expect("should succeed");
        assert_eq!(resp.message, "Reference audio deleted successfully");
    }

    #[tokio::test]
    async fn rename_reference_success() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/references/uuid-1/name"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": "Reference audio renamed successfully",
                "name": "My Voice"
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client
            .rename_reference("uuid-1", "My Voice")
            .await
            .expect("should succeed");
        assert_eq!(resp.name, "My Voice");
    }

    #[tokio::test]
    async fn delete_generated_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/generated/task-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": "Generated audio deleted successfully"
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client
            .delete_generated("task-1")
            .await
            .expect("should succeed");
        assert_eq!(resp.message, "Generated audio deleted successfully");
    }

    #[tokio::test]
    async fn voice_design_success() {
        let server = MockServer::start().await;
        let request = VoiceDesignRequest {
            text: "Hello".to_owned(),
            instruct: "A warm voice".to_owned(),
            language: "auto".to_owned(),
        };

        Mock::given(method("POST"))
            .and(path("/voice-design"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "task_id": "task-2",
                "status": "processing",
                "output_path": null,
                "message": "Voice design started",
                "estimated_time": null
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client.voice_design(&request).await.expect("should succeed");
        assert_eq!(resp.task_id, "task-2");
    }

    #[tokio::test]
    async fn custom_voice_success() {
        let server = MockServer::start().await;
        let request = CustomVoiceRequest {
            text: "Hello".to_owned(),
            speaker: "Chelsie".to_owned(),
            language: "auto".to_owned(),
            instruct: None,
        };

        Mock::given(method("POST"))
            .and(path("/custom-voice"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "task_id": "task-3",
                "status": "processing",
                "output_path": null,
                "message": "Custom voice started",
                "estimated_time": null
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client.custom_voice(&request).await.expect("should succeed");
        assert_eq!(resp.task_id, "task-3");
    }

    #[tokio::test]
    async fn multi_speaker_success() {
        let server = MockServer::start().await;
        let request = MultiSpeakerRequest {
            segments: vec![super::super::types::MultiSpeakerSegment {
                text: "Line 1".to_owned(),
                ref_audio_id: "uuid-1".to_owned(),
                ref_text: None,
                language: "auto".to_owned(),
            }],
        };

        Mock::given(method("POST"))
            .and(path("/clone-multi-speaker"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "task_id": "task-4",
                "status": "processing",
                "output_path": null,
                "message": "Multi-speaker started",
                "estimated_time": null
            })))
            .mount(&server)
            .await;

        let client = ApiClient::new(&server.uri());
        let resp = client
            .clone_multi_speaker(&request)
            .await
            .expect("should succeed");
        assert_eq!(resp.task_id, "task-4");
    }
}
