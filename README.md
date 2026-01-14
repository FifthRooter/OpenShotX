# <div style="font-size: 72px;">ALPHA AS FUCK</div>

### openshotX

Screenshot tool for linux users. Handles both x11 and wayland with native backends.

Since moving from MacOS to Linux, the app I most miss is CleanShot X - by far the most feature-full and "just works" screenshot app out there. So now I'm trying to build one for Linux.

It's a feature-rich screen capture and recording tool designed exclusively for Linux. Inspired by the functionality and user experience of CleanShot X on macOS, this application aims to bring a similar, if not better, level of convenience and capability to the Linux environment.

#### Key Features:

- UX that gets out of the way
- Screen recording with customizable settings, including recording in GIF
- From full-screen captures to selective area snapshots, to scrolling capture
- Edit and annotate your screenshots with a variety of tools.
- OCR for extracting text from images.
- Option to directly upload and share your captures to your cloud.

Thinking about adding:
- Sensitive info blurring
- Timelapse (of screen or window)

## Status

### v0.1.0-alpha

**X11 Backend:** Complete
- Direct screen/area/window capture via x11rb
- Cursor capture with XFixes support
- Robust pixel format handling (RGB/BGR, 24/32-bit, LSB/MSB)
- Comprehensive test coverage

**Wayland Backend:** Complete
- Screen capture via xdg-desktop-portal using ashpd
- Area and window capture (interactive mode only)
- Tested on Hyprland, works on KDE/Sway/GNOME
- Full test coverage

**OCR Module:** Complete
- Text extraction via tesseract
- Multi-language support (eng, fra, deu, etc.)
- Automatic clipboard integration (wl-copy for Wayland, arboard for X11)
- Configurable confidence threshold
- CLI: `cargo run -- capture area --ocr`

**Note:** Wayland has security limitations - area/window capture require user interaction through portal dialogs. Coordinate-based and programmatic capture are intentionally not possible on Wayland.

### Recent Progress
- Completed OCR implementation:
  - tesseract integration for text extraction
  - Cross-platform clipboard support (Wayland: wl-copy, X11: arboard)
  - Standalone `ocr` command and integrated `--ocr` flag
  - Multi-language and confidence configuration
  - 41/42 tests passing

Check [ROADMAP.md](ROADMAP.md) for:
- Full feature list & progress
- Technical architecture
- Development guidelines
- Testing strategy
- Release criteria

## Install Guide

![no](https://raw.githubusercontent.com/jglovier/gifs/gh-pages/no/homero-no.gif)

wait for v0.1 you impatient fuck

## Building from source

1. get rust
2. Install OCR dependencies (optional, for OCR feature):
   - Arch: `sudo pacman -S tesseract leptonica tesseract-data-eng`
   - Ubuntu: `sudo apt install tesseract-ocr libtesseract-dev`
   - Fedora: `sudo dnf install tesseract leptonica`
3. `cargo build`
4. pray it works
5. if not, fix it yourself

## License

do whatever the fuck you want public license
