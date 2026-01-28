# openshotx roadmap

linux screenshot tool inspired by cleanshot x (macos).

## philosophy

- native rust, no electron
- works on x11 and wayland
- fast and lightweight
- simple when you want it, powerful when you need it

## current state (v0.2.0)

### done
- [x] x11 backend (x11rb)
- [x] wayland backend (xdg-desktop-portal)
- [x] screen/area/window capture
- [x] cursor capture
- [x] gtk4 area selection overlay
- [x] screen recording (mp4/webm/gif)
- [x] ocr text extraction
- [x] clipboard integration

## priority features

### v0.1.0 - core capture (done)
- [x] screen/area/window capture
- [x] x11 and wayland backends
- [x] gtk4 selection overlay
- [x] save to ~/pictures
- [x] cursor capture

### v0.2.0 - recording (done)
- [x] mp4/webm recording via gstreamer
- [x] gif recording with clipboard copy
- [x] automatic codec fallback
- [ ] audio capture (blocked by pipewire negotiation issues)

### v0.3.0 - ocr (done)
- [x] tesseract integration
- [x] text extraction from screenshots
- [x] optimized for dark-mode uis
- [x] clipboard copy

### upcoming
- [ ] scrolling capture
- [ ] annotation editor
- [ ] config system
- [ ] multi-monitor improvements
- [ ] cloud upload support

## nice to have

- [ ] smart window tracking
- [ ] timelapse mode
- [ ] url shortening for uploads
- [ ] custom keybindings

## technical challenges

### x11
- pixel format variety (solved)
- cursor capture via xfixes (solved)
- window decorations (pending)

### wayland
- protocol fragmentation (solved via portals)
- security model requires user interaction (known limitation)
- area/window capture must go through portal dialogs (by design)

### performance
- fast pixel buffer handling
- efficient format conversion
- memory management for recordings

## testing

- unit tests for core logic
- integration tests for backends
- manual testing on:
  - x11: gnome, kde, i3, bspwm
  - wayland: gnome, kde, sway, hyprland
  - multi-monitor and hidpi configs

## release criteria (v1.0)

1. stable across all supported environments
2. feature parity with cleanshot x basics
3. < 50mb binary size
4. < 100ms capture latency
5. zero external deps for basic operation
