"""Voice generation endpoints (clone, voice-design, custom-voice, multi-speaker)."""

from __future__ import annotations

import asyncio
import logging
import time
import uuid
from typing import TYPE_CHECKING

import numpy as np
from fastapi import APIRouter, Form, HTTPException, Request, UploadFile

from server.app import get_engine, get_storage, get_task_manager
from server.schemas import (
    CloneRequest,
    CloneResponse,
    CustomVoiceRequest,
    GeneratedAudioMeta,
    MultiSpeakerRequest,
    VoiceDesignRequest,
)

if TYPE_CHECKING:
    from server.tasks import TaskState

logger = logging.getLogger(__name__)

router = APIRouter()


def _make_task_id() -> str:
    return uuid.uuid4().hex


# ─── Task coroutines ────────────────────────────────────────────


async def _run_clone(
    state: TaskState,
    request: Request,
    text: str,
    ref_audio_id: str,
    ref_text: str | None,
    language: str,
) -> None:
    """Execute a clone generation task."""
    engine = get_engine(request)
    storage = get_storage(request)

    ref_path = storage.get_reference_path(ref_audio_id)
    if ref_path is None:
        state.status = "failed"
        state.error = "Reference audio not found"
        return

    ref_name: str | None = None
    for ref in storage.list_references():
        if ref.id == ref_audio_id:
            ref_name = ref.name or ref.original_name
            break

    state.progress = 10
    start = time.monotonic()

    wav, sr = await asyncio.to_thread(
        engine.generate_clone, text, ref_path, ref_text, language
    )

    elapsed = time.monotonic() - start
    state.progress = 90

    meta = GeneratedAudioMeta(
        id=state.task_id,
        filename=f"{state.task_id}.wav",
        ref_audio_id=ref_audio_id,
        ref_audio_name=ref_name,
        generated_text=text,
        created_at=str(time.time()),
        generation_time_seconds=round(elapsed, 2),
    )

    storage.save_generated(state.task_id, wav, sr, meta)

    state.status = "completed"
    state.progress = 100
    state.output_path = meta.filename
    state.generation_time_seconds = round(elapsed, 2)


async def _run_voice_design(
    state: TaskState,
    request: Request,
    text: str,
    instruct: str,
    language: str,
) -> None:
    """Execute a voice design generation task."""
    engine = get_engine(request)
    storage = get_storage(request)

    state.progress = 10
    start = time.monotonic()

    wav, sr = await asyncio.to_thread(
        engine.generate_voice_design, text, instruct, language
    )

    elapsed = time.monotonic() - start
    state.progress = 90

    meta = GeneratedAudioMeta(
        id=state.task_id,
        filename=f"{state.task_id}.wav",
        generated_text=text,
        created_at=str(time.time()),
        generation_time_seconds=round(elapsed, 2),
    )

    storage.save_generated(state.task_id, wav, sr, meta)

    state.status = "completed"
    state.progress = 100
    state.output_path = meta.filename
    state.generation_time_seconds = round(elapsed, 2)


async def _run_custom_voice(
    state: TaskState,
    request: Request,
    text: str,
    speaker: str,
    language: str,
    instruct: str | None,
) -> None:
    """Execute a custom voice generation task."""
    engine = get_engine(request)
    storage = get_storage(request)

    state.progress = 10
    start = time.monotonic()

    wav, sr = await asyncio.to_thread(
        engine.generate_custom_voice, text, speaker, language, instruct
    )

    elapsed = time.monotonic() - start
    state.progress = 90

    meta = GeneratedAudioMeta(
        id=state.task_id,
        filename=f"{state.task_id}.wav",
        generated_text=text,
        created_at=str(time.time()),
        generation_time_seconds=round(elapsed, 2),
    )

    storage.save_generated(state.task_id, wav, sr, meta)

    state.status = "completed"
    state.progress = 100
    state.output_path = meta.filename
    state.generation_time_seconds = round(elapsed, 2)


