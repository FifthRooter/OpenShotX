# CleanShitX Roadmap

## Current State (v0.0.1-alpha)
- [x] Basic POC with xbindkeys
  - X11: scrot for window captures
  - Wayland: gnome-screenshot fallback
  - Hotkeys: Ctrl+4 (full), Ctrl+Shift+4 (coming soon: area)
- [x] Initial repo structure

## Core Architecture

### Backend (Rust)
```rust
// Abstract display server handling
trait DisplayBackend {
    fn capture_screen(&self) -> Result<Image>;
    fn capture_area(&self, x: i32, y: i32, w: i32, h: i32) -> Result<Image>;
    fn capture_window(&self, id: WindowId) -> Result<Image>;
}

// Concrete implementations
struct X11Backend {
    // Learned: Need XGetImage for raw pixels
    // Challenge: Handle different pixel formats (24/32bpp, BGR/RGB)
    // Note: XFixes needed for cursor capture
}

struct WaylandBackend {
    // Challenge: Different compositors support different protocols
    // Need: wlr-screencopy-unstable-v1 for Sway/Hyprland
    // Fallback: gnome-screenshot for GNOME
}
```

### DBus API
```
Interface: com.cleanshitx.Linux.Capture
Methods:
  - screenshot_area(x: i32, y: i32, w: i32, h: i32)
  - screenshot_window(window_id: u64)
  - screenshot_screen(screen_id: i32)
  - start_recording(config: RecordingConfig)
  - stop_recording()
```

## Priority Features

### 1. Core Screenshot (v0.1.0)
- [ ] Rust backend with proper abstraction
- [ ] Area selection with preview
- [ ] Window detection/selection
- [ ] Multi-monitor support
- [ ] Proper cursor handling
- [ ] Quick edit overlay
- [ ] Save to ~/Pictures with better naming

### 2. Recording (v0.2.0)
- [ ] Screen recording (FFmpeg integration)
- [ ] GIF mode with optimization
- [ ] Audio capture support
- [ ] Basic video trimming
- [ ] Format selection (mp4/webm)

### 3. Advanced Capture (v0.3.0)
- [ ] Scrolling capture
  - Challenge: Need to handle different scrolling mechanisms
  - Research: Firefox smooth scroll vs Chrome vs native
- [ ] Delayed capture
- [ ] OCR integration (tesseract-rs)
- [ ] Quick actions (copy/upload/edit)

### 4. Editor (v0.4.0)
- [ ] Basic annotations
  - Arrows, boxes, text
  - Color picker
  - Blur/pixelate
- [ ] Crop/resize
- [ ] Basic filters
- [ ] Undo/redo

## Nice to Have
- [ ] Cloud integration
  - S3/custom server support
  - URL shortening
  - Share history
- [ ] Timelapse mode
  - Configurable intervals
  - Auto-stop conditions
- [ ] Smart window tracking
  - Remember window positions
  - Auto-capture on changes
- [ ] Meme generator mode
  - Text overlay
  - Common templates
  - Export formats

## Technical Challenges

### X11
- Different pixel formats between servers
- Cursor capture requires XFixes
- Window decorations handling
- Compositor effects (shadows, transparency)

### Wayland
- Protocol fragmentation
- Limited screen capture APIs
- Need different approaches per compositor
- Security/permission models

### Performance
- Fast pixel buffer handling
- Efficient format conversion
- Memory management for recordings
- Smooth UI during capture

## Development Guidelines
- Minimal dependencies
- Fail fast, fail loud
- Extensive error handling
- Smart defaults, configurable everything
- No electron bloat

## Testing Strategy
- Unit tests for core logic
- Integration tests for capture
- Manual testing matrix:
  - X11 + common WMs
  - Wayland + major compositors
  - Multi-monitor setups
  - HiDPI configurations

## Release Criteria (v1.0)
1. Rock-solid stability
2. Full feature parity with CleanShot X
3. < 50MB binary size
4. < 100ms capture latency
5. Zero external deps for basic operation

Remember: This isn't just another screenshot tool. This is CleanShitX - the screenshot tool Linux deserves, built by chads for chads.
