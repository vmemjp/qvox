"""Pydantic V2 request/response models with strict validation."""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, Field, field_validator

# ─── Supported values ───────────────────────────────────────────

SUPPORTED_LANGUAGES: list[str] = [
    "auto",
    "Chinese",
    "English",
    "Japanese",
    "Korean",
    "German",
    "French",
    "Russian",
    "Portuguese",
    "Spanish",
    "Italian",
]

SUPPORTED_SPEAKERS: list[str] = [
    "Vivian",
    "Serena",
    "Uncle_Fu",
    "Dylan",
    "Eric",
    "Ryan",
    "Aiden",
    "Ono_Anna",
    "Sohee",
]


def _validate_language(v: str) -> str:
    if v not in SUPPORTED_LANGUAGES:
        msg = f"Unsupported language: {v}. Must be one of: {SUPPORTED_LANGUAGES}"
        raise ValueError(msg)
    return v


def _validate_speaker(v: str) -> str:
    if v not in SUPPORTED_SPEAKERS:
        msg = f"Unsupported speaker: {v}. Must be one of: {SUPPORTED_SPEAKERS}"
        raise ValueError(msg)
    return v


# ─── Reference Audio ────────────────────────────────────────────


class ReferenceAudioMeta(BaseModel):
    """Metadata for a reference audio file."""

    id: str
    filename: str
    original_name: str
    name: str | None = None
    ref_text: str | None = None
    created_at: str


# ─── Generated Audio ────────────────────────────────────────────


class GeneratedAudioMeta(BaseModel):
    """Metadata for a generated audio file."""

    id: str
    filename: str
    ref_audio_id: str | None = None
    ref_audio_name: str | None = None
    generated_text: str
    created_at: str
    generation_time_seconds: float | None = None


# ─── Health / Capabilities ──────────────────────────────────────


class HealthResponse(BaseModel):
    """Response from GET /health."""

    status: str
    voice_cloner_loaded: bool
    loaded_models: list[str]


class CapabilitiesResponse(BaseModel):
    """Response from GET /capabilities."""

    models: list[str]
    speakers: list[str]


class LanguagesResponse(BaseModel):
    """Response from GET /languages."""

    languages: list[str]


# ─── Voice Generation Requests ──────────────────────────────────


class CloneRequest(BaseModel):
    """Request body for POST /clone."""

    text: str = Field(min_length=1, max_length=10000)
    ref_audio_id: str = Field(min_length=1, max_length=200)
    ref_text: str | None = Field(default=None, max_length=10000)
    language: str = Field(default="auto")

    @field_validator("language")
    @classmethod
    def check_language(cls, v: str) -> str:
        return _validate_language(v)


class MultiSpeakerSegment(BaseModel):
    """A single segment in a multi-speaker request."""

    text: str = Field(min_length=1, max_length=10000)
    ref_audio_id: str = Field(min_length=1, max_length=200)
    ref_text: str | None = Field(default=None, max_length=10000)
    language: str = Field(default="auto")

    @field_validator("language")
    @classmethod
    def check_language(cls, v: str) -> str:
        return _validate_language(v)


class MultiSpeakerRequest(BaseModel):
    """Request body for POST /clone-multi-speaker."""

    segments: list[MultiSpeakerSegment] = Field(min_length=1, max_length=100)


class VoiceDesignRequest(BaseModel):
    """Request body for POST /voice-design."""

    text: str = Field(min_length=1, max_length=10000)
    instruct: str = Field(min_length=1, max_length=1000)
    language: str = Field(default="auto")

    @field_validator("language")
    @classmethod
    def check_language(cls, v: str) -> str:
        return _validate_language(v)


class CustomVoiceRequest(BaseModel):
    """Request body for POST /custom-voice."""

    text: str = Field(min_length=1, max_length=10000)
    speaker: str = Field(min_length=1)
    language: str = Field(default="auto")
    instruct: str | None = Field(default=None, max_length=1000)

    @field_validator("language")
    @classmethod
    def check_language(cls, v: str) -> str:
        return _validate_language(v)

    @field_validator("speaker")
    @classmethod
    def check_speaker(cls, v: str) -> str:
        return _validate_speaker(v)


# ─── Voice Generation Response ──────────────────────────────────


class CloneResponse(BaseModel):
    """Shared response for all voice generation endpoints."""

    task_id: str
    status: str
    output_path: str | None = None
    message: str
    estimated_time: float | None = None


# ─── Task Management ────────────────────────────────────────────


class TaskStatusResponse(BaseModel):
    """Response from GET /tasks/{task_id}."""

    status: Literal["processing", "completed", "failed", "cancelled"]
    progress: int = Field(ge=0, le=100)
    output_path: str | None = None
    ref_audio_id: str | None = None
    generation_time_seconds: float | None = None
    error: str | None = None
    is_multi_speaker: bool | None = None
    total_segments: int | None = None
    current_segment: int | None = None


class CancelResponse(BaseModel):
    """Response from POST /tasks/{task_id}/cancel."""

    message: str


# ─── Delete / Rename ────────────────────────────────────────────


class DeleteResponse(BaseModel):
    """Response from DELETE endpoints."""

    message: str


class RenameRequest(BaseModel):
    """Request body for PUT /references/{audio_id}/name."""

    name: str = Field(min_length=1, max_length=200)


class RenameResponse(BaseModel):
    """Response from PUT /references/{audio_id}/name."""

    message: str
    name: str
