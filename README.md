# <div style="font-size: 72px;">ALPHA AS FUCK</div>

### openshotX

screenshot tool for linux. handles x11 and wayland.

ever since i moved from macos to linux and cleanshot x is the one app i actually miss.

so let's rip it off and share it with the linux world for free.

i'm building this while HEAVILY assissted by LLMs (Gemini 3s, Claudes, GLM 4.7). it's fine if you don't approve etc, i don't really care. it's been 2 years since i started this project and nothing proper has come out still, and i still really miss OpenShot X. imo it's better this project exists because of AI than never having existed. you don't have to use it.

## what actually works right now

**screenshots:**
- `cargo run -- capture screen` - grabs the whole screen
- `cargo run -- capture area` - drag to select an area (x11 has gtk overlay, wayland uses portal dialogs)
- `cargo run -- capture window` - window capture (wayland: portal, x11: not implemented yet)

**OCR - text extraction from screenshots:**
- `cargo run -- capture area --ocr` - select area, extract text, copy to clipboard
- `cargo run -- ocr screenshot.png` - run ocr on existing image
- `cargo run -- ocr screenshot.png --lang eng+fra --min-conf 60` - multi-language + confidence threshold

saves to ~/Pictures by default. you can change that with `--output /some/path`

options:
- `--output <path>` - save somewhere specific
- `--no-cursor` - don't include the mouse cursor
- `--jpeg [quality]` - save as jpeg instead of png (quality 1-100)
- `--prefix <text>` - custom filename prefix
- `--ocr` - run ocr after capture and copy text to clipboard
- `--lang <code>` - ocr language (eng, fra, deu, eng+fra, etc.)
- `--min-conf <n>` - ocr confidence threshold (0-100, default 50)
- `--no-clipboard` - don't copy ocr result to clipboard

## technicals

**x11 backend:**
- uses x11rb directly (no xlib garbage)
- XFixes for cursor capture
- handles every pixel format variant (rgb/bgr, 24/32-bit, lsb/msb)
- gtk4 overlay for area selection

**wayland backend:**
- uses xdg-desktop-portal via ashpd
- works on hyprland, kde, sway, gnome
- security note: wayland doesn't let programs just capture whatever they want. area/window capture go through portal dialogs. this is a feature, not a bug.

**ocr:**
- tesseract for text extraction
- wl-copy on wayland (arboard on x11) for clipboard
- converts rgba to grayscale because tesseract likes that better

## building

1. install rust
2. install ocr deps if you want that feature:
   - arch: `sudo pacman -S tesseract leptonica tesseract-data-eng`
   - ubuntu: `sudo apt install tesseract-ocr libtesseract-dev`
   - fedora: `sudo dnf install tesseract leptonica`
3. `cargo build`
4. `cargo run -- capture area` or whatever

## what's coming (eventually)

**screen recording:**
- ffmpeg integration
- mp4, webm, gif output
- audio capture

**scrolling capture:**
- auto-scroll detection
- frame stitching

**editor:**
- annotations (arrows, boxes, text, blur)
- undo/redo

**cloud upload:**
- s3/custom server
- url shortening

## status

v0.1.0-alpha. shit works but it's early.

x11 backend: complete
wayland backend: complete
ocr: complete
gtk4 area overlay: complete

check ROADMAP.md if you want the full picture.

## license

do whatever the fuck you want public license
