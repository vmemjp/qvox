"""FastAPI application with lifespan and DI setup."""

from __future__ import annotations

import logging
import os
from contextlib import asynccontextmanager
from pathlib import Path
from typing import TYPE_CHECKING, cast

import platformdirs
from fastapi import FastAPI, Request

from server.models import QwenTTSEngine
from server.storage import FileStorage
from server.tasks import TaskManager

if TYPE_CHECKING:
    from collections.abc import AsyncIterator

    from server.protocols import StorageBackend, TTSEngine

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncIterator[None]:
    """Application lifespan: initialize engine, storage, and task manager."""
    models_str = os.environ.get("QVOX_MODELS", "base")
    device = os.environ.get("QVOX_DEVICE", "auto")
    model_size = os.environ.get("QVOX_MODEL_SIZE", "1.7B")

    model_names = [m.strip() for m in models_str.split(",") if m.strip()]

    engine = QwenTTSEngine(
        model_names=model_names,
        device=device,
        model_size=model_size,
    )

    data_dir = Path(platformdirs.user_data_dir("qvox"))
    storage = FileStorage(data_dir=data_dir)
    task_manager = TaskManager()

    app.state.engine = engine  # type: ignore[attr-defined]
    app.state.storage = storage  # type: ignore[attr-defined]
    app.state.task_manager = task_manager  # type: ignore[attr-defined]

    logger.info(
        "Server started: models=%s, device=%s, model_size=%s, data_dir=%s",
        model_names,
        device,
        model_size,
        data_dir,
    )
    yield
    task_manager.cancel_all()
    logger.info("Server shutting down")


# ─── Dependency accessors ───────────────────────────────────────


def get_engine(request: Request) -> TTSEngine:
    """Extract TTSEngine from app state."""
    return cast("TTSEngine", request.app.state.engine)


def get_storage(request: Request) -> StorageBackend:
    """Extract StorageBackend from app state."""
    return cast("StorageBackend", request.app.state.storage)


def get_task_manager(request: Request) -> TaskManager:
    """Extract TaskManager from app state."""
    return cast("TaskManager", request.app.state.task_manager)


# ─── App factory ────────────────────────────────────────────────


def create_app() -> FastAPI:
    """Create and configure the FastAPI application."""
    app = FastAPI(title="qvox TTS Server", lifespan=lifespan)

    from server.routes.generated import router as generated_router
    from server.routes.generation import router as generation_router
    from server.routes.health import router as health_router
    from server.routes.references import router as references_router
    from server.routes.tasks import router as tasks_router

    app.include_router(health_router)
    app.include_router(references_router)
    app.include_router(generation_router)
    app.include_router(tasks_router)
    app.include_router(generated_router)

    return app
