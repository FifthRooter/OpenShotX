# Implementation Log

A chronological record of what was implemented, attempted, failed, and abandoned.

## 2026-04-26 - v0.2.2 Session

### Successfully Implemented

#### 1. Clipboard Integration for All Capture Types
**Problem:** `capture area` saved an image to disk but didn't copy it to clipboard (unlike OCR and GIF which did).

**Solution:** Added `copy_image_to_clipboard()` function in `src/capture/mod.rs`:
- Uses `wl-copy --type image/png` on Wayland
- Uses `xclip -selection clipboard -t image/png` on X11
- Reads the actual image file and pipes to clipboard tool
- Called automatically after every successful capture (except OCR which has its own text clipboard)

**Files modified:**
- `src/capture/mod.rs` - added function and SaveError::Clipboard variant
- `src/lib.rs` - exported new function
- `src/main.rs` - called after save_capture in run_capture()

**Testing:** `Super+Ctrl+1` (capture area) → image copied to clipboard ✓

---

#### 2. Hyprland Keybindings
**Problem:** Running commands via terminal defeats the purpose of having quick access to screenshot tools.

**Solution:** Added keybindings to `~/.config/hypr/bindings.conf`:
```
Super+Ctrl+1 → openshotx capture area
Super+Ctrl+2 → openshotx capture area --ocr
Super+Ctrl+3 → openshotx record area --gif
Super+Ctrl+4 → openshotx capture screen
Super+Ctrl+5 → openshotx scroll

# Additional (no shortcut yet):
# Super+Ctrl+6 → openshotx record screen
# Super+Ctrl+7 → openshotx ocr
```

---

#### 3. Binary Installation
**Problem:** Running via `cargo run` is slow and requires being in the project directory.

**Solution:** Built release binary and installed to `~/.local/bin/openshotx`:
```bash
cargo build --release
install -Dm755 target/release/cleanshitx ~/.local/bin/openshotx
```

Binary is now in PATH and responds to `openshotx --help`.

---

### Attempted / Deferred

#### 4. Wayland Portal Bypass for area recording
**Problem:** Running `record area --gif` on Wayland shows a portal dialog requiring user to select screen/window/area before capturing. User wanted to jump straight to region selection.

**Analysis:**
- Wayland screencast portal (`xdg-desktop-portal`) is designed with security in mind
- User interaction is required before any capture can begin
- There's no CLI flag to skip the chooser and go straight to region selection
- This is a fundamental design constraint, not something we can work around

**Decision:** Leave as-is. The portal interaction is a Wayland security feature.

**Note:** X11 does NOT have this issue - the GTK overlay goes straight to region selection with no dialog.

---

#### 5. Simplifying Scroll Command Dispatch
**Problem:** The `run_scroll()` function had redundant conditional logic for parsing command-line arguments.

**Before:**
```rust
if args.len() > 2 && args[2].starts_with("--") { ... }
else if args.len() == 2 || args[2].starts_with("--") { ... }  // redundant
else { ... }
```

**After:**
```rust
if let Err(e) = run_scroll(&args).await { ... }
```

**Result:** Cleaned up - all argument parsing now happens inside `run_scroll()`.

---

### Code Cleanup

#### 6. Removed Commented-Out OCR Tests
**Problem:** Two tests in `src/ocr/mod.rs` were commented out with `TODO: Fix these tests - rgba_to_luma function is missing`. The function was refactored away (luma calculation is now inline in `preprocess_image()`) but tests remained.

**Solution:** Removed the commented-out tests entirely.

---

#### 7. Removed Unused Variable
**Problem:** `stable_count` variable in `src/scrolling/mod.rs` was declared but never used. The auto-stop feature was removed (user presses ENTER to stop), but variable remained.

**Solution:** Removed `stable_count` variable.

---

### Documentation Updates

- **PROGRESS.md** - Updated with v0.2.2 entry, marked scrolling capture as "in progress" with known issues, updated test count to 56
- **README.md** - Added keyboard shortcuts section, updated clipboard behavior description, added xclip to install instructions
- **TASK_WRAPUP.md** - Created new document outlining how to properly wrap up tasks
- **SCROLLING_CAPTURE.md** - Already exists with detailed analysis of scrolling capture issues

---

## 2026-04-XX - v0.2.1 Session (Historical)

### OCR Improvements
- Fix tokio runtime panic in wayland backend
- Add 3x upscaling (lanczos3) for better dpi
- Add color inversion for dark-mode uis
- Add mild contrast enhancement
- Result: confidence 81% → 91% on dark-mode text

**See PROGRESS.md for full history of earlier sessions.**

---

## Known Outstanding Issues

### Scrolling Capture (v0.3.0 - Partial Implementation)
**Status:** Works but has significant quality issues

**Known Issues:**
1. **Duplicate frames** - GStreamer produces frames at fixed 25fps, slow scrolling = repeated frames. 67% of captured frames were duplicates in testing.
2. **Wayland portal region limitation** - Captures more than selected region. Portal doesn't provide region metadata to crop stream.
3. **Slow stitching performance** - O(n²) overlap detection is too slow for many frames.
4. **Poor overlap detection** - Simple pixel difference doesn't handle subpixel scrolling well.

**Recommendations from SCROLLING_CAPTURE.md:**
- HIGH: Implement deduplication during capture
- HIGH: Optimize overlap detection algorithm
- MEDIUM: Use adaptive capture rate
- MEDIUM: Better overlap detection (feature-based matching)
- LOW: Real-time preview, scroll direction detection

**Current workaround:** User presses ENTER to stop, manual stitching of captured frames.

---

## Deferred / Abandoned Ideas

### Audio Capture
**Status:** Blocked - pipewire negotiation issues

**Reason:** Cannot negotiate audio stream alongside video in current GStreamer pipeline.

**Impact:** Medium - audio is not a priority for MVP

---

### Config System (YAML)
**Status:** Not started

**Reason:** Not needed for MVP, can hardcode for now

---

### Multi-Monitor Support
**Status:** Not started

**Reason:** Limited testing environment, works on single monitor

---

### Annotation Editor
**Status:** Not started

**Reason:** Future feature, not needed for MVP

---

### Cloud Upload
**Status:** Not started

**Reason:** Future feature, requires server infrastructure

---

## Test Summary

| Category | Count |
|----------|-------|
| Unit tests (lib) | 56 |
| Integration tests | 13 |
| Total | 69 |

**Note:** Earlier PROGRESS.md mentioned 33/33 tests - count has grown with new modules.