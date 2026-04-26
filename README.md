# <div style="font-size: 72px;">ALPHA AS FUCK</div>

### openshotX

screenshot++ tool for linux. handles x11 and wayland.

ever since i moved from macos to linux CleanShot X is the one app i actually miss.

so let's rip it off and share it with the linux world for free.

i'm building this while HEAVILY assissted by LLMs (Gemini 3s, Claudes, GLM 4.7). it's fine if you don't approve etc, i don't really care. it's been 2 years since i created this repo and nothing proper has come out still. imo it's better this project exists because of AI than never having existed. you don't have to use it.

## what actually works right now

**screenshots (all auto-copy to clipboard):**
- `openshotx capture screen` - grabs the whole screen
- `openshotx capture area` - drag to select an area (x11: gtk overlay, wayland: portal dialogs)
- `openshotx capture window` - window capture (wayland: portal, x11: not implemented yet)

**screen recording:**
- `openshotx record screen` - record full screen to MP4 (Wayland/X11)
- `openshotx record area` - record selected window/monitor (Wayland) or drawn area (X11)
- `openshotx record area --gif` - record high-quality GIF and **copy to clipboard**
- automatic fallback to WebM/Theora if H.264 codecs are missing

**OCR - text extraction from screenshots:**
- `openshotx capture area --ocr` - select area, extract text, copy to clipboard
- `openshotx ocr screenshot.png` - run ocr on existing image
- `openshotx ocr screenshot.png --lang eng+fra --min-conf 60` - multi-language + confidence threshold

**scrolling capture (beta - see SCROLLING_CAPTURE.md for known issues):**
- `openshotx scroll` - capture scrolling content by stitching overlapping frames

**keyboard shortcuts (Hyprland):**
```
Super+Ctrl+1 → capture area
Super+Ctrl+2 → capture area --ocr
Super+Ctrl+3 → record area --gif
Super+Ctrl+4 → capture screen
Super+Ctrl+5 → scroll
```

All captures save to ~/Pictures (screenshots) or ~/Videos (recordings) by default. use `--output /some/path` to change.

## options

**screenshot options:**
- `--output <path>` - save somewhere specific
- `--no-cursor` - don't include the mouse cursor
- `--jpeg [quality]` - save as jpeg instead of png (quality 1-100)
- `--prefix <text>` - custom filename prefix
- `--ocr` - run ocr after capture and copy text to clipboard

**recording options:**
- `--output <path>` - save to specific path (default: ~/Videos/output.mp4)
- `--gif` - record as high-quality GIF (automatic clipboard copy)

**scrolling capture options:**
- `--output <path>` - save to specific path (default: ~/Pictures)
- `--interval <ms>` - capture interval in milliseconds (default: 200)
- `--max-height <n>` - maximum output height in pixels (default: 20000)
- `--prefix <text>` - custom filename prefix (default: 'scroll')

**ocr options:**
- `--lang <code>` - ocr language (eng, fra, deu, eng+fra, etc.)
- `--min-conf <n>` - ocr confidence threshold (0-100, default: 50)
- `--no-clipboard` - don't copy ocr result to clipboard

## technicals

**x11 backend:**
- uses x11rb directly (no xlib garbage)
- XFixes for cursor capture
- handles every pixel format variant (rgb/bgr, 24/32-bit, lsb/msb)
- gtk4 overlay for area selection - goes straight to region selection (no dialog)

**wayland backend:**
- uses xdg-desktop-portal via ashpd
- manual DBus implementation to support modern portal features (Region recording)
- works on hyprland, kde, sway, gnome
- **security note:** wayland doesn't let programs just capture whatever they want. area/window capture go through portal dialogs. this is a feature, not a bug.

**recording (MP4/WebM):**
- GStreamer pipeline with hardware-accelerated encoder fallback
- Wayland: PipeWire integration via Portal
- X11: `ximagesrc` for low-latency capture

**recording (GIF):**
- GStreamer -> FFmpeg Pipe architecture
- real-time streaming for high performance (no dropped frames)
- `palettegen` filter for superior color quality
- copies File URI (`file://`) for instant paste into Discord/Slack/Browsers

**ocr:**
- tesseract for text extraction
- wl-copy on wayland, xclip on x11
- **preprocessing pipeline optimized for ui text:**
  - 3x upscaling (lanczos3) for better dpi - tesseract prefers ~300 dpi
  - color inversion for dark-mode uis - tesseract is trained on dark-on-light documents
  - mild contrast enhancement
- **before these improvements:** ocr on dark-mode apps (telegram, discord) had frequent misreads like "i"→"1", "b"→"87", "?"→"p"
- **after:** ~91% confidence on typical chat text with proper character recognition

**clipboard:**
- screenshots: automatic copy of image/png data via wl-copy (wayland) or xclip (x11)
- ocr text: copy via wl-copy (wayland) or arboard (x11)
- gifs: copy file:// URI for compatibility with chat apps

## building

1. install rust
2. **install system dependencies:**

   **Arch Linux:**
   ```bash
   sudo pacman -S tesseract leptonica tesseract-data-eng # OCR
   sudo pacman -S gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly # Recording
   sudo pacman -S ffmpeg wl-clipboard xclip # GIF and Clipboard support
   ```

   **Ubuntu/Debian:**
   ```bash
   sudo apt install tesseract-ocr libtesseract-dev # OCR
   sudo apt install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
       gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
       gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly # Recording
   sudo apt install ffmpeg wl-clipboard xclip # GIF and Clipboard support
   ```

   **Fedora:**
   ```bash
   sudo dnf install tesseract leptonica # OCR
   sudo dnf install gstreamer1-devel gstreamer1-plugins-base-devel \
       gstreamer1-plugins-good gstreamer1-plugins-bad-free gstreamer1-plugins-ugly-free # Recording
   sudo dnf install ffmpeg wl-clipboard xclip # GIF and Clipboard support
   ```

3. `cargo build --release`
4. `install -Dm755 target/release/cleanshitx ~/.local/bin/openshotx`

## roadmap

**done:**
- x11 backend (x11rb)
- wayland backend (xdg-desktop-portal)
- screen/area/window capture
- cursor capture
- gtk4 area selection overlay
- screen recording (mp4/webm/gif)
- ocr text extraction
- clipboard integration (all capture types)
- keyboard shortcuts (hyprland)

**in progress:**
- scrolling capture (beta - has known issues with duplicate frames and overlap detection)

**upcoming:**
- annotation editor
- config system (yaml)
- multi-monitor improvements
- audio capture (pipewire negotiation issues)
- cloud upload

**nice to have:**
- smart window tracking
- timelapse mode
- url shortening for uploads
- custom keybindings

## status

- x11 backend: complete (tested on Hyprland via XWayland)
- wayland backend: complete
- ocr: complete (91% confidence on dark-mode text)
- screen recording: complete (MP4/WebM/GIF)
- gtk4 area overlay: complete
- scrolling capture: partial (works but has quality issues)
- clipboard: complete (all capture types)
- keybindings: complete (Super+Ctrl+1-5)

tested on Arch Linux + Hyprland. it's very much a 'it works on my machine' situation fyi.

check ROADMAP.md if you want the full picture.

## license

do whatever the fuck you want public license