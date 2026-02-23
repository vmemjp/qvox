"""Generated audio list and deletion endpoints."""

from __future__ import annotations

from fastapi import APIRouter, HTTPException, Request

from server.app import get_storage
from server.schemas import DeleteResponse, GeneratedAudioMeta

router = APIRouter()


@router.get("/generated")
def list_generated(request: Request) -> list[GeneratedAudioMeta]:
    """List all generated audio files."""
    storage = get_storage(request)
    return storage.list_generated()


@router.delete("/generated/{audio_id}")
def delete_generated(request: Request, audio_id: str) -> DeleteResponse:
    """Delete a generated audio file."""
    storage = get_storage(request)
    if not storage.delete_generated(audio_id):
        raise HTTPException(status_code=404, detail="Generated audio not found")
    return DeleteResponse(message="Generated audio deleted successfully")
