# Progress Tracker

## Completed

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

## Notes

**Wayland limitations:** Area/window capture require user interaction through portal dialogs. Coordinate-based capture is intentionally not possible.

**Test status:** 33/33 tests passing (22 backend + 11 capture)
