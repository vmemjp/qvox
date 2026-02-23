"""Health, capabilities, and language endpoints."""

from __future__ import annotations

from fastapi import APIRouter, Request

from server.app import get_engine
from server.schemas import (
    SUPPORTED_LANGUAGES,
    CapabilitiesResponse,
    HealthResponse,
    LanguagesResponse,
)

router = APIRouter()


@router.get("/health")
def health(request: Request) -> HealthResponse:
    """Check server health and loaded models."""
    engine = get_engine(request)
    return HealthResponse(
        status="healthy",
        voice_cloner_loaded=engine.is_ready,
        loaded_models=engine.loaded_models,
    )


@router.get("/capabilities")
def capabilities(request: Request) -> CapabilitiesResponse:
    """List available models and speakers."""
    engine = get_engine(request)
    return CapabilitiesResponse(
        models=engine.loaded_models,
        speakers=engine.speakers,
    )


@router.get("/languages")
def languages() -> LanguagesResponse:
    """List supported languages."""
    return LanguagesResponse(languages=SUPPORTED_LANGUAGES)
