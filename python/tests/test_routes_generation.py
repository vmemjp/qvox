"""Voice generation route tests."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from httpx import AsyncClient

FAKE_WAV = b"RIFF" + b"\x00" * 40


async def _upload_ref(client: AsyncClient) -> str:
    """Helper: upload a reference audio and return its ID."""
    resp = await client.post(
        "/upload-reference",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
    )
    return resp.json()["id"]


@pytest.mark.asyncio
async def test_clone(client: AsyncClient) -> None:
    ref_id = await _upload_ref(client)
    resp = await client.post(
        "/clone",
        json={"text": "Hello world", "ref_audio_id": ref_id},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "processing"
    assert "task_id" in data
    assert data["message"] == "Voice cloning started"


@pytest.mark.asyncio
async def test_clone_ref_not_found(client: AsyncClient) -> None:
    resp = await client.post(
        "/clone",
        json={"text": "Hello", "ref_audio_id": "nonexistent"},
    )
    assert resp.status_code == 404


@pytest.mark.asyncio
async def test_clone_empty_text_rejected(client: AsyncClient) -> None:
    ref_id = await _upload_ref(client)
    resp = await client.post(
        "/clone",
        json={"text": "", "ref_audio_id": ref_id},
    )
    assert resp.status_code == 422


@pytest.mark.asyncio
async def test_clone_invalid_language_rejected(client: AsyncClient) -> None:
    ref_id = await _upload_ref(client)
    resp = await client.post(
        "/clone",
        json={"text": "Hello", "ref_audio_id": ref_id, "language": "Klingon"},
    )
    assert resp.status_code == 422


@pytest.mark.asyncio
async def test_clone_with_upload(client: AsyncClient) -> None:
    resp = await client.post(
        "/clone-with-upload",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
        data={"text": "Hello world"},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "processing"
    assert "task_id" in data


@pytest.mark.asyncio
async def test_clone_with_upload_empty_file(client: AsyncClient) -> None:
    resp = await client.post(
        "/clone-with-upload",
        files={"file": ("empty.wav", b"", "audio/wav")},
        data={"text": "Hello"},
    )
    assert resp.status_code == 400


@pytest.mark.asyncio
async def test_voice_design(client: AsyncClient) -> None:
    resp = await client.post(
        "/voice-design",
        json={"text": "Hello world", "instruct": "A warm female voice"},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "processing"
    assert data["message"] == "Voice design started"


@pytest.mark.asyncio
async def test_voice_design_empty_instruct_rejected(client: AsyncClient) -> None:
    resp = await client.post(
        "/voice-design",
        json={"text": "Hello", "instruct": ""},
    )
    assert resp.status_code == 422


@pytest.mark.asyncio
async def test_custom_voice(client: AsyncClient) -> None:
    resp = await client.post(
        "/custom-voice",
        json={"text": "Hello world", "speaker": "Chelsie"},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "processing"
    assert data["message"] == "Custom voice generation started"


@pytest.mark.asyncio
async def test_custom_voice_with_instruct(client: AsyncClient) -> None:
    resp = await client.post(
        "/custom-voice",
        json={
            "text": "Hello",
            "speaker": "Ethan",
            "instruct": "Speak slowly",
        },
    )
    assert resp.status_code == 200


@pytest.mark.asyncio
async def test_custom_voice_empty_speaker_rejected(client: AsyncClient) -> None:
    resp = await client.post(
        "/custom-voice",
        json={"text": "Hello", "speaker": ""},
    )
    assert resp.status_code == 422


@pytest.mark.asyncio
async def test_clone_multi_speaker(client: AsyncClient) -> None:
    ref_id1 = await _upload_ref(client)
    ref_id2 = await _upload_ref(client)
    resp = await client.post(
        "/clone-multi-speaker",
        json={
            "segments": [
                {"text": "Line one", "ref_audio_id": ref_id1},
                {"text": "Line two", "ref_audio_id": ref_id2},
            ]
        },
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "processing"
    assert data["message"] == "Multi-speaker cloning started"


@pytest.mark.asyncio
async def test_clone_multi_speaker_empty_segments_rejected(
    client: AsyncClient,
) -> None:
    resp = await client.post(
        "/clone-multi-speaker",
        json={"segments": []},
    )
    assert resp.status_code == 422


@pytest.mark.asyncio
async def test_clone_multi_speaker_ref_not_found(client: AsyncClient) -> None:
    resp = await client.post(
        "/clone-multi-speaker",
        json={
            "segments": [
                {"text": "Hello", "ref_audio_id": "nonexistent"},
            ]
        },
    )
    assert resp.status_code == 404
