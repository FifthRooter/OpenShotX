# OCR Feature Implementation Task

## Project Overview

**OpenShotX** is a Linux screenshot tool (Rust, X11/Wayland backends) inspired by macOS's CleanShot X.
Goal: Bring CleanShot X's functionality to Linux with native backends and zero electron bloat.

**Current Status (v0.1.0-alpha):**
- X11 backend: Complete (x11rb, XFixes cursor, pixel format handling)
- Wayland backend: Complete (ashpd portal integration)
- Image saving: PNG/JPEG, cursor compositing, timestamps
- CLI frontend: `cargo run -- capture screen|area|window`
- GTK4 overlay: X11 area selection with drag preview

**Architecture Philosophy:**
- Zero bullshit: No electron, no web tech stack
- Native Rust + DBus + X11/Wayland abstraction
- Minimal dependencies, fail fast/loud
- Smart defaults, configurable everything

---

## Task: Implement OCR (Optical Character Recognition)

### Objective
Add text extraction capability from screenshots using tesseract-rs, with clipboard integration.

### User-Facing Requirements
1. **CLI Command**: `cargo run -- ocr <path_to_image>` or integrate into capture workflow
2. **Extract text** from screenshot images
3. **Language selection** (default: English, support common languages)
4. **Copy to clipboard** with optional fallback to stdout
5. **Confidence scoring** for extraction quality

---

## Technical Requirements

### 1. Dependencies to Add
```toml
# Cargo.toml
tesseract-rs = "0.1"  # or latest stable version
# Consider: leptos? for clipboard
```

### 2. Module Structure
Create new module: `src/ocr/mod.rs`

```rust
// Core OCR functionality
pub fn extract_text(image: &image::DynamicImage, lang: &str) -> OcrResult<String>

// Configuration
pub struct OcrConfig {
    pub language: String,
    pub min_confidence: f32,
    pub clipboard_output: bool,
}

// Error types
pub enum OcrError { ... }
```

### 3. Integration Points

**With capture module:**
- `src/capture/mod.rs` already handles `CaptureData` → `RgbaImage` conversion
- OCR should accept `CaptureData` directly or saved image path

**With CLI:**
- Extend `src/main.rs` with new subcommand: `ocr`
- Support piping: `capture area | ocr`

**Clipboard:**
- Research Linux clipboard options (wayland: wl-clipboard, x11: xclip)
- Or use Rust native crate if available

---

## Files to Read First

### Essential Context
1. **`src/lib.rs`** - Public API, module structure
2. **`src/backend/mod.rs`** - DisplayBackend trait, CaptureData structure
3. **`src/capture/mod.rs`** - Image conversion pipeline (CaptureData → RgbaImage)
4. **`src/main.rs`** - CLI structure and argument parsing
5. **`Cargo.toml`** - Current dependencies, project metadata

### Documentation
1. **`README.md`** - Project goals and feature list
2. **`ROADMAP.md`** - Full feature roadmap and architecture
3. **`DEVELOPMENT.md`** - Development guidelines and testing matrix
4. **`PROGRESS.md`** - Completed features and TODOs

---

## Implementation Approach

### Phase 1: Core OCR Module
1. Add `tesseract-rs` dependency
2. Create `src/ocr/mod.rs` with:
   - `OcrConfig` struct (language, confidence threshold, clipboard)
   - `extract_text()` function using tesseract-rs
   - Error handling (tesseract init, image processing, extraction)
3. Write unit tests with sample images

### Phase 2: CLI Integration
1. Add `ocr` subcommand to `src/main.rs`:
   ```bash
   cargo run -- ocr screenshot.png
   cargo run -- capture area | ocr  # pipe support
   cargo run -- capture area --ocr  # integrated mode
   ```
2. Handle language selection: `--lang eng+fra`
3. Add clipboard copy flag: `--clipboard`

### Phase 3: Capture Pipeline Integration
1. Extend `SaveConfig` in `src/capture/mod.rs` to include OCR option
2. After saving, optionally run OCR and copy to clipboard
3. Example: `cargo run -- capture area --ocr --clipboard`

### Phase 4: Error Handling & Edge Cases
1. Tesseract not installed → helpful error message
2. Low confidence text → warn user
3. No text detected → return empty string or error
4. Multi-language support → handle `eng+fra` syntax

---

## Technical Considerations

### Image Pre-processing
- Convert `CaptureData` to format tesseract accepts (likely RGB/RGBA)
- Handle various pixel formats (RGB24, BGR24, RGBA32, etc.)
- Consider image enhancement (contrast, threshold) for better OCR

### Language Support
- Default: `eng` (English)
- Support multi-language: `eng+fra+deu`
- Validate language codes against available tesseract data

### Clipboard Integration
**Options:**
1. **CLI wrappers**: `wl-copy` (Wayland), `xclip` (X11)
2. **Rust crates**: `copypasta` (cross-platform), `wl-clipboard-rs`
3. **Detect backend**: Use appropriate method based on X11/Wayland

### Performance
- Tesseract can be slow on large images
- Consider:
  - Image downsampling for text extraction
  - Progress indicators for large images
  - Caching tesseract instance

---

## Testing Strategy

### Unit Tests
1. Test OCR with sample images (text-heavy, low contrast, multi-language)
2. Test error handling (missing tesseract, invalid images)
3. Test language configuration parsing

### Integration Tests
1. Test full pipeline: capture → OCR → clipboard
2. Test on both X11 and Wayland
3. Test various image formats (PNG, JPEG)

### Manual Testing
1. Capture code editor screenshot → verify text extraction
2. Capture terminal output → verify monospace text
3. Capture mixed content (text + graphics) → verify accuracy
4. Test clipboard copy on X11 and Wayland

---

## Success Criteria

- [ ] `cargo run -- ocr screenshot.png` extracts and prints text
- [ ] `--lang` flag works for language selection
- [ ] `--clipboard` flag copies to system clipboard
- [ ] Integrated mode: `cargo run -- capture area --ocr` works end-to-end
- [ ] Helpful error messages (missing tesseract, invalid image)
- [ ] Tests pass for core OCR functionality
- [ ] Works on both X11 and Wayland

---

## Next Session Output

Expected deliverables:
1. `src/ocr/mod.rs` with full OCR implementation
2. Updated `src/lib.rs` with ocr module
3. Updated `src/main.rs` with ocr CLI subcommand
4. Updated `Cargo.toml` with tesseract-rs dependency
5. Unit tests in `src/ocr/mod.rs`
6. Documentation updates (README, PROGRESS.md)

---

## Resources

**tesseract-rs documentation:**
- crates.io: https://crates.io/crates/tesseract-rs
- GitHub: Check for examples and API docs

**Tesseract language data:**
- Install via system package manager (apt, pacman, etc.)
- Language codes: `eng`, `fra`, `deu`, `spa`, `por`, `rus`, etc.

**Clipboard on Linux:**
- Wayland: `wl-clipboard` package
- X11: `xclip` or `xsel` package
- Rust crates: `copypasta`, `smithay-clipboard`

---

## Current File Locations

**Source:**
- `/home/arbestor/code/projects/openshotx/src/`
- `/home/arbestor/code/projects/openshotx/Cargo.toml`

**Docs:**
- `/home/arbestor/code/projects/openshotx/*.md`

**Tests:**
- Inline with modules: `#[cfg(test)] mod tests`

---

## Last Commit

```
96fa7ca add GTK4 overlay for X11 area selection
```

**Branch:** main
**Working directory:** /home/arbestor/code/projects/openshotx
