"""Entry point for the Qwen3-TTS FastAPI server."""

from __future__ import annotations

import argparse
import os
import signal
import sys
import threading


def _start_parent_watchdog(parent_pid: int) -> None:
    """Watch the parent process and exit when it dies."""

    def _watch() -> None:
        import time

        while True:
            time.sleep(2)
            try:
                os.kill(parent_pid, 0)
            except OSError:
                os.kill(os.getpid(), signal.SIGTERM)
                return

    t = threading.Thread(target=_watch, daemon=True)
    t.start()


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(description="Qwen3-TTS FastAPI server")
    parser.add_argument("--port", type=int, default=8000, help="Port to listen on")
    parser.add_argument(
        "--models",
        nargs="+",
        default=["base"],
        choices=["base", "voice_design", "custom_voice"],
        help="Models to make available",
    )
    parser.add_argument(
        "--device",
        type=str,
        default="auto",
        help="Device to use (auto, cuda, cpu)",
    )
    parser.add_argument(
        "--model-size",
        type=str,
        default="1.7B",
        choices=["0.6B", "1.7B"],
        help="Model size variant",
    )
    parser.add_argument(
        "--parent-pid",
        type=int,
        default=None,
        help="Parent process PID to monitor (exit when parent dies)",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> None:
    """Start the server with uvicorn."""
    args = parse_args(argv)

    parent_pid: int | None = args.parent_pid  # pyright: ignore[reportAny]
    if parent_pid is None and sys.platform != "win32":
        parent_pid = os.getppid()
    if parent_pid is not None:  # pyright: ignore[reportUnnecessaryComparison]
        _start_parent_watchdog(parent_pid)

    os.environ["QVOX_MODELS"] = ",".join(args.models)
    os.environ["QVOX_DEVICE"] = args.device
    os.environ["QVOX_MODEL_SIZE"] = args.model_size

    import uvicorn

    uvicorn.run(
        "server.app:create_app",
        factory=True,
        host="0.0.0.0",
        port=args.port,
        log_level="info",
    )


if __name__ == "__main__":
    main()
