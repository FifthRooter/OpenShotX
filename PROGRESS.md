# Progress Tracker

## Completed

### Phase 7: Screen Recording (v0.2.0)
- [x] Add `gstreamer`, `gstreamer-app`, and `gstreamer-video` dependencies
- [x] Create `src/recording/mod.rs` with:
  - GStreamer pipeline abstraction
  - Wayland support via `ashpd` (PipeWire)
  - X11 support via `ximagesrc`
  - Dynamic encoder selection (H.264, VP8, VP9, Theora)
  - Automatic fallback for missing codecs
  - Clean Ctrl+C finalization (EOS handling)
- [x] **GIF Recording Support**:
  - Implemented high-performance streaming pipe to FFmpeg.
  - High-quality color quantization via `palettegen` filter.
  - Robust handling of Ctrl+C interruption and process signals.
- [x] **Clipboard Integration**:
  - Automatically copies recorded GIF as a File URI (`text/uri-list`).
  - Ensures compatibility with Discord, Slack, and browsers.
- [x] Update `src/main.rs` with `record` subcommand
- [x] Transition CLI to async runtime using `tokio`
- [x] Update documentation with system requirements

### Phase 8: Wayland Region Fix (Critical)
- [x] Implement manual DBus handling for `Start` request
- [x] Bypass `ashpd` enum deserialization to support `SourceType: 16`
- [x] Fix `invalid value: 16` crash on Region selection
- [x] Validate interactive Region recording on Wayland

**System Requirements for Recording:**
- Arch: `sudo pacman -S gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly`
- Ubuntu: `sudo apt install gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly`

### Phase 6: OCR Module
- [x] Add tesseract-rs and arboard dependencies
- [x] Create `src/ocr/mod.rs` module with:
  - `OcrConfig` struct (language, min_confidence, clipboard, datapath)
  - `extract_text()` function from CaptureData
  - `extract_text_from_path()` function from image files
  - `copy_to_clipboard()` function using arboard
  - RGBA to grayscale conversion for Tesseract
  - Comprehensive error handling
  - 8 unit tests
- [x] Update `src/lib.rs` to export ocr module
- [x] Add `ocr` subcommand to CLI with options:
  - `--lang <code>` for language selection
  - `--min-conf <n>` for confidence threshold
  - `--no-clipboard` to disable clipboard copy
- [x] Add `--ocr` flag to capture command for integrated workflow
- [x] Update usage documentation

**System Requirements for OCR:**
- `tesseract` (system package)
- `leptonica` (system package)
- Language data: `tesseract-data-eng` (or other languages)

**Install commands:**
- Arch: `sudo pacman -S tesseract leptonica tesseract-data-eng`
- Ubuntu/Debian: `sudo apt install tesseract-ocr libtesseract-dev`
- Fedora: `sudo dnf install tesseract leptonica`

### Phase 1: Wayland Backend (ashpd)
- [x] Add `ashpd` crate for xdg-desktop-portal integration
- [x] Implement `capture_screen()` with non-interactive mode
- [x] Implement `capture_area()` with interactive mode
- [x] Implement `capture_window()` with interactive mode
- [x] Add integration tests with portal handling
- [x] Fix integer overflow bug in test validation

### Phase 2: X11 Backend
- [x] Direct XGetImage capture via `x11rb`
- [x] Screen, area, and window capture
- [x] XFixes cursor capture with fallback
- [x] Pixel format detection (RGB/BGR, 24/32-bit)
- [x] Comprehensive unit and integration tests

### Phase 3: Image Saving Module
- [x] Add `image` crate with PNG/JPEG features
- [x] Add `chrono` crate for timestamps
- [x] Implement pixel format conversion (6 variants)
- [x] Implement cursor compositing with alpha blending
- [x] Implement `save_capture()` with configurable options
- [x] Implement `quick_save()` with defaults
- [x] Add 11 unit tests for conversion and saving
- [x] Update library exports

## In Progress

### Phase 4: CLI Frontend
- [x] Create `src/main.rs` executable
- [x] Implement command-line argument parsing
- [x] Wire up capture → save pipeline
- [x] Re-export backend implementations
- [x] Test on Wayland (screen capture working)

### Phase 5: GTK4 Overlay (X11 area selection)
- [x] Add gtk4 dependency (0.10)
- [x] Create `src/overlay.rs` module
- [x] Full-screen transparent window with crosshair cursor
- [x] Mouse drag selection with live dimension display
- [x] ESC to cancel handling
- [x] Integration with X11 backend area capture

## TODO (v0.1.0 blockers)

- [ ] Config system (YAML via `serde_yml`)
- [ ] Multi-monitor support
- [ ] CLI hotkey integration
- [ ] Audio Capture (v0.2.x - encountered negotiation issues)

## Notes

**Wayland limitations:** Area/window capture require user interaction through portal dialogs. Coordinate-based capture is intentionally not possible.

**Test status:** 33/33 tests passing (22 backend + 11 capture)