"""Health, capabilities, and languages route tests."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from server.schemas import SUPPORTED_LANGUAGES

if TYPE_CHECKING:
    from httpx import AsyncClient


@pytest.mark.asyncio
async def test_health(client: AsyncClient) -> None:
    resp = await client.get("/health")
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "healthy"
    assert data["voice_cloner_loaded"] is True
    assert isinstance(data["loaded_models"], list)


@pytest.mark.asyncio
async def test_capabilities(client: AsyncClient) -> None:
    resp = await client.get("/capabilities")
    assert resp.status_code == 200
    data = resp.json()
    assert "models" in data
    assert "speakers" in data
    assert isinstance(data["models"], list)
    assert isinstance(data["speakers"], list)
    assert len(data["models"]) > 0
    assert len(data["speakers"]) > 0


@pytest.mark.asyncio
async def test_languages(client: AsyncClient) -> None:
    resp = await client.get("/languages")
    assert resp.status_code == 200
    data = resp.json()
    assert data["languages"] == SUPPORTED_LANGUAGES
    assert "auto" in data["languages"]
    assert "English" in data["languages"]
