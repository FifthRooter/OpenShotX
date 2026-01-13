# Development Notes

## Environment Detection Results
- Display Server: Wayland
- Compositor: Mutter (GNOME)
- Current Screenshot Setup: 
  - Full screen: `gnome-screenshot` (Ctrl+4)
  - Area select: Coming soon (Ctrl+Shift+4)

## Local Development

### Config Locations
- User config: `~/.config/cleanshitx/config.yaml`
- Screenshots: `~/Pictures/` (timestamp format: `%Y-%m-%d_%H-%M-%S`)
- Hotkeys: `~/.xbindkeysrc`

### Building
```bash
cargo build     # debug build
cargo run       # run with debug info
cargo test      # run test suite
```

### Backend Architecture

1. Display Server Abstraction:
   ```rust
   // Core trait for display server backends
   pub trait DisplayBackend {
       fn new() -> DisplayResult<Self>;
       fn capture_screen(&self) -> DisplayResult<CaptureData>;
       fn capture_area(&self, x: i32, y: i32, width: i32, height: i32) -> DisplayResult<CaptureData>;
       fn capture_window(&self, window_id: u64) -> DisplayResult<CaptureData>;
       fn is_supported() -> bool;
   }

   // Raw captured image data with metadata
   pub struct CaptureData {
       pixels: Vec<u8>,
       width: u32,
       height: u32,
       stride: u32,
       format: PixelFormat,
   }

   // Supported pixel formats
   impl PixelFormat {
       const RGB24: Self;  // 24-bit RGB (8 bits/channel)
       const RGB32: Self;  // 32-bit RGB (8 bits/channel + padding)
       const RGBA32: Self; // 32-bit RGBA (8 bits/channel)
       const BGR24: Self;  // 24-bit BGR (8 bits/channel)
       const BGR32: Self;  // 32-bit BGR (8 bits/channel + padding)
   }
   ```

2. Wayland Handling (COMPLETED):
   - Primary: xdg-desktop-portal via ashpd library
   - Supports: Screen, area (interactive), window (interactive)
   - Tested on: Hyprland, works on KDE/Sway/GNOME
   - Limitations: No programmatic area/window capture due to security model

3. X11 Handling (Implemented):
   - Direct XGetImage via x11rb for screen/area/window capture
   - XFixes extension for cursor capture with fallback
   - Robust pixel format handling:
     - Automatic bit depth detection (24/32-bit)
     - LSB/MSB byte order support
     - RGB/BGR format conversion
   - Error handling:
     - Custom error types for connection/reply errors
     - Coordinate validation and bounds checking
     - Graceful fallback when XFixes unavailable
   - Comprehensive test coverage:
     - Unit tests for format detection and error cases
     - Integration tests with X11 availability checks

4. Config Structure (WIP):
```yaml
display:
  backend: auto  # or 'x11', 'wayland'
  fallback: true # use fallback methods if primary fails

paths:
  screenshots: ~/Pictures
  config: ~/.config/cleanshitx

hotkeys:
  full_screen: Control + 4
  area_select: Control + Shift + 4
  window_select: Control + Alt + 4

capture:
  format: png
  quality: 100
  include_cursor: true
  timestamp_format: "%Y-%m-%d_%H-%M-%S"
```

### Testing Matrix
- [x] X11 Backend
  - [x] Screen capture with pixel format handling
  - [x] Area selection with coordinate validation
  - [x] Window capture with geometry detection
  - [x] Cursor capture with XFixes
  - [x] Error cases and edge conditions
- [x] Wayland Backend
  - [x] Full screen capture
  - [x] Area selection (interactive mode)
  - [x] Window selection (interactive mode)
  - [x] Error cases and portal cancellation handling

### Known Issues
1. Need to handle different DPI scales
2. Portal permissions on first run
3. GTK overlay needs compositor support
4. X11 window decorations not captured
5. Compositor effects (shadows, transparency) may affect capture
