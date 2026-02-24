# qvox

> **Beta** — This project is under active development. Expect rough edges.

Rust native GUI client for [Qwen3-TTS](https://github.com/QwenLM/Qwen3-TTS) voice synthesis.

Built with [iced](https://github.com/iced-rs/iced), communicates with a Python FastAPI inference server over HTTP.

## Features

- Voice cloning (single and multi-speaker)
- Voice design from text descriptions
- Custom voice with built-in speakers
- Built-in audio recording and playback
- Local Whisper transcription for reference audio
- Server lifecycle management (auto-start, auto-terminate)

## Status

| Feature | Status |
|---------|--------|
| Voice clone | Verified |
| Clone with upload | Verified |
| Multi-speaker clone | Not yet verified |
| Voice design | Not yet verified |
| Custom voice | Not yet verified |
| GUI polish | In progress |

## Requirements

- [Nix](https://nixos.org/) with flakes enabled
- NVIDIA GPU with CUDA support (tested with RTX 4070)

## Setup

```sh
nix develop
cargo run
```

The GUI automatically spawns the Python TTS server on startup.
On first run, model weights are downloaded from HuggingFace (~3.5 GB).

## Notice

This project has only been tested on **NixOS**. It may or may not work on other Linux distributions, macOS, or Windows.

## License

MIT
