"""QwenTTSEngine — wraps qwen_tts with dynamic VRAM model swapping."""

from __future__ import annotations

import logging
import threading
from typing import TYPE_CHECKING, Any

import numpy as np

if TYPE_CHECKING:
    from pathlib import Path

    from numpy.typing import NDArray

logger = logging.getLogger(__name__)

# ─── Model name mapping ────────────────────────────────────────

MODEL_REGISTRY: dict[str, dict[str, str]] = {
    "1.7B": {
        "base": "Qwen/Qwen3-TTS-12Hz-1.7B-Base",
        "voice_design": "Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign",
        "custom_voice": "Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice",
    },
    "0.6B": {
        "base": "Qwen/Qwen3-TTS-12Hz-0.6B-Base",
        "voice_design": "Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign",  # 1.7B only
        "custom_voice": "Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice",
    },
}


def resolve_model_name(model_type: str, model_size: str) -> str:
    """Resolve CLI model type + size to a HuggingFace model name."""
    size_map = MODEL_REGISTRY.get(model_size)
    if size_map is None:
        msg = f"Unknown model size: {model_size}"
        raise ValueError(msg)
    name = size_map.get(model_type)
    if name is None:
        msg = f"Unknown model type: {model_type}"
        raise ValueError(msg)
    return name


# ─── Engine ─────────────────────────────────────────────────────


class QwenTTSEngine:
    """TTS engine with dynamic VRAM model swapping.

    Only one model is kept in VRAM at a time. When a different model type
    is requested, the current model is unloaded and the new one is loaded.
    A threading lock ensures GPU exclusivity during inference.
    """

    def __init__(
        self,
        model_names: list[str],
        device: str,
        model_size: str = "1.7B",
    ) -> None:
        self._available_models = list(model_names)
        self._device = device
        self._model_size = model_size
        self._lock = threading.Lock()
        self._current_model_type: str | None = None
        self._model: Any = None

    @property
    def loaded_models(self) -> list[str]:
        return list(self._available_models)

    @property
    def speakers(self) -> list[str]:
        return ["Chelsie", "Aidan", "Aaliyah", "Ethan"]

    @property
    def is_ready(self) -> bool:
        return True

    def _ensure_model(self, model_type: str) -> Any:
        """Load the required model, swapping if necessary. Must hold _lock."""
        if model_type not in self._available_models:
            msg = f"Model '{model_type}' is not available. Available: {self._available_models}"
            raise ValueError(msg)

        if self._current_model_type == model_type and self._model is not None:
            return self._model

        import torch  # pyright: ignore[reportMissingImports]
        from qwen_tts import Qwen3TTSModel  # type: ignore[import-untyped]

        # Unload current model
        if self._model is not None:
            logger.info("Unloading model: %s", self._current_model_type)
            del self._model
            self._model = None
            self._current_model_type = None
            torch.cuda.empty_cache()

        hf_name = resolve_model_name(model_type, self._model_size)
        logger.info("Loading model: %s (%s)", model_type, hf_name)

        device = self._device
        if device == "auto":
            device = "cuda" if torch.cuda.is_available() else "cpu"

        model: object = Qwen3TTSModel.from_pretrained(  # pyright: ignore[reportUnknownVariableType,reportUnknownMemberType]
            hf_name, device=device
        )
        self._model = model
        self._current_model_type = model_type
        return model  # pyright: ignore[reportUnknownVariableType]

    def generate_clone(
        self,
        text: str,
        ref_audio_path: Path,
        ref_text: str | None,
        language: str,
    ) -> tuple[NDArray[np.float32], int]:
        with self._lock:
            model = self._ensure_model("base")
            wav, sr = model.synthesize(
                text=text,
                ref_audio=str(ref_audio_path),
                ref_text=ref_text or "",
                lang=language if language != "auto" else None,
            )
            return np.asarray(wav, dtype=np.float32), int(sr)

    def generate_voice_design(
        self,
        text: str,
        instruct: str,
        language: str,
    ) -> tuple[NDArray[np.float32], int]:
        with self._lock:
            model = self._ensure_model("voice_design")
            wav, sr = model.synthesize(
                text=text,
                instruct=instruct,
                lang=language if language != "auto" else None,
            )
            return np.asarray(wav, dtype=np.float32), int(sr)

    def generate_custom_voice(
        self,
        text: str,
        speaker: str,
        language: str,
        instruct: str | None,
    ) -> tuple[NDArray[np.float32], int]:
        with self._lock:
            model = self._ensure_model("custom_voice")
            kwargs: dict[str, str | None] = {
                "text": text,
                "speaker": speaker,
                "lang": language if language != "auto" else None,
            }
            if instruct is not None:
                kwargs["instruct"] = instruct
            wav, sr = model.synthesize(**kwargs)
            return np.asarray(wav, dtype=np.float32), int(sr)
