# TODO

## Verification

- [ ] Multi-speaker clone — not yet tested end-to-end
- [ ] Voice design — not yet tested end-to-end
- [ ] Custom voice — not yet tested end-to-end

## CI

- [ ] Add Python checks to GitHub Actions (pytest, pyright, ruff)
- [ ] Pin uv version in CI
- [ ] Consider skipping GPU-dependent tests in CI (mock TTS engine)

## Platform Support

- [ ] Test on non-NixOS Linux (Ubuntu, Fedora, Arch)
- [ ] Test on macOS (MPS backend for torch)
- [ ] Test on Windows (CUDA path differences)
- [ ] Document manual setup without Nix

## GUI Polish

Current UI is minimal iced widgets. The goal is a focused, efficient
workflow for voice synthesis — not a DAW clone. Every element should
serve a clear purpose and reduce the number of clicks to get from
"I have text" to "I have audio I like."

### Design Principles

- **Audio-first**: waveforms and playback are primary, not secondary
- **Non-destructive**: keep every generated take; let users compare and pick
- **Progressive disclosure**: simple by default, advanced options on demand
- **Keyboard-friendly**: power users should never need the mouse

### Layout & Navigation

- [ ] Sidebar navigation with icon + label (replace horizontal tab bar)
- [ ] Collapsible sidebar for more workspace when generating
- [ ] Persistent bottom player bar (always accessible, not per-tab)
- [ ] Consistent spacing, padding, and visual hierarchy across all views

### Audio & Waveform

- [ ] Waveform display for reference audio (interactive seek on click)
- [ ] Waveform display for generated audio with playhead
- [ ] Mini waveform thumbnails in history list
- [ ] Visual recording indicator with live level meter
- [ ] Real-time generation progress (not just elapsed time)

### Voice Management

- [ ] Voice profile cards with name, sample count, and inline preview
- [ ] Drag-and-drop audio file upload
- [ ] Inline rename (click-to-edit)
- [ ] Bulk select and delete
- [ ] Import/export voice profiles

### Generation Workflow

- [ ] Generation history panel (persisted across sessions)
- [ ] Quick replay from history with one click
- [ ] Side-by-side A/B comparison of generated outputs
- [ ] Seed input for reproducible generation
- [ ] "Regenerate" button reusing same parameters
- [ ] Queue multiple generation requests

### Multi-speaker

- [ ] Timeline view for arranging speaker segments
- [ ] Per-segment waveform preview
- [ ] Drag-and-drop segment reordering
- [ ] Per-segment speaker and language override
- [ ] Split / merge segments

### Polish

- [ ] Keyboard shortcuts (Space: play/pause, Ctrl+Enter: generate, Ctrl+1-5: switch tabs)
- [ ] Dark / light theme toggle (wiring exists, needs visual pass)
- [ ] Toast notifications for errors and completion (replace inline text)
- [ ] Skeleton loading states
- [ ] Window title shows current activity ("Generating..." / "Ready")

## Server

- [ ] Port retry (try next port if default is occupied)
- [ ] Server status indicator in GUI (connected / disconnected / error)
- [ ] Model download progress in GUI
- [ ] Support 0.6B model size selection from GUI

## Performance

- [ ] `flash-attn` integration (optional dependency, already wired)
- [ ] Voice clone prompt caching (`create_voice_clone_prompt()`)
- [ ] Streaming generation if upstream supports it
