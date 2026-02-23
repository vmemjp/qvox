# qvox

Rust native GUI client for [Qwen3-TTS](https://github.com/QwenLM/Qwen3-TTS) voice synthesis.

Built with [iced](https://github.com/iced-rs/iced), communicates with a Python FastAPI inference server over HTTP.

## Features

- Voice cloning (single and multi-speaker)
- Voice design from text descriptions
- Built-in audio recording and playback
- Local Whisper transcription for reference audio
- Server lifecycle management

## Requirements

- [Nix](https://nixos.org/) with flakes enabled

## Setup

```sh
nix develop
cargo build
```

## License

MIT
