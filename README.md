# <div style="font-size: 72px;">ALPHA AS FUCK</div>

### openshotX

Screenshot tool for linux users. (should) handle both x11 and wayland, but i'm doing development on wayland.

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

### Recent Progress
- Completed X11 backend implementation:
  - Direct screen/area/window capture via x11rb
  - Cursor capture with XFixes support
  - Robust pixel format handling
  - Comprehensive test coverage

Currently working on area selection in branch `feat/area-selection`. Check [ROADMAP.md](ROADMAP.md) for:
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
2. `cargo build`
3. pray it works
4. if not, fix it yourself

## License

do whatever the fuck you want public license
