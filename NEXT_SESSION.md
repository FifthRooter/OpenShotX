# Next Session

*Previous task (Region Recording) verified on Wayland.*

## Current State

**Last commit:** (uncommitted changes for Region support verified)
**Branch:** feat/screen-recording

## Completed

- **Screen Recording (Video)**:
  - GStreamer pipeline with PipeWire (Wayland) and XImage (X11) support.
  - Automatic encoder selection (x264 > VP8 > Theora).
  - Robust handling of missing codecs (helpful error messages).
  - Validated MP4 recording on Wayland (Monitor selection).
  - **Fixed Wayland Region Recording**: Implemented manual DBus handling to bypass `ashpd` crash on "Region" selection.
- **OCR Module**: Full text extraction.

## Next Session: Audio & Config

### Priority 1: Audio Capture
- [ ] Add `pulsesrc` (PulseAudio/PipeWire audio) to the pipeline.
- [ ] Mux audio stream into MP4/WebM container.
- [ ] Add CLI flag `--audio` or `--mic`.

### Priority 2: Configuration
- [ ] Implement YAML config to save preferences (e.g., default encoder, path).

### Priority 3: Merge & Release
- [ ] Merge `feat/screen-recording` to `main`.
- [ ] Tag v0.2.0-alpha.

### Priority 2: Configuration
- [ ] Implement YAML config to save preferences (e.g., default encoder, path).

### Priority 3: Merge & Release
- [ ] Merge `feat/screen-recording` to `main`.
- [ ] Tag v0.2.0-alpha.

## Notes

**Build Requirements:**
Ensure GStreamer plugins are installed (see README).