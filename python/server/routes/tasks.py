"""Task management endpoints."""

from __future__ import annotations

from fastapi import APIRouter, HTTPException, Request
from fastapi.responses import Response

from server.app import get_storage, get_task_manager
from server.schemas import CancelResponse, TaskStatusResponse

router = APIRouter()


@router.get("/tasks/{task_id}")
def task_status(request: Request, task_id: str) -> TaskStatusResponse:
    """Get the status of a generation task."""
    task_manager = get_task_manager(request)
    state = task_manager.get(task_id)
    if state is None:
        raise HTTPException(status_code=404, detail="Task not found")

    return TaskStatusResponse(
        status=state.status,  # type: ignore[arg-type]
        progress=state.progress,
        output_path=state.output_path,
        ref_audio_id=state.ref_audio_id,
        generation_time_seconds=state.generation_time_seconds,
        error=state.error,
        is_multi_speaker=state.is_multi_speaker,
        total_segments=state.total_segments,
        current_segment=state.current_segment,
    )


@router.post("/tasks/{task_id}/cancel")
def cancel_task(request: Request, task_id: str) -> CancelResponse:
    """Cancel a running generation task."""
    task_manager = get_task_manager(request)
    if task_manager.get(task_id) is None:
        raise HTTPException(status_code=404, detail="Task not found")
    task_manager.cancel(task_id)
    return CancelResponse(message="Task cancelled successfully")


@router.get("/tasks/{task_id}/audio")
def task_audio(request: Request, task_id: str) -> Response:
    """Download the generated audio for a completed task."""
    task_manager = get_task_manager(request)
    state = task_manager.get(task_id)
    if state is not None and state.status != "completed":
        raise HTTPException(status_code=400, detail="Task not completed")

    # Fall back to storage directly — task state is ephemeral but files persist
    storage = get_storage(request)
    audio_bytes = storage.get_generated_audio(task_id)
    if audio_bytes is None:
        raise HTTPException(status_code=404, detail="Audio file not found")

    return Response(content=audio_bytes, media_type="audio/wav")
