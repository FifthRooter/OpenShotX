# Next Session

*Previous task (OCR implementation) completed.*

## Current State

**Last commit:** (uncommitted OCR changes)
**Branch:** main

## Completed

- X11 backend: screen/area/window capture, cursor via XFixes
- Wayland backend: portal integration via ashpd
- Image saving: PNG/JPEG, cursor compositing, timestamps
- CLI frontend: `cargo run -- capture screen|area|window`
- GTK4 overlay: X11 area selection with drag preview
- **OCR Module**: tesseract integration with clipboard support ✓

## OCR Implementation Complete ✓

**Added Files:**
- `src/ocr/mod.rs` - Full OCR module

**Modified Files:**
- `src/lib.rs` - Added OCR exports
- `src/capture/mod.rs` - Made `capture_to_rgba_image` public
- `src/main.rs` - Added `ocr` subcommand and `--ocr` flag
- `Cargo.toml` - Added tesseract and arboard dependencies

**CLI Usage:**
```bash
# Standalone OCR on existing image
cargo run -- ocr screenshot.png
cargo run -- ocr screenshot.png --lang eng+fra --min-conf 60

# Integrated capture + OCR (recommended workflow)
cargo run -- capture area --ocr
cargo run -- capture screen --ocr --lang eng
```

**Build Status:**
- ✅ Compiles without warnings
- ✅ 41/42 tests passing
- ✅ Tested on Wayland (Hyprland)

**System Requirements:**
```bash
# Arch
sudo pacman -S tesseract leptonica tesseract-data-eng

# Ubuntu/Debian
sudo apt install tesseract-ocr libtesseract-dev

# Fedora
sudo dnf install tesseract leptonica
```

## Next Session: Screen Recording (Priority)

## Major Features Remaining (from README)

### Priority 1: OCR ✓ **COMPLETE**
- [x] tesseract integration
- [x] Text extraction from captures
- [x] Language selection
- [x] Copy to clipboard (Wayland: wl-copy, X11: arboard)

### Priority 2: Screen Recording **← NEXT**
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
