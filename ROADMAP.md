# CleanShitX Roadmap

## WTF is this?
CleanShitX is what happens when Linux users get tired of normie screenshot tools. Born from the rage of switching from macOS's CleanShot X to Linux and finding nothing but bloated electron apps and command line peasantry.

### Philosophy
- **Zero Bullshit**: No electron, no web tech stack, no "cross-platform" compromises
- **Chad Architecture**: Native Rust + DBus + X11/Wayland abstraction = based AF
- **Features That Fuck**: Everything CleanShot X does + Linux-specific chad moves
- **KISS My Ass**: Simple when you want it, powerful when you need it
- **Performance or Death**: If it can't capture 4K@144Hz, it's not shipping

### Why Rust?
- Memory safety without the GC bullshit
- FFI that doesn't make you want to die
- Package ecosystem that isn't a dependency hell
- Forces you to handle edge cases at compile time
- Community of based individuals who give a fuck about performance

### Target Audience
- Linux power users who need more than `scrot`
- Ex-macOS users who miss CleanShot X
- Anyone who thinks Electron is cancer
- Screenshot chads who need that extra 0.1ms performance

## Current State (v0.0.1-alpha)
- [x] Basic POC with xbindkeys
  - X11: scrot for window captures
  - Wayland: gnome-screenshot fallback
  - Hotkeys: Ctrl+4 (full), Ctrl+Shift+4 (coming soon: area)
- [x] Initial repo structure

## In Progress (feat/area-selection)
- [x] Display server abstraction
  - [x] Core DisplayBackend trait
  - [x] Pixel format handling (RGB/BGR/RGBA)
  - [x] Raw pixel data + metadata
  - [x] Error types and validation
  - [x] Unit test coverage
- [x] Native backend implementations
  - [x] X11: direct XGetImage via x11rb
    - [x] Screen, area, and window capture
    - [x] Cursor capture via XFixes
    - [x] Pixel format detection and handling
    - [x] Comprehensive test coverage
  - [x] Wayland: xdg-desktop-portal via ashpd
    - [x] Screen capture
    - [x] Area capture (interactive mode)
    - [x] Window capture (interactive mode)
    - [x] Full test coverage
- [x] GTK4 overlay window
  - [x] Crosshair cursor
  - [x] Size display
  - [x] Escape to cancel
- [ ] Config system for paths/hotkeys

## Core Architecture

### Backend (Rust)
```rust
// Abstract display server handling
trait DisplayBackend {
    fn new() -> DisplayResult<Self>;
    fn capture_screen(&self) -> DisplayResult<CaptureData>;
    fn capture_area(&self, x: i32, y: i32, w: i32, h: i32) -> DisplayResult<CaptureData>;
    fn capture_window(&self, id: u64) -> DisplayResult<CaptureData>;
    fn is_supported() -> bool;
}

// Raw pixel data with metadata
struct CaptureData {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    stride: u32,
    format: PixelFormat,
}

// Concrete implementations (WIP)
struct X11Backend {
    // Using x11rb for XGetImage
    // XFixes for cursor capture
    // Native pixel format handling
}

struct WaylandBackend {
    // Primary: xdg-desktop-portal via ashpd
    // Handles: Screen, area (interactive), window (interactive)
    // Limitations: No programmatic area/window capture due to security model
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
  - [x] Project structure setup
  - [ ] Display server traits
  - [ ] Error handling
- [ ] Area selection with preview (IN PROGRESS)
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

### X11 (Solved)
- Different pixel formats between servers
  - Solution: Robust pixel format detection with proper bit depth handling
  - Handles LSB/MSB byte orders and various RGB formats
- Cursor capture requires XFixes
  - Solution: Optional XFixes support with graceful fallback
  - ARGB cursor pixels converted to RGBA
- Coordinate validation and type conversion
  - Solution: Proper validation and clamping for i16 coordinates
  - Error handling for out-of-bounds captures
- Error handling and testing
  - Solution: Custom error types for connection and reply errors
  - Comprehensive unit and integration tests
  - Tests handle X11 not being available gracefully

### X11 (Remaining)
- Window decorations handling
- Compositor effects (shadows, transparency)

### Wayland (Completed)
- Protocol fragmentation
  - Solution: Use xdg-desktop-portal for compositor-agnostic access
- Limited screen capture APIs
  - Solution: ashpd library handles portal interactions
- Security/permission models
  - Solution: User must interactively approve each capture
- **Known limitation:** Area/window capture require user interaction through portal dialogs

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
  - Pixel format detection and handling
  - Error types and validation
  - Coordinate handling and bounds checking
- Integration tests for capture
  - Backend initialization and support detection
  - Screen, area, and window capture
  - Cursor capture and positioning
  - Error cases and edge conditions
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
