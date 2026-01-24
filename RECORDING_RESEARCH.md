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

## GIF Recording Options & Performance Research

Achieving high-quality, high-framerate GIF recording is computationally expensive due to the need for color quantization (reducing 16 million colors to a 256-color palette) for *every single frame*. We explored several architectural approaches to solve the "stalling/slow encoding" problem.

### 1. Pure Rust: Serial Encoding (The "Naive" Approach)
*   **Method:** Capture frames to memory/disk, then iterate sequentially using the `image` crate's `GifEncoder`.
*   **Performance:** ~200-500ms per frame. A 10-second recording (250 frames) takes >50 seconds to encode.
*   **Bottleneck:** The `image` crate performs NeuQuant color quantization on the main thread for each frame. This is strictly single-threaded and CPU-bound.
*   **Verdict:** Too slow for a modern user experience.

### 2. Pure Rust: Parallel Quantization (The "Robust" Approach)
*   **Method:** Decouple quantization from encoding.
    1.  Capture frames to a buffer.
    2.  Use `rayon` to spawn a thread pool.
    3.  In parallel, map each RGBA frame to a 256-color indexed frame (palette + indices).
    4.  Feed the pre-quantized frames to the `GifEncoder` (which just writes bytes).
*   **Pros:** Linear speedup with CPU cores (e.g., 8-core CPU = ~8x faster). Keeps the application self-contained (no external CLI tools).
*   **Cons:** High implementation complexity. Requires managing frame ordering and significant memory bandwidth. Still slower than highly optimized C libraries.
*   **Verdict:** The best "pure Rust" fallback if we want to avoid external dependencies in v1.0.

### 3. Pure Rust: Global Palette Optimization (The "Speed" Hack)
*   **Method:** Generate a palette *once* from the first frame (or a sample of frames) and reuse it for the entire video.
*   **Performance:** Near-instant encoding (mapping pixels to an existing palette is fast).
*   **Cons:** Severe visual artifacts if the screen content changes drastically (e.g., switching from a dark window to a white web page).
*   **Verdict:** Good for UI demos, bad for general screen recording.

### 4. Native GStreamer Plugins (`gifenc`)
*   **Method:** Use the standard `gifenc` element in the GStreamer pipeline.
*   **Pros:** "Architecturally pure" GStreamer usage.
*   **Cons:**
    *   **Availability:** The plugin is often missing or blacklisted on user systems (e.g., Arch/Ubuntu default installs), leading to crashes or "element not found" errors.
    *   **Quality:** The standard encoder produces large, grainy files with poor dithering.
*   **Verdict:** Too unreliable for a general-purpose tool.

### 5. FFmpeg Pipe (The "Industry Standard")
*   **Method:** Spawn an `ffmpeg` subprocess reading from `stdin`. Stream raw RGBA bytes from GStreamer directly to FFmpeg.
*   **Command:** `ffmpeg -f rawvideo -pix_fmt rgba -s WxH -r 25 -i pipe:0 ... output.gif`
*   **Pros:**
    *   **Speed:** Leveraging FFmpeg's highly optimized C/Assembly encoders. Real-time encoding is possible.
    *   **Quality:** Access to the `palettegen` filter, which generates a statistically optimal palette for the video content.
    *   **Efficiency:** Zero intermediate disk IO if piped directly.
*   **Cons:** Adds a runtime dependency on the `ffmpeg` CLI tool.
*   **Verdict:** **CHOSEN SOLUTION.** It offers the best balance of performance, maintainability, and output quality.

**Final Decision:** We will implement Option 5 (FFmpeg Pipe) as the primary method. If `ffmpeg` is missing, we disable GIF recording rather than shipping a broken/slow experience.

## Implementation Journey (GIF)

- **Attempt 1: Serial Rust.** Used `image` crate. Extremely slow (>1min for 10s clip). Bottleneck: NeuQuant on CPU.
- **Attempt 2: GStreamer gifenc.** Plugin missing on many distros. Poor quality.
- **Attempt 3: Static linking gst-plugin-gif.** Linking issues (cdylib vs rlib).
- **Final: FFmpeg Pipe.** GStreamer captures -> `appsink` -> Pipe raw RGBA to `ffmpeg` stdin. 
  - **Why it works:** FFmpeg is highly optimized. `palettegen` filter gives superior colors.
  - **Clipboard fix:** Raw GIF bytes paste as static images in many apps. Copying the File URI (`file://`) triggers the "File Upload" flow in chat apps, preserving animation.
  - **Signal handling:** Added support for exit codes 255/130 to handle Ctrl+C without reporting failure.
