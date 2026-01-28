# progress tracker

## completed

### v0.2.1 - ocr improvements
- [x] fix tokio runtime panic in wayland backend
- [x] add 3x upscaling (lanczos3) for better dpi
- [x] add color inversion for dark-mode uis
- [x] add mild contrast enhancement
- [x] confidence: 81% -> 91% on dark-mode text
- [x] fix common misreads: i→1, b→87, ?→p

### v0.2.0 - screen recording
- [x] gstreamer integration for mp4/webm
- [x] wayland support via pipewire
- [x] x11 support via ximagesrc
- [x] dynamic encoder selection (h.264, vp8, vp9, theora)
- [x] gif recording with clipboard copy
- [x] ctrl+c handling with eos

### v0.2.0 - wayland region fix
- [x] manual dbus handling for start request
- [x] bypass ashpd enum for sourcetype: 16
- [x] fix invalid value: 16 crash

### v0.1.0 - ocr module
- [x] tesseract integration
- [x] text extraction from captures and files
- [x] clipboard integration (wl-copy/arboard)
- [x] multi-language support
- [x] confidence thresholds

### v0.1.0 - wayland backend
- [x] xdg-desktop-portal via ashpd
- [x] screen/area/window capture
- [x] interactive mode support

### v0.1.0 - x11 backend
- [x] x11rb for capture
- [x] screen/area/window capture
- [x] xfixes cursor capture
- [x] pixel format detection

### v0.1.0 - image saving
- [x] png/jpeg output
- [x] cursor compositing
- [x] configurable paths and prefixes
- [x] timestamp handling

### v0.1.0 - gtk4 overlay
- [x] full-screen transparent window
- [x] mouse drag selection
- [x] live dimension display
- [x] esc to cancel

## in progress

nothing active

## todo

### blockers
- [ ] config system (yaml)
- [ ] multi-monitor support
- [ ] cli hotkey integration

### future
- [ ] scrolling capture
- [ ] annotation editor
- [ ] audio capture (pipewire negotiation issues)
- [ ] cloud upload

## system requirements

### recording
- arch: `sudo pacman -s gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly`
- ubuntu: `sudo apt install gstreamer1.0-plugins-*`
- fedora: `sudo dnf install gstreamer1-plugins-*`

### ocr
- arch: `sudo pacman -s tesseract leptonica tesseract-data-eng`
- ubuntu: `sudo apt install tesseract-ocr libtesseract-dev`
- fedora: `sudo dnf install tesseract leptonica`

### general
- rust toolchain
- gtk4 dev files
- wl-clipboard (wayland)

## notes

**wayland limitations:** area/window capture require user interaction through portal dialogs. coordinate-based capture is not possible due to security design.

**test status:** 33/33 tests passing