async def _run_multi_speaker(
    state: TaskState,
    request: Request,
    segments: list[dict[str, str | None]],
) -> None:
    """Execute a multi-speaker clone generation task."""
    engine = get_engine(request)
    storage = get_storage(request)

    all_wavs: list[tuple[np.ndarray[tuple[int], np.dtype[np.float32]], int]] = []
    total = len(segments)
    start = time.monotonic()
    combined_text_parts: list[str] = []

    for i, seg in enumerate(segments):
        state.current_segment = i + 1
        state.progress = int((i / total) * 90)

        ref_audio_id = seg["ref_audio_id"]
        if ref_audio_id is None:
            state.status = "failed"
            state.error = f"Segment {i}: missing ref_audio_id"
            return

        ref_path = storage.get_reference_path(ref_audio_id)
        if ref_path is None:
            state.status = "failed"
            state.error = f"Segment {i}: reference audio not found"
            return

        text = seg.get("text") or ""
        ref_text = seg.get("ref_text")
        language = seg.get("language") or "auto"
        combined_text_parts.append(text)

        wav, sr = await asyncio.to_thread(
            engine.generate_clone, text, ref_path, ref_text, language
        )
        all_wavs.append((wav, sr))

    elapsed = time.monotonic() - start
    state.progress = 90

    if not all_wavs:
        state.status = "failed"
        state.error = "No audio generated"
        return

    target_sr = all_wavs[0][1]
    combined = np.concatenate([w for w, _ in all_wavs])

    combined_text = " ".join(combined_text_parts)
    meta = GeneratedAudioMeta(
        id=state.task_id,
        filename=f"{state.task_id}.wav",
        generated_text=combined_text,
        created_at=str(time.time()),
        generation_time_seconds=round(elapsed, 2),
    )

    storage.save_generated(state.task_id, combined, target_sr, meta)

    state.status = "completed"
    state.progress = 100
    state.output_path = meta.filename
    state.generation_time_seconds = round(elapsed, 2)


# ─── Endpoints ──────────────────────────────────────────────────


@router.post("/clone")
async def clone(request: Request, body: CloneRequest) -> CloneResponse:
    """Start a voice cloning task."""
    storage = get_storage(request)
    if storage.get_reference_path(body.ref_audio_id) is None:
        raise HTTPException(status_code=404, detail="Reference audio not found")

    task_manager = get_task_manager(request)
    task_id = _make_task_id()

    state = task_manager.register(task_id, ref_audio_id=body.ref_audio_id)
    task_manager.start(
        state,
        _run_clone(state, request, body.text, body.ref_audio_id, body.ref_text, body.language),
    )

    return CloneResponse(
        task_id=task_id,
        status="processing",
        message="Voice cloning started",
    )


@router.post("/clone-with-upload")
async def clone_with_upload(
    request: Request,
    file: UploadFile,
    text: str = Form(),
    ref_text: str | None = Form(default=None),
    language: str = Form(default="auto"),
) -> CloneResponse:
    """Upload reference audio and start cloning in one step."""
    storage = get_storage(request)
    audio_bytes = await file.read()
    if len(audio_bytes) == 0:
        raise HTTPException(status_code=400, detail="Empty audio file")

    original_name = file.filename or "unknown.wav"
    ref_meta = storage.save_reference(audio_bytes, original_name, ref_text)

    task_manager = get_task_manager(request)
    task_id = _make_task_id()

    state = task_manager.register(task_id, ref_audio_id=ref_meta.id)
    task_manager.start(
        state,
        _run_clone(state, request, text, ref_meta.id, ref_text, language),
    )

    return CloneResponse(
        task_id=task_id,
        status="processing",
        message="Voice cloning started",
    )


@router.post("/clone-multi-speaker")
async def clone_multi_speaker(request: Request, body: MultiSpeakerRequest) -> CloneResponse:
    """Start a multi-speaker clone task."""
    storage = get_storage(request)

    for i, seg in enumerate(body.segments):
        if storage.get_reference_path(seg.ref_audio_id) is None:
            raise HTTPException(
                status_code=404,
                detail=f"Segment {i}: reference audio '{seg.ref_audio_id}' not found",
            )

    task_manager = get_task_manager(request)
    task_id = _make_task_id()

    segments = [
        {
            "text": seg.text,
            "ref_audio_id": seg.ref_audio_id,
            "ref_text": seg.ref_text,
            "language": seg.language,
        }
        for seg in body.segments
    ]

    state = task_manager.register(
        task_id, is_multi_speaker=True, total_segments=len(body.segments)
    )
    task_manager.start(state, _run_multi_speaker(state, request, segments))

    return CloneResponse(
        task_id=task_id,
        status="processing",
        message="Multi-speaker cloning started",
    )


@router.post("/voice-design")
async def voice_design(request: Request, body: VoiceDesignRequest) -> CloneResponse:
    """Start a voice design generation task."""
    task_manager = get_task_manager(request)
    task_id = _make_task_id()

    state = task_manager.register(task_id)
    task_manager.start(
        state,
        _run_voice_design(state, request, body.text, body.instruct, body.language),
    )

    return CloneResponse(
        task_id=task_id,
        status="processing",
        message="Voice design started",
    )


@router.post("/custom-voice")
async def custom_voice(request: Request, body: CustomVoiceRequest) -> CloneResponse:
    """Start a custom voice generation task."""
    task_manager = get_task_manager(request)
    task_id = _make_task_id()

    state = task_manager.register(task_id)
    task_manager.start(
        state,
        _run_custom_voice(
            state, request, body.text, body.speaker, body.language, body.instruct
        ),
    )

    return CloneResponse(
        task_id=task_id,
        status="processing",
        message="Custom voice generation started",
    )
