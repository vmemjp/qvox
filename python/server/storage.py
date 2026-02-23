"""FileStorage — file-based storage with JSON metadata sidecar files."""

from __future__ import annotations

import io
import json
import logging
import time
import uuid
from typing import TYPE_CHECKING

import soundfile as sf

from server.schemas import GeneratedAudioMeta, ReferenceAudioMeta

if TYPE_CHECKING:
    from pathlib import Path

    import numpy as np
    from numpy.typing import NDArray

logger = logging.getLogger(__name__)


class FileStorage:
    """File-system backed storage for reference and generated audio.

    Each audio file has a companion `.json` metadata sidecar.
    """

    def __init__(self, data_dir: Path) -> None:
        self._data_dir = data_dir
        self._ref_dir = data_dir / "references"
        self._gen_dir = data_dir / "generated"
        self._ref_dir.mkdir(parents=True, exist_ok=True)
        self._gen_dir.mkdir(parents=True, exist_ok=True)

    def save_reference(
        self, audio_bytes: bytes, original_name: str, ref_text: str | None
    ) -> ReferenceAudioMeta:
        audio_id = uuid.uuid4().hex
        filename = f"{audio_id}.wav"
        audio_path = self._ref_dir / filename

        audio_path.write_bytes(audio_bytes)

        meta = ReferenceAudioMeta(
            id=audio_id,
            filename=filename,
            original_name=original_name,
            ref_text=ref_text,
            created_at=str(time.time()),
        )

        meta_path = self._ref_dir / f"{audio_id}.json"
        meta_path.write_text(meta.model_dump_json(), encoding="utf-8")
        return meta

    def list_references(self) -> list[ReferenceAudioMeta]:
        results: list[ReferenceAudioMeta] = []
        for meta_path in sorted(self._ref_dir.glob("*.json")):
            try:
                data = json.loads(meta_path.read_text(encoding="utf-8"))
                results.append(ReferenceAudioMeta.model_validate(data))
            except (json.JSONDecodeError, ValueError):
                logger.warning("Skipping invalid metadata: %s", meta_path)
        return results

    def get_reference_path(self, audio_id: str) -> Path | None:
        path = self._ref_dir / f"{audio_id}.wav"
        return path if path.exists() else None

    def get_reference_audio(self, audio_id: str) -> bytes | None:
        path = self.get_reference_path(audio_id)
        if path is None:
            return None
        return path.read_bytes()

    def delete_reference(self, audio_id: str) -> bool:
        audio_path = self._ref_dir / f"{audio_id}.wav"
        meta_path = self._ref_dir / f"{audio_id}.json"
        if not audio_path.exists():
            return False
        audio_path.unlink(missing_ok=True)
        meta_path.unlink(missing_ok=True)
        return True

    def rename_reference(self, audio_id: str, name: str) -> ReferenceAudioMeta | None:
        meta_path = self._ref_dir / f"{audio_id}.json"
        if not meta_path.exists():
            return None
        data = json.loads(meta_path.read_text(encoding="utf-8"))
        meta = ReferenceAudioMeta.model_validate(data)
        meta = meta.model_copy(update={"name": name})
        meta_path.write_text(meta.model_dump_json(), encoding="utf-8")
        return meta

    def save_generated(
        self,
        task_id: str,
        wav_data: NDArray[np.float32],
        sr: int,
        meta: GeneratedAudioMeta,
    ) -> Path:
        filename = f"{task_id}.wav"
        audio_path = self._gen_dir / filename

        buf = io.BytesIO()
        sf.write(buf, wav_data, sr, format="WAV", subtype="FLOAT")
        audio_path.write_bytes(buf.getvalue())

        meta_path = self._gen_dir / f"{task_id}.json"
        meta_path.write_text(meta.model_dump_json(), encoding="utf-8")
        return audio_path

    def list_generated(self) -> list[GeneratedAudioMeta]:
        results: list[GeneratedAudioMeta] = []
        for meta_path in sorted(self._gen_dir.glob("*.json")):
            try:
                data = json.loads(meta_path.read_text(encoding="utf-8"))
                results.append(GeneratedAudioMeta.model_validate(data))
            except (json.JSONDecodeError, ValueError):
                logger.warning("Skipping invalid metadata: %s", meta_path)
        return results

    def get_generated_audio(self, task_id: str) -> bytes | None:
        path = self._gen_dir / f"{task_id}.wav"
        if not path.exists():
            return None
        return path.read_bytes()

    def delete_generated(self, audio_id: str) -> bool:
        audio_path = self._gen_dir / f"{audio_id}.wav"
        meta_path = self._gen_dir / f"{audio_id}.json"
        if not audio_path.exists():
            return False
        audio_path.unlink(missing_ok=True)
        meta_path.unlink(missing_ok=True)
        return True
