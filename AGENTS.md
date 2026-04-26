# openshotX - Agent Context

A screenshot tool for Linux (X11 and Wayland). Inspired by CleanShot X.

## Project Overview

**Binary name:** `openshotx`
**Language:** Rust (native, no Electron)
**Build:** `cargo build --release` → `target/release/openshotx`

## What's Implemented ✅

### Core Features
- **Screenshot capture:** screen, area (drag to select), window
- **Screen recording:** MP4/WebM via GStreamer, GIF with FFmpeg
- **OCR text extraction:** via Tesseract, optimized for dark-mode UIs
- **Clipboard:** automatic copy for all capture types (image/png via wl-copy/xclip)

### Backends
- **X11:** x11rb crate, GTK4 overlay for area selection (no dialog, direct selection)
- **Wayland:** xdg-desktop-portal via ashpd, security by design requires user interaction

### CLI Commands
```
openshotx capture screen|area|window [--ocr] [--output <path>] [--no-cursor] [--jpeg [quality]] [--prefix <text>]
openshotx record screen|area [--gif] [--output <path>]
openshotx ocr <image> [--lang eng+fra] [--min-conf 50] [--no-clipboard]
openshotx scroll [--output <path>] [--interval 200] [--max-height 20000]
```

## File Structure

```
src/
├── main.rs           # CLI entry point, argument parsing
├── lib.rs            # Module exports, public API
├── backend/
│   ├── mod.rs        # Trait DisplayBackend, CaptureData, PixelFormat
│   ├── x11.rs        # X11Backend implementation
│   └── wayland.rs    # WaylandBackend implementation
├── capture/
│   └── mod.rs        # Image saving, format conversion, clipboard copy
├── overlay.rs        # GTK4 overlay for X11 area selection
├── ocr/
│   └── mod.rs        # Tesseract OCR with preprocessing
├── recording/
│   └── mod.rs        # GStreamer video/GIF recording
├── scrolling/
│   └── mod.rs        # Scrolling capture (PARTIALLY WORKING - has issues)
└── utils/
    └── mod.rs        # Utility functions

Cargo.toml            # Dependencies: x11rb, ashpd, gtk4, gstreamer, tesseract, image, etc.
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| x11rb | X11 capture |
| ashpd | Wayland portal communication |
| gtk4 | Area selection overlay (X11) |
| gstreamer* | Video recording |
| tesseract | OCR |
| image | Image processing |
| zbus | DBus for Wayland portal |
| tokio | Async runtime |

## Current Issues (Scrolling Capture)

**Status: PARTIALLY WORKING** - Has significant quality issues.

### Known Problems

1. **Duplicate frames (67% of captures)**
   - GStreamer produces at fixed 25fps
   - Slow scrolling = same frame multiple times
   - Location: `src/scrolling/mod.rs` - `CapturedFrame::calculate_diff()`

2. **Slow stitching (O(n²))**
   - Linear search through all possible overlaps
   - `find_overlap()` function in scrolling/mod.rs

3. **Wayland region cropping not working**
   - Portal captures full monitor, not selected region
   - Security limitation, cannot be bypassed

4. **Poor overlap detection**
   - Simple pixel difference doesn't handle subpixel scrolling
   - Threshold of 10 may be too strict/loose

### Priority Fixes

1. **Frame deduplication during capture** - Check diff before adding to list, skip if < 5
2. **Optimize overlap detection** - Downsample first, only search relevant regions
3. **Adaptive capture rate** - 100ms for fast scroll, 400ms for slow

## Development Context (Local Only)

Development docs are stored in `dev-docs/` directory. This directory is gitignored and NOT committed to the public repository.

### Dev Docs Structure
```
dev-docs/                        # All dev docs live here (gitignored)
├── SCROLLING_CAPTURE.md          # Scrolling capture detailed analysis
├── PROGRESS.md                   # Development progress tracker
├── TASK_WRAPUP.md                # Task completion checklist
├── IMPLEMENTATION_LOG.md         # Dev journal (attempts, failures, successes)
├── RECORDING_RESEARCH.md         # Old research notes
└── NEXT_SESSION.md               # Transient notes for next session
```

### How to Use Dev Docs
- **Scrolling capture work:** Read `dev-docs/SCROLLING_CAPTURE.md` - has full analysis of issues, solutions
- **Tracking progress:** Read `dev-docs/PROGRESS.md` - what was completed when
- **After completing a task:** Update docs in `dev-docs/` directory
- **New session:** Start by reading `AGENTS.md` (this file) + relevant dev docs

### Key Files for This Project
| File | Purpose |
|------|---------|
| `AGENTS.md` | This file - project context for agents |
| `README.md` | Public-facing install/use instructions |
| `ROADMAP.md` | Feature roadmap and priorities |
| `dev-docs/SCROLLING_CAPTURE.md` | **Critical for scrolling capture work** |
| `dev-docs/PROGRESS.md` | Dev history and status |

See `dev-docs/SCROLLING_CAPTURE.md` for full details on scrolling capture issues.

## Future Features

- Annotation editor (arrows, boxes, text, blur)
- Config system (YAML for user preferences)
- Multi-monitor support
- Cloud upload
- Audio capture (blocked by PipeWire negotiation issues)

## Important Notes

- **Wayland security:** Area/window capture requires portal dialog interaction. Cannot bypass.
- **X11 advantage:** GTK overlay goes straight to region selection, no dialog.
- **Binary location:** `~/.local/bin/openshotx` after install.
- **Dev docs:** Stored in `dev-docs/` (gitignored), NOT committed to repo.

## Common Tasks

### Build and install
```bash
cargo build --release
install -Dm755 target/release/openshotx ~/.local/bin/openshotx
```

### Run tests
```bash
cargo test
cargo clippy
```

### Quick capture test
```bash
openshotx capture area
```

### Work on scrolling capture
- Read `dev-docs/SCROLLING_CAPTURE.md` for context
- Main code in `src/scrolling/mod.rs`
- Focus on `CapturedFrame::calculate_diff()` and `find_overlap()`

## Workflow

When completing a task:
1. Run `cargo test && cargo clippy && cargo build --release`
2. Test the feature manually
3. Update relevant local docs (dev-docs/SCROLLING_CAPTURE.md, dev-docs/PROGRESS.md)
4. Commit with clear message describing what was done

Do NOT commit local dev docs to the public repository.