"""Shared test fixtures: FakeTTSEngine, FakeStorage, and test app."""

from __future__ import annotations

import time
import uuid
from pathlib import Path
from typing import TYPE_CHECKING

import numpy as np
import pytest
from httpx import ASGITransport, AsyncClient

from server.schemas import GeneratedAudioMeta, ReferenceAudioMeta
from server.tasks import TaskManager

if TYPE_CHECKING:
    from collections.abc import AsyncIterator

    from fastapi import FastAPI
    from numpy.typing import NDArray

# ─── Fake TTS Engine ────────────────────────────────────────────

FAKE_SAMPLE_RATE = 24000
FAKE_DURATION_SAMPLES = 2400  # 0.1 seconds at 24kHz


class FakeTTSEngine:
    """Test double that returns silence instead of real TTS output."""

    def __init__(
        self,
        models: list[str] | None = None,
        speakers: list[str] | None = None,
    ) -> None:
        self._models = models or ["base", "voice_design", "custom_voice"]
        self._speakers = speakers or ["Chelsie", "Aidan", "Aaliyah", "Ethan"]

    @property
    def loaded_models(self) -> list[str]:
        return list(self._models)

    @property
    def speakers(self) -> list[str]:
        return list(self._speakers)

    @property
    def is_ready(self) -> bool:
        return True

    def _fake_audio(self) -> tuple[NDArray[np.float32], int]:
        wav = np.zeros(FAKE_DURATION_SAMPLES, dtype=np.float32)
        return wav, FAKE_SAMPLE_RATE

    def generate_clone(
        self,
        text: str,
        ref_audio_path: Path,
        ref_text: str | None,
        language: str,
    ) -> tuple[NDArray[np.float32], int]:
        return self._fake_audio()

    def generate_voice_design(
        self,
        text: str,
        instruct: str,
        language: str,
    ) -> tuple[NDArray[np.float32], int]:
        return self._fake_audio()

    def generate_custom_voice(
        self,
        text: str,
        speaker: str,
        language: str,
        instruct: str | None,
    ) -> tuple[NDArray[np.float32], int]:
        return self._fake_audio()


# ─── Fake Storage ───────────────────────────────────────────────


class FakeStorage:
    """In-memory storage backend for testing."""

    def __init__(self) -> None:
        self._references: dict[str, tuple[ReferenceAudioMeta, bytes]] = {}
        self._generated: dict[str, tuple[GeneratedAudioMeta, bytes]] = {}

    def save_reference(
        self, audio_bytes: bytes, original_name: str, ref_text: str | None
    ) -> ReferenceAudioMeta:
        audio_id = uuid.uuid4().hex
        meta = ReferenceAudioMeta(
            id=audio_id,
            filename=f"{audio_id}.wav",
            original_name=original_name,
            ref_text=ref_text,
            created_at=str(time.time()),
        )
        self._references[audio_id] = (meta, audio_bytes)
        return meta

    def list_references(self) -> list[ReferenceAudioMeta]:
        return [meta for meta, _ in self._references.values()]

    def get_reference_path(self, audio_id: str) -> Path | None:
        if audio_id in self._references:
            return Path(f"/fake/references/{audio_id}.wav")
        return None

    def get_reference_audio(self, audio_id: str) -> bytes | None:
        entry = self._references.get(audio_id)
        if entry is None:
            return None
        return entry[1]

    def delete_reference(self, audio_id: str) -> bool:
        if audio_id not in self._references:
            return False
        del self._references[audio_id]
        return True

    def rename_reference(self, audio_id: str, name: str) -> ReferenceAudioMeta | None:
        entry = self._references.get(audio_id)
        if entry is None:
            return None
        meta, audio_bytes = entry
        updated = meta.model_copy(update={"name": name})
        self._references[audio_id] = (updated, audio_bytes)
        return updated

    def save_generated(
        self,
        task_id: str,
        wav_data: NDArray[np.float32],
        sr: int,
        meta: GeneratedAudioMeta,
    ) -> Path:
        # Convert numpy to bytes for storage
        wav_bytes = wav_data.tobytes()
        self._generated[task_id] = (meta, wav_bytes)
        return Path(f"/fake/generated/{task_id}.wav")

    def list_generated(self) -> list[GeneratedAudioMeta]:
        return [meta for meta, _ in self._generated.values()]

    def get_generated_audio(self, task_id: str) -> bytes | None:
        entry = self._generated.get(task_id)
        if entry is None:
            return None
        return entry[1]

    def delete_generated(self, audio_id: str) -> bool:
        if audio_id not in self._generated:
            return False
        del self._generated[audio_id]
        return True


# ─── Fixtures ───────────────────────────────────────────────────


@pytest.fixture
def fake_engine() -> FakeTTSEngine:
    return FakeTTSEngine()


@pytest.fixture
def fake_storage() -> FakeStorage:
    return FakeStorage()


@pytest.fixture
def test_app(fake_engine: FakeTTSEngine, fake_storage: FakeStorage) -> FastAPI:
    """Create a FastAPI app with fake dependencies injected."""
    from server.app import create_app

    app = create_app()
    app.state.engine = fake_engine  # type: ignore[attr-defined]
    app.state.storage = fake_storage  # type: ignore[attr-defined]
    app.state.task_manager = TaskManager()  # type: ignore[attr-defined]
    return app


@pytest.fixture
async def client(test_app: FastAPI) -> AsyncIterator[AsyncClient]:
    """Async HTTP client for testing."""
    transport = ASGITransport(app=test_app)  # type: ignore[arg-type]
    async with AsyncClient(transport=transport, base_url="http://test") as ac:
        yield ac
