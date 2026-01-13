# Next Session

*Previous task (GTK4 overlay for area selection) completed.*

## Current State

**Last commit:** 96fa7ca
**Branch:** main

## Completed

- X11 backend: screen/area/window capture, cursor via XFixes
- Wayland backend: portal integration via ashpd
- Image saving: PNG/JPEG, cursor compositing, timestamps
- CLI frontend: `cargo run -- capture screen|area|window`
- GTK4 overlay: X11 area selection with drag preview

## Next Session: OCR (Priority)

Implement text extraction from screenshots using tesseract-rs.

**Task Brief:** See `OCR_TASK_PROMPT.md` for comprehensive implementation details.

**Quick Overview:**
- Add tesseract-rs dependency
- Create `src/ocr/mod.rs` module
- CLI: `cargo run -- ocr <image>` and `--ocr` flag for capture
- Extract text with language selection
- Copy to clipboard (X11/Wayland aware)

**Files to Read First:**
1. `OCR_TASK_PROMPT.md` - Full task specification
2. `src/lib.rs` - Module structure
3. `src/capture/mod.rs` - Image conversion pipeline
4. `src/main.rs` - CLI structure

## Major Features Remaining (from README)

### Priority 1: OCR **← NEXT**
- [ ] tesseract-rs integration
- [ ] Text extraction from captures
- [ ] Language selection
- [ ] Copy to clipboard

### Priority 2: Screen Recording
- [ ] FFmpeg integration for video capture
- [ ] Output formats: mp4, webm, gif
- [ ] GIF optimization
- [ ] Audio capture support

### Priority 3: Scrolling Capture
- [ ] Auto-scroll detection and triggering
- [ ] Frame stitching and alignment
- [ ] Browser/app-specific handling

### Priority 4: Screenshot Editor
- [ ] GTK4/cairo-based annotation canvas
- [ ] Tools: arrows, boxes, text, blur/pixelate
- [ ] Undo/redo stack

### Priority 5: Cloud Integration
- [ ] S3/custom server upload
- [ ] URL shortening
- [ ] Upload history

## Technical Notes

**OCR Architecture:**
- Reuse existing `CaptureData` → `RgbaImage` conversion
- Pipe to tesseract-rs for text extraction
- Clipboard: detect X11 (xclip) vs Wayland (wl-copy)
- Support multi-language: `eng+fra+deu`

**Dependencies to Add:**
```toml
tesseract-rs = "0.1"
# clipboard: copypasta or CLI wrappers
```
