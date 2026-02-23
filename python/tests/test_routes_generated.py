"""Generated audio route tests."""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from httpx import AsyncClient

FAKE_WAV = b"RIFF" + b"\x00" * 40


async def _generate_audio(client: AsyncClient) -> str:
    """Helper: upload ref, clone, wait, return task_id."""
    upload = await client.post(
        "/upload-reference",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
    )
    ref_id = upload.json()["id"]
    resp = await client.post(
        "/clone",
        json={"text": "Hello world", "ref_audio_id": ref_id},
    )
    task_id = resp.json()["task_id"]
    await asyncio.sleep(0.1)
    return task_id


@pytest.mark.asyncio
async def test_list_generated_empty(client: AsyncClient) -> None:
    resp = await client.get("/generated")
    assert resp.status_code == 200
    assert resp.json() == []


@pytest.mark.asyncio
async def test_list_generated_after_clone(client: AsyncClient) -> None:
    await _generate_audio(client)
    resp = await client.get("/generated")
    assert resp.status_code == 200
    items = resp.json()
    assert len(items) == 1
    assert items[0]["generated_text"] == "Hello world"


@pytest.mark.asyncio
async def test_delete_generated(client: AsyncClient) -> None:
    task_id = await _generate_audio(client)
    resp = await client.delete(f"/generated/{task_id}")
    assert resp.status_code == 200
    assert "deleted" in resp.json()["message"].lower()

    # Verify gone
    resp = await client.get("/generated")
    assert resp.json() == []


@pytest.mark.asyncio
async def test_delete_generated_not_found(client: AsyncClient) -> None:
    resp = await client.delete("/generated/nonexistent")
    assert resp.status_code == 404
