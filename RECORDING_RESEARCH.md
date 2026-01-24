# Screen Recording Research & Plan

## Goal
Implement screen recording (screen, area, window) for both Wayland and X11, supporting audio and video output (MP4/WebM/GIF).

## Architecture Analysis

### Wayland (The Hard Part)
- **Mechanism:** Cannot simply "grab" pixels repeatedly (too slow/secure). Must use **PipeWire**.
- **Discovery:** `xdg-desktop-portal` (via `ashpd`) requests a "ScreenCast" session.
- **Handshake:**
  1. App requests Session via Portal.
  2. User selects Screen/Window in system dialog.
  3. Portal returns a **PipeWire Node ID**.
- **Consumption:** We need a PipeWire client to consume this Node ID.

### X11 (The Easy Part)
- **Mechanism:** `ximagesrc` (GStreamer) or `x11grab` (FFmpeg) can directly capture the X server display.
- **No Portal needed:** (Usually) though we can use Portals on X11 too for consistency if desired, but native is faster/simpler to start.

## Technology Choice: GStreamer vs FFmpeg

| Feature | GStreamer (Rust Bindings) | FFmpeg (CLI Wrapper) | FFmpeg (Rust Bindings) |
|---------|---------------------------|----------------------|------------------------|
| **PipeWire** | First-class (`pipewiresrc`) | Doable but messy | Complex setup |
| **Performance** | High (Zero-copy pipelines) | Med (IPC overhead) | High |
| **Reliability** | High (Stable Rust crates) | High | Med (Bindings get out of sync) |
| **Integration** | "Native" Rust feel | Process spawning | C-FFI heavy |

**Verdict:** **GStreamer** is the correct choice for a "Chad" Linux tool. It handles the PipeWire handshake natively and avoids the overhead of spawning child processes for high-bandwidth video data.

## Implementation Plan

### Phase 1: Dependencies & Core
- Add `gstreamer`, `gstreamer-app`, `gstreamer-video`.
- Ensure `ashpd` has `pipewire` features enabled (if applicable, or just generic).

### Phase 2: Recording Manager
- Create `src/recording/mod.rs`.
- Define `RecordingConfig` (path, format, audio toggle).
- Trait `Recorder` with `start()`, `stop()`, `pause()`.

### Phase 3: Wayland Implementation
- `ashpd::desktop::screencast` to get Node ID.
- Build GStreamer pipeline:
  ```
  pipewiresrc path=<node-id> ! videoconvert ! x264enc ! mp4mux ! filesink location=out.mp4
  ```

### Phase 4: X11 Implementation
- Build GStreamer pipeline:
  ```
  ximagesrc ! videoconvert ! x264enc ! mp4mux ! filesink location=out.mp4
  ```

### Phase 5: CLI Integration
- `cargo run -- record screen`
- `cargo run -- record area` (Requires overlay to get coordinates for X11 crop, or portal for Wayland)

## Audio Recording (Attempted v0.2.0)

### Implementation Attempt
Attempted to add audio capture using `pulsesrc` muxed into the video pipeline.
Pipeline structure:
```
{muxer} name=mux ! filesink location={path}
{video_source} ! videoconvert ! videorate ! queue ! {video_encoder} ! mux.
pulsesrc ! audioconvert ! audioresample ! queue ! {audio_encoder} ! mux.
```

### Issues Encountered
1. **Unhandled Format Error:** `pipewiresrc` reported `stream error: unhandled format` immediately after adding the audio branch. This likely indicates a negotiation conflict between the two branches when muxing, or a PipeWire session contention.
2. **Encoder Availability:** Common AAC encoders like `voaacenc` and `avenc_aac` were missing on the test system. `faac` was available but the negotiation issues persisted.
3. **Resource Contention:** Recording audio triggered "corking" (pausing) of other media players on the system.

### Potential Solutions
- **Caps Negotiation:** Explicitly define caps between `pipewiresrc` and `videoconvert` (e.g., `video/x-raw(memory:DmaBuf)` or `video/x-raw`).
- **Separate Recording:** Record audio to a temp file and mux later (avoids real-time muxing complexity but adds latency/post-processing).
- **PipeWire Audio:** Use `pipewiresrc` for audio as well, potentially leveraging the same session if supported.
- **Dynamic Probing:** Implement better probing for available audio encoders.

*Status: Deferred to future version.*