# Next Session

*Previous task (Screen Recording logic) implemented.*

## Current State

**Last commit:** (uncommitted Recording changes)
**Branch:** feat/screen-recording

## Completed

- **OCR Module**: Full text extraction support ✓
- **Recording Module**:
  - Scaffolding: `src/recording/mod.rs`
  - Engine: **GStreamer** (replaced FFmpeg plan)
  - Wayland: `ashpd` Portal -> PipeWire Node ID -> `pipewiresrc`
  - X11: `ximagesrc` with area support
  - CLI: `cargo run -- record screen|area`
  - Async Runtime: `tokio` integrated into `main.rs`

## Build Requirements (NEW)

**System Dependencies (GStreamer):**
To build the new recording module, you must install GStreamer development libraries:

```bash
# Ubuntu/Debian
sudo apt install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    libgstreamer-plugins-bad1.0-dev

# Arch
sudo pacman -S gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly

# Fedora
sudo dnf install gstreamer1-devel gstreamer1-plugins-base-devel
```

## Next Session: Verification & Polish

### Priority 1: Verify Recording
- [ ] Build project (ensure GStreamer links)
- [ ] Test X11 recording (`record area`)
- [ ] Test Wayland recording (`record screen`)
- [ ] Verify `Ctrl+C` cleanly stops and finalizes MP4

### Priority 2: Audio Support (Deferred)
- [ ] Add `pulsesrc` to pipeline
- [ ] Sync audio/video streams

### Priority 3: Config System
- [ ] YAML config implementation

## Notes

**Key Decisions:**
1. **Engine Swap:** Switched to GStreamer for robust PipeWire support.
2. **Wayland Area:** Uses Portal Window/Screen selection (system dialog) instead of custom drag overlay.
3. **Video Only:** Audio deferred to ensure video stability first.