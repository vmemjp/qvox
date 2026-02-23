"""TaskManager — in-memory async task management."""

from __future__ import annotations

import asyncio
import logging
import time
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Coroutine

logger = logging.getLogger(__name__)


@dataclass
class TaskState:
    """Mutable state for a single generation task."""

    task_id: str
    status: str = "processing"
    progress: int = 0
    output_path: str | None = None
    ref_audio_id: str | None = None
    generation_time_seconds: float | None = None
    error: str | None = None
    is_multi_speaker: bool | None = None
    total_segments: int | None = None
    current_segment: int | None = None
    start_time: float = field(default_factory=time.monotonic)
    async_task: asyncio.Task[None] | None = field(default=None, repr=False)


class TaskManager:
    """Manages async TTS generation tasks.

    Two-phase usage:
      1. ``register()`` — create and store a TaskState
      2. ``start()`` — attach a coroutine and launch it

    This lets the coroutine receive a reference to the real TaskState
    so it can update progress and status in place.
    """

    def __init__(self) -> None:
        self._tasks: dict[str, TaskState] = {}

    def get(self, task_id: str) -> TaskState | None:
        """Get task state by ID."""
        return self._tasks.get(task_id)

    def register(
        self,
        task_id: str,
        *,
        ref_audio_id: str | None = None,
        is_multi_speaker: bool = False,
        total_segments: int | None = None,
    ) -> TaskState:
        """Create and store a TaskState (not yet running)."""
        state = TaskState(
            task_id=task_id,
            ref_audio_id=ref_audio_id,
            is_multi_speaker=is_multi_speaker or None,
            total_segments=total_segments,
        )
        self._tasks[task_id] = state
        return state

    def start(self, state: TaskState, coro: Coroutine[Any, Any, None]) -> None:
        """Attach a coroutine to an already-registered TaskState and launch it."""
        if self._tasks.get(state.task_id) is not state:
            msg = f"Task {state.task_id} not registered"
            raise KeyError(msg)

        async def _run() -> None:
            try:
                await coro
            except asyncio.CancelledError:
                state.status = "cancelled"
                raise
            except Exception as exc:
                logger.exception("Task %s failed", state.task_id)
                state.status = "failed"
                state.error = str(exc)

        state.async_task = asyncio.create_task(_run())

    def cancel(self, task_id: str) -> bool:
        """Cancel a running task. Returns True if the task was cancelled."""
        state = self._tasks.get(task_id)
        if state is None:
            return False
        if state.async_task is not None and not state.async_task.done():
            state.async_task.cancel()
            state.status = "cancelled"
            return True
        return False

    def cancel_all(self) -> None:
        """Cancel all running tasks."""
        for task_id in list(self._tasks):
            self.cancel(task_id)
