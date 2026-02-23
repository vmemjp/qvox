"""TaskManager unit tests."""

from __future__ import annotations

import asyncio

import pytest

from server.tasks import TaskManager, TaskState


class TestTaskState:
    def test_default_values(self) -> None:
        state = TaskState(task_id="test-1")
        assert state.status == "processing"
        assert state.progress == 0
        assert state.output_path is None
        assert state.error is None
        assert state.is_multi_speaker is None
        assert state.async_task is None

    def test_multi_speaker_fields(self) -> None:
        state = TaskState(
            task_id="test-2",
            is_multi_speaker=True,
            total_segments=3,
        )
        assert state.is_multi_speaker is True
        assert state.total_segments == 3
        assert state.current_segment is None


class TestTaskManager:
    def test_get_nonexistent_returns_none(self) -> None:
        tm = TaskManager()
        assert tm.get("nonexistent") is None

    def test_register_creates_state(self) -> None:
        tm = TaskManager()
        state = tm.register("task-1", ref_audio_id="ref-1")
        assert state.task_id == "task-1"
        assert state.ref_audio_id == "ref-1"
        assert state.status == "processing"

    def test_get_after_register(self) -> None:
        tm = TaskManager()
        state = tm.register("task-1")
        fetched = tm.get("task-1")
        assert fetched is state

    @pytest.mark.asyncio
    async def test_start_runs_coroutine(self) -> None:
        tm = TaskManager()
        state = tm.register("task-1")

        async def work() -> None:
            state.status = "completed"
            state.progress = 100

        tm.start(state, work())
        assert state.async_task is not None
        await state.async_task
        assert state.status == "completed"
        assert state.progress == 100

    @pytest.mark.asyncio
    async def test_failed_task(self) -> None:
        tm = TaskManager()
        state = tm.register("task-1")

        async def fail() -> None:
            msg = "something broke"
            raise RuntimeError(msg)

        tm.start(state, fail())
        assert state.async_task is not None
        await state.async_task
        assert state.status == "failed"
        assert state.error == "something broke"

    @pytest.mark.asyncio
    async def test_cancel_running_task(self) -> None:
        tm = TaskManager()
        state = tm.register("task-1")

        async def slow() -> None:
            await asyncio.sleep(100)

        tm.start(state, slow())
        assert tm.cancel("task-1") is True
        assert state.status == "cancelled"

    def test_cancel_nonexistent_returns_false(self) -> None:
        tm = TaskManager()
        assert tm.cancel("nope") is False

    @pytest.mark.asyncio
    async def test_cancel_completed_returns_false(self) -> None:
        tm = TaskManager()
        state = tm.register("task-1")

        async def instant() -> None:
            state.status = "completed"

        tm.start(state, instant())
        assert state.async_task is not None
        await state.async_task
        assert tm.cancel("task-1") is False

    @pytest.mark.asyncio
    async def test_cancel_all(self) -> None:
        tm = TaskManager()
        states: list[TaskState] = []
        for i in range(3):
            s = tm.register(f"task-{i}")
            states.append(s)

            async def slow() -> None:
                await asyncio.sleep(100)

            tm.start(s, slow())

        tm.cancel_all()
        for s in states:
            assert s.status == "cancelled"

    @pytest.mark.asyncio
    async def test_register_multi_speaker(self) -> None:
        tm = TaskManager()
        state = tm.register(
            "task-ms", is_multi_speaker=True, total_segments=5
        )
        assert state.is_multi_speaker is True
        assert state.total_segments == 5

    @pytest.mark.asyncio
    async def test_start_unregistered_raises(self) -> None:
        tm = TaskManager()
        state = TaskState(task_id="unregistered")

        async def noop() -> None:
            pass

        with pytest.raises(KeyError):
            tm.start(state, noop())
