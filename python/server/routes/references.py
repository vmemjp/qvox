"""Reference audio CRUD endpoints."""

from __future__ import annotations

from fastapi import APIRouter, Form, HTTPException, Request, UploadFile
from fastapi.responses import Response

from server.app import get_storage
from server.schemas import (
    DeleteResponse,
    ReferenceAudioMeta,
    RenameRequest,
    RenameResponse,
)

router = APIRouter()


@router.get("/references")
def list_references(request: Request) -> list[ReferenceAudioMeta]:
    """List all reference audio files."""
    storage = get_storage(request)
    return storage.list_references()


@router.post("/upload-reference")
async def upload_reference(
    request: Request,
    file: UploadFile,
    ref_text: str | None = Form(default=None),
) -> ReferenceAudioMeta:
    """Upload a new reference audio file."""
    storage = get_storage(request)
    audio_bytes = await file.read()
    if len(audio_bytes) == 0:
        raise HTTPException(status_code=400, detail="Empty audio file")
    original_name = file.filename or "unknown.wav"
    return storage.save_reference(audio_bytes, original_name, ref_text)


@router.get("/references/{audio_id}/audio")
def get_reference_audio(request: Request, audio_id: str) -> Response:
    """Download reference audio by ID."""
    storage = get_storage(request)
    audio_bytes = storage.get_reference_audio(audio_id)
    if audio_bytes is None:
        raise HTTPException(status_code=404, detail="Reference audio not found")
    return Response(content=audio_bytes, media_type="audio/wav")


@router.delete("/references/{audio_id}")
def delete_reference(request: Request, audio_id: str) -> DeleteResponse:
    """Delete a reference audio file."""
    storage = get_storage(request)
    if not storage.delete_reference(audio_id):
        raise HTTPException(status_code=404, detail="Reference audio not found")
    return DeleteResponse(message="Reference audio deleted successfully")


@router.put("/references/{audio_id}/name")
def rename_reference(
    request: Request, audio_id: str, body: RenameRequest
) -> RenameResponse:
    """Rename a reference audio file."""
    storage = get_storage(request)
    meta = storage.rename_reference(audio_id, body.name)
    if meta is None:
        raise HTTPException(status_code=404, detail="Reference audio not found")
    return RenameResponse(
        message="Reference audio renamed successfully",
        name=body.name,
    )
