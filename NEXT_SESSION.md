# Next Session

*Previous task (Screen Recording) successfully verified.*

## Current State

**Last commit:** feat(recording): implement screen recording with GStreamer
**Branch:** feat/screen-recording (ready to merge)

## Completed

- **Screen Recording (Video)**:
  - GStreamer pipeline with PipeWire (Wayland) and XImage (X11) support.
  - Automatic encoder selection (x264 > VP8 > Theora).
  - Robust handling of missing codecs (helpful error messages).
  - Validated MP4 recording on Wayland.
- **OCR Module**: Full text extraction.

## Next Session: Audio Support & Polish

### Priority 1: Audio Capture
- [ ] Add `pulsesrc` (PulseAudio/PipeWire audio) to the pipeline.
- [ ] Mux audio stream into MP4/WebM container.
- [ ] Add CLI flag `--audio` or `--mic`.

### Priority 2: Configuration
- [ ] Implement YAML config to save preferences (e.g., default encoder, path).

### Priority 3: Merge & Release
- [ ] Merge `feat/screen-recording` to `main`.
- [ ] Tag v0.2.0-alpha.

## Notes

**Build Requirements:**
Ensure GStreamer plugins are installed (see README).
