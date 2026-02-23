"""Entry point for the Qwen3-TTS FastAPI server."""

from __future__ import annotations

import argparse
import os


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
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> None:
    """Start the server with uvicorn."""
    args = parse_args(argv)

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
