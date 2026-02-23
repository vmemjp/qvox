"""Protocol definitions for dependency injection."""

from __future__ import annotations

from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    from pathlib import Path

    import numpy as np
    from numpy.typing import NDArray

    from server.schemas import GeneratedAudioMeta, ReferenceAudioMeta


class TTSEngine(Protocol):
    """Interface for TTS model engines."""

    @property
    def loaded_models(self) -> list[str]: ...

    @property
    def speakers(self) -> list[str]: ...

    @property
    def is_ready(self) -> bool: ...

    def generate_clone(
        self,
        text: str,
        ref_audio_path: Path,
        ref_text: str | None,
        language: str,
    ) -> tuple[NDArray[np.float32], int]: ...

    def generate_voice_design(
        self,
        text: str,
        instruct: str,
        language: str,
    ) -> tuple[NDArray[np.float32], int]: ...

    def generate_custom_voice(
        self,
        text: str,
        speaker: str,
        language: str,
        instruct: str | None,
    ) -> tuple[NDArray[np.float32], int]: ...


class StorageBackend(Protocol):
    """Interface for audio file storage."""

    def save_reference(
        self, audio_bytes: bytes, original_name: str, ref_text: str | None
    ) -> ReferenceAudioMeta: ...

    def list_references(self) -> list[ReferenceAudioMeta]: ...

    def get_reference_path(self, audio_id: str) -> Path | None: ...

    def get_reference_audio(self, audio_id: str) -> bytes | None: ...

    def delete_reference(self, audio_id: str) -> bool: ...

    def rename_reference(self, audio_id: str, name: str) -> ReferenceAudioMeta | None: ...

    def save_generated(
        self,
        task_id: str,
        wav_data: NDArray[np.float32],
        sr: int,
        meta: GeneratedAudioMeta,
    ) -> Path: ...

    def list_generated(self) -> list[GeneratedAudioMeta]: ...

    def get_generated_audio(self, task_id: str) -> bytes | None: ...

    def delete_generated(self, audio_id: str) -> bool: ...
