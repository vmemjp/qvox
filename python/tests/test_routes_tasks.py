"""Task management route tests."""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from httpx import AsyncClient

FAKE_WAV = b"RIFF" + b"\x00" * 40


async def _start_clone(client: AsyncClient) -> str:
    """Helper: upload ref + start clone, return task_id."""
    upload = await client.post(
        "/upload-reference",
        files={"file": ("test.wav", FAKE_WAV, "audio/wav")},
    )
    ref_id = upload.json()["id"]
    resp = await client.post(
        "/clone",
        json={"text": "Hello world", "ref_audio_id": ref_id},
    )
    return resp.json()["task_id"]


@pytest.mark.asyncio
async def test_task_status_processing(client: AsyncClient) -> None:
    task_id = await _start_clone(client)
    resp = await client.get(f"/tasks/{task_id}")
    assert resp.status_code == 200
    data = resp.json()
    # Status should be processing or completed (fake engine is fast)
    assert data["status"] in ("processing", "completed")


@pytest.mark.asyncio
async def test_task_status_completed(client: AsyncClient) -> None:
    task_id = await _start_clone(client)
    # Give the async task time to complete
    await asyncio.sleep(0.1)
    resp = await client.get(f"/tasks/{task_id}")
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "completed"
    assert data["progress"] == 100


@pytest.mark.asyncio
async def test_task_not_found(client: AsyncClient) -> None:
    resp = await client.get("/tasks/nonexistent")
    assert resp.status_code == 404


@pytest.mark.asyncio
async def test_cancel_task(client: AsyncClient) -> None:
    task_id = await _start_clone(client)
    resp = await client.post(f"/tasks/{task_id}/cancel")
    assert resp.status_code == 200
    assert "cancelled" in resp.json()["message"].lower()


@pytest.mark.asyncio
async def test_cancel_nonexistent_task(client: AsyncClient) -> None:
    resp = await client.post("/tasks/nonexistent/cancel")
    assert resp.status_code == 404


@pytest.mark.asyncio
async def test_task_audio(client: AsyncClient) -> None:
    task_id = await _start_clone(client)
    await asyncio.sleep(0.1)

    resp = await client.get(f"/tasks/{task_id}/audio")
    assert resp.status_code == 200
    assert resp.headers["content-type"] == "audio/wav"
    assert len(resp.content) > 0


@pytest.mark.asyncio
async def test_task_audio_not_completed(client: AsyncClient) -> None:
    """Fetching audio before task completes should fail or return 400."""
    task_id = await _start_clone(client)
    # Cancel immediately to prevent completion
    await client.post(f"/tasks/{task_id}/cancel")
    resp = await client.get(f"/tasks/{task_id}/audio")
    assert resp.status_code == 400


@pytest.mark.asyncio
async def test_task_audio_not_found(client: AsyncClient) -> None:
    resp = await client.get("/tasks/nonexistent/audio")
    assert resp.status_code == 404
