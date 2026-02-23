use std::io::Cursor;

use anyhow::{Context, Result};
use rodio::{Decoder, MixerDeviceSink, Player};

/// Playback state exposed to the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Wraps rodio's `Player` + `MixerDeviceSink` for controlled audio playback.
///
/// The `MixerDeviceSink` (stream handle) must stay alive for the duration of
/// playback — dropping it silences all audio immediately.
pub struct AudioPlayer {
    _stream: MixerDeviceSink,
    player: Player,
    state: PlaybackState,
}

impl std::fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl AudioPlayer {
    /// Create a new player using the default audio output device.
    pub fn new() -> Result<Self> {
        let stream =
            rodio::DeviceSinkBuilder::open_default_sink().context("failed to open audio device")?;
        let player = Player::connect_new(stream.mixer());
        Ok(Self {
            _stream: stream,
            player,
            state: PlaybackState::Stopped,
        })
    }

    /// Current playback state.
    pub fn state(&self) -> PlaybackState {
        // Sync internal state: if the queue drained, we're stopped.
        if self.state == PlaybackState::Playing && self.player.empty() {
            return PlaybackState::Stopped;
        }
        self.state
    }

    /// Play WAV audio from raw bytes.  Stops any current playback first.
    pub fn play_bytes(&mut self, wav_data: Vec<u8>) -> Result<()> {
        self.player.stop();
        let cursor = Cursor::new(wav_data);
        let source =
            Decoder::try_from(cursor).context("failed to decode audio data")?;
        self.player.append(source);
        self.state = PlaybackState::Playing;
        Ok(())
    }

    /// Pause the current playback.
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.player.pause();
            self.state = PlaybackState::Paused;
        }
    }

    /// Resume paused playback.
    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            self.player.play();
            self.state = PlaybackState::Playing;
        }
    }

    /// Stop playback and clear the queue.
    pub fn stop(&mut self) {
        self.player.stop();
        self.state = PlaybackState::Stopped;
    }

}
