"""FileStorage tests using tmp_path fixture."""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
import pytest

from server.schemas import GeneratedAudioMeta
from server.storage import FileStorage

if TYPE_CHECKING:
    from pathlib import Path


@pytest.fixture
def storage(tmp_path: Path) -> FileStorage:
    return FileStorage(data_dir=tmp_path)


FAKE_WAV_BYTES = b"RIFF" + b"\x00" * 40  # minimal placeholder


class TestReferenceStorage:
    def test_save_and_list(self, storage: FileStorage) -> None:
        meta = storage.save_reference(FAKE_WAV_BYTES, "test.wav", "hello")
        assert meta.original_name == "test.wav"
        assert meta.ref_text == "hello"
        assert meta.name is None

        refs = storage.list_references()
        assert len(refs) == 1
        assert refs[0].id == meta.id

    def test_save_without_ref_text(self, storage: FileStorage) -> None:
        meta = storage.save_reference(FAKE_WAV_BYTES, "test.wav", None)
        assert meta.ref_text is None

    def test_get_reference_path(self, storage: FileStorage) -> None:
        meta = storage.save_reference(FAKE_WAV_BYTES, "test.wav", None)
        path = storage.get_reference_path(meta.id)
        assert path is not None
        assert path.exists()

    def test_get_reference_path_not_found(self, storage: FileStorage) -> None:
        assert storage.get_reference_path("nonexistent") is None

    def test_get_reference_audio(self, storage: FileStorage) -> None:
        meta = storage.save_reference(FAKE_WAV_BYTES, "test.wav", None)
        audio = storage.get_reference_audio(meta.id)
        assert audio == FAKE_WAV_BYTES

    def test_get_reference_audio_not_found(self, storage: FileStorage) -> None:
        assert storage.get_reference_audio("nonexistent") is None

    def test_delete_reference(self, storage: FileStorage) -> None:
        meta = storage.save_reference(FAKE_WAV_BYTES, "test.wav", None)
        assert storage.delete_reference(meta.id) is True
        assert storage.list_references() == []
        assert storage.get_reference_path(meta.id) is None

    def test_delete_nonexistent_returns_false(self, storage: FileStorage) -> None:
        assert storage.delete_reference("nonexistent") is False

    def test_rename_reference(self, storage: FileStorage) -> None:
        meta = storage.save_reference(FAKE_WAV_BYTES, "test.wav", None)
        updated = storage.rename_reference(meta.id, "My Voice")
        assert updated is not None
        assert updated.name == "My Voice"

        # Verify persisted
        refs = storage.list_references()
        assert refs[0].name == "My Voice"

    def test_rename_nonexistent_returns_none(self, storage: FileStorage) -> None:
        assert storage.rename_reference("nonexistent", "name") is None

    def test_multiple_references(self, storage: FileStorage) -> None:
        for i in range(5):
            storage.save_reference(FAKE_WAV_BYTES, f"test{i}.wav", None)
        assert len(storage.list_references()) == 5


class TestGeneratedStorage:
    def test_save_and_list(self, storage: FileStorage) -> None:
        wav = np.zeros(2400, dtype=np.float32)
        meta = GeneratedAudioMeta(
            id="task-1",
            filename="task-1.wav",
            generated_text="hello",
            created_at="123.456",
        )
        path = storage.save_generated("task-1", wav, 24000, meta)
        assert path.exists()

        items = storage.list_generated()
        assert len(items) == 1
        assert items[0].id == "task-1"

    def test_get_generated_audio(self, storage: FileStorage) -> None:
        wav = np.zeros(2400, dtype=np.float32)
        meta = GeneratedAudioMeta(
            id="task-1",
            filename="task-1.wav",
            generated_text="hello",
            created_at="123.456",
        )
        storage.save_generated("task-1", wav, 24000, meta)
        audio = storage.get_generated_audio("task-1")
        assert audio is not None
        assert len(audio) > 0

    def test_get_generated_audio_not_found(self, storage: FileStorage) -> None:
        assert storage.get_generated_audio("nonexistent") is None

    def test_delete_generated(self, storage: FileStorage) -> None:
        wav = np.zeros(2400, dtype=np.float32)
        meta = GeneratedAudioMeta(
            id="task-1",
            filename="task-1.wav",
            generated_text="hello",
            created_at="123.456",
        )
        storage.save_generated("task-1", wav, 24000, meta)
        assert storage.delete_generated("task-1") is True
        assert storage.list_generated() == []

    def test_delete_nonexistent_returns_false(self, storage: FileStorage) -> None:
        assert storage.delete_generated("nonexistent") is False

    def test_wav_is_valid_soundfile(self, storage: FileStorage) -> None:
        """Verify saved WAV can be read back by soundfile."""
        import soundfile as sf

        wav = np.random.default_rng(42).random(4800, dtype=np.float64).astype(np.float32)
        meta = GeneratedAudioMeta(
            id="task-sf",
            filename="task-sf.wav",
            generated_text="test",
            created_at="0.0",
        )
        path = storage.save_generated("task-sf", wav, 24000, meta)
        read_result = sf.read(path, dtype="float32")  # pyright: ignore[reportUnknownVariableType]
        data = np.asarray(read_result[0], dtype=np.float32)
        sr: int = read_result[1]  # pyright: ignore[reportUnknownMemberType]
        assert sr == 24000
        assert len(data) == 4800
        np.testing.assert_allclose(data, wav, atol=1e-6)
