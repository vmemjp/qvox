"""Reference audio route tests."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from httpx import AsyncClient

FAKE_WAV = b"RIFF" + b"\x00" * 40


@pytest.mark.asyncio
async def test_list_references_empty(client: AsyncClient) -> None:
    resp = await client.get("/references")
    assert resp.status_code == 200
    assert resp.json() == []


@pytest.mark.asyncio
async def test_upload_reference(client: AsyncClient) -> None:
    resp = await client.post(
        "/upload-reference",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["original_name"] == "test.wav"
    assert "id" in data
    assert "created_at" in data


@pytest.mark.asyncio
async def test_upload_reference_with_ref_text(client: AsyncClient) -> None:
    resp = await client.post(
        "/upload-reference",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
        data={"ref_text": "hello world"},
    )
    assert resp.status_code == 200
    data = resp.json()
    assert data["ref_text"] == "hello world"


@pytest.mark.asyncio
async def test_upload_empty_file_rejected(client: AsyncClient) -> None:
    resp = await client.post(
        "/upload-reference",
        files={"file": ("empty.wav", b"", "audio/wav")},
    )
    assert resp.status_code == 400


@pytest.mark.asyncio
async def test_list_references_after_upload(client: AsyncClient) -> None:
    await client.post(
        "/upload-reference",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
    )
    resp = await client.get("/references")
    assert resp.status_code == 200
    refs = resp.json()
    assert len(refs) == 1


@pytest.mark.asyncio
async def test_get_reference_audio(client: AsyncClient) -> None:
    upload = await client.post(
        "/upload-reference",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
    )
    audio_id = upload.json()["id"]

    resp = await client.get(f"/references/{audio_id}/audio")
    assert resp.status_code == 200
    assert resp.headers["content-type"] == "audio/wav"
    assert resp.content == FAKE_WAV


@pytest.mark.asyncio
async def test_get_reference_audio_not_found(client: AsyncClient) -> None:
    resp = await client.get("/references/nonexistent/audio")
    assert resp.status_code == 404


@pytest.mark.asyncio
async def test_delete_reference(client: AsyncClient) -> None:
    upload = await client.post(
        "/upload-reference",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
    )
    audio_id = upload.json()["id"]

    resp = await client.delete(f"/references/{audio_id}")
    assert resp.status_code == 200
    assert "deleted" in resp.json()["message"].lower()

    # Verify gone
    resp = await client.get("/references")
    assert resp.json() == []


@pytest.mark.asyncio
async def test_delete_reference_not_found(client: AsyncClient) -> None:
    resp = await client.delete("/references/nonexistent")
    assert resp.status_code == 404


@pytest.mark.asyncio
async def test_rename_reference(client: AsyncClient) -> None:
    upload = await client.post(
        "/upload-reference",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
    )
    audio_id = upload.json()["id"]

    resp = await client.put(
        f"/references/{audio_id}/name",
        json={"name": "My Voice"},
    )
    assert resp.status_code == 200
    assert resp.json()["name"] == "My Voice"


@pytest.mark.asyncio
async def test_rename_reference_not_found(client: AsyncClient) -> None:
    resp = await client.put(
        "/references/nonexistent/name",
        json={"name": "My Voice"},
    )
    assert resp.status_code == 404


@pytest.mark.asyncio
async def test_rename_empty_name_rejected(client: AsyncClient) -> None:
    upload = await client.post(
        "/upload-reference",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
    )
    audio_id = upload.json()["id"]

    resp = await client.put(
        f"/references/{audio_id}/name",
        json={"name": ""},
    )
    assert resp.status_code == 422  # Validation error
