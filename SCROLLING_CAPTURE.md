# Scrolling Capture Implementation

## Status: **PARTIALLY WORKING** 🟡

The scrolling capture feature is implemented and partially functional, but has significant quality issues that make it unsuitable for production use.

## What Works ✅

1. **PipeWire/GStreamer Integration**
   - Successfully establishes one-time portal connection (no repeated dialogs)
   - Pulls frames continuously from PipeWire stream
   - Works on both X11 and Wayland

2. **Activity Detection**
   - Detects when user starts scrolling (pixel diff > 15)
   - Only begins capturing after significant movement
   - Updates reference frame while waiting to keep it fresh

3. **User-Controlled Stop**
   - User presses ENTER to finish capturing
   - Ctrl+C to cancel entirely
   - No premature auto-stop (removed stability detection)

4. **Basic Stitching**
   - Captures multiple frames
   - Attempts to stitch overlapping frames
   - Rejects identical frames (diff < 3)

## What Doesn't Work Well ❌

### 1. **Duplicate Frames (Critical Issue)**
**Problem**: Many duplicate frames are captured and stitched together.

**Root Cause**:
- GStreamer `videorate` element produces frames at fixed 25fps
- If scrolling is slow or pauses, we get the same frame multiple times
- 200ms capture interval × slow scrolling = repeated frames

**Evidence from logs**:
```
Frame 4 - diff: 0      ← duplicate
Frame 5 - diff: 0      ← duplicate
Frame 6 - diff: 0      ← duplicate
Frame 7 - diff: 0      ← duplicate
Frame 8 - diff: 13     ← finally different!
```

**Impact**: Output image has repeating sections, wasting vertical space and creating visual glitches.

### 2. **Wayland Portal Region Limitation**
**Problem**: Captures more than the selected region.

**Root Cause**:
- Wayland screencast portal captures entire monitor/window
- Region selection metadata is not being used to crop the stream
- We capture the full PipeWire stream without cropping

**Evidence**: User reported "selected area + a little bit below" was captured.

**Impact**: Cannot capture precise regions on Wayland (security limitation).

### 3. **Slow Stitching Performance**
**Problem**: Takes several seconds to stitch 24 frames.

**Root Cause**:
- Overlap detection is O(n²) in frame height
- For each new frame, searches entire height for overlap
- Python-like nested loops instead of optimized algorithms

**Impact**: Poor user experience for long scrolling sessions.

### 4. **Poor Overlap Detection**
**Problem**: Frames don't align correctly when stitched.

**Root Cause**:
- Simple pixel difference doesn't handle subpixel scrolling well
- Threshold of 10 might be too high/low depending on content
- Doesn't use more sophisticated algorithms (SIFT, feature matching)

**Impact**: Misaligned seams, cut-off content between frames.

## Technical Architecture

### Current Flow

```
1. User runs `cargo run -- scroll`
2. Wayland portal opens → user selects area
3. GStreamer pipeline starts:
   pipewiresrc → videoconvert → videorate → appsink
4. Pull frames every 10ms, process every 200ms
5. Phase 1: Wait for activity (diff > 15)
6. Phase 2: Capture all frames until ENTER pressed
7. Stitch frames using overlap detection
8. Save to ~/Pictures/
```

### GStreamer Pipeline
```
pipewiresrc path={node_id} do-timestamp=true
  → videoconvert
  → videorate
  → video/x-raw,format=RGBA,framerate=25/1
  → appsink name=sink emit-signals=true sync=false drop=false max-buffers=200
```

## Potential Improvements

### High Priority (Fixes Critical Issues)

#### 1. **Deduplicate Frames Before Stitching**
**Approach**: Calculate diff before adding to frames list, skip if too similar.

```rust
// Before adding frame:
if !frames.is_empty() {
    let diff = frames.last().unwrap().calculate_diff(&current_frame);
    if diff < 5 {  // Threshold for duplicate
        continue;  // Skip this frame
    }
}
frames.push(current_frame);
```

**Expected Impact**: 50-70% reduction in duplicate frames, much cleaner output.

#### 2. **Optimize Overlap Detection**
**Current**: Linear search through all possible overlaps, O(n²)

**Better Options**:
- Use OpenCV template matching (GPU accelerated)
- Implement downsampling pyramid (compare at 1/4 resolution first)
- Use hashing for quick rejection (perceptual hash)
- Only search bottom 20% and top 20% of frames

**Expected Impact**: 10-100x faster stitching.

#### 3. **Add Frame Deduplication at Capture Time**
**Approach**: Maintain rolling hash of last frame, only capture if hash changes significantly.

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

let mut hasher = DefaultHasher::new();
for pixel in image.pixels() {
    pixel.hash(&mut hasher);
}
let hash = hasher.finish();

if last_hash.map_or(true, |h| (h as i64 - hash as i64).abs() > 1000) {
    frames.push(current_frame);
    last_hash = Some(hash);
}
```

**Expected Impact**: Prevent most duplicates at source.

### Medium Priority (Improves Quality)

#### 4. **Better Overlap Detection Algorithm**
**Current**: Simple pixel difference

**Better Options**:
- **Multi-scale template matching**: Search at multiple scales
- **Feature-based matching**: SIFT, ORB, AKAZE features
- **Phase correlation**: FFT-based alignment
- **Edge-based matching**: More robust to lighting changes

**Expected Impact**: Better alignment, fewer visual artifacts.

#### 5. **Adaptive Capture Rate**
**Approach**: Adjust capture interval based on scroll speed.

```rust
// If scrolling fast (high diff), capture more frequently
// If scrolling slow (low diff), capture less frequently
let interval = if diff > 30 {
    Duration::from_millis(100)  // Fast scroll
} else if diff > 15 {
    Duration::from_millis(200)  // Normal
} else {
    Duration::from_millis(400)  // Slow
};
```

**Expected Impact**: Fewer duplicates, better coverage.

#### 6. **Handle Wayland Region Cropping**
**Approach**: Parse portal response for region metadata, crop frames.

```rust
// Check if stream has position/size metadata
let (x, y, w, h) = extract_crop_from_stream_metadata(&caps, &streams);
// Crop each frame to this region
let cropped = image.view(x, y, w, h);
```

**Expected Impact**: Accurate region capture on Wayland.

### Low Priority (Nice to Have)

#### 7. **Real-time Preview**
**Approach**: Show preview of stitched result as capture progresses.

**Expected Impact**: User can see quality immediately, adjust if needed.

#### 8. **Scroll Direction Detection**
**Approach**: Detect scrolling up/down/left/right and align accordingly.

**Expected Impact**: Support horizontal scrolling.

#### 9. **Variable Scroll Speed Handling**
**Approach**: Detect if user is scrolling smoothly vs in bursts.

**Expected Impact**: Better frame selection for different scroll patterns.

## Alternative Approaches

### 1. **Video-Based Approach**
Instead of capturing individual frames, record a video of the scrolling:
1. Start video recording
2. User scrolls
3. Stop recording
4. Use FFmpeg to extract keyframes: `ffmpeg -i video.mp4 -vf "select='gt(scene,0.1)'" -vsync 0 frames%d.png`
5. Stitch extracted frames

**Pros**:
- Simpler implementation
- FFmpeg has optimized scene detection
- Can use `-vsync vfr` for variable frame rate

**Cons**:
- Large intermediate video file
- Post-processing step required

### 2. **Browser-Based Approach (for web content)**
1. Use headless browser (Chrome/Firefox) with DevTools Protocol
2. Execute JavaScript to scroll and capture
3. Browser knows exact scroll position

**Pros**:
- Pixel-perfect capture
- Can control scroll speed
- Can handle dynamic content loading

**Cons**:
- Only works for web content
- Heavy dependency

### 3. **OCR-Based Alignment**
Instead of pixel-based overlap, use OCR text to align frames:
1. Extract text from each frame
2. Find overlapping text regions
3. Align based on text content

**Pros**:
- More robust to visual changes
- Can handle animations

**Cons**:
- Requires text in content
- OCR is slow
- Alignment errors possible

## Lessons Learned

1. **PipeWire is the right approach for Wayland** - One portal dialog is much better than repeatedly triggering it.

2. **Frame deduplication is critical** - At fixed framerates, scrolling creates many duplicate frames that must be filtered out.

3. **User control > auto-detection** - Trying to auto-detect when scrolling stops was unreliable. User knows best when they're done.

4. **Overlap detection is hard** - Simple pixel difference doesn't work well for real-world scrolling with subpixel movement, animations, and dynamic content.

5. **Performance matters** - O(n²) overlap detection becomes a bottleneck with many frames.

## Recommendations

### For Production Use
**Don't use this implementation as-is.** The duplicate frame issue makes it unreliable for actual use.

### For Testing/Development
1. **Implement deduplication first** (see improvement #1)
2. **Test with simple content** (solid backgrounds, clear text)
3. **Use smaller capture areas** (reduces processing time)

### For Production-Ready Solution
Consider using:
- **Video-based approach** with FFmpeg scene detection (most reliable)
- **Specialized tools** like:
  - **ShareX** (Windows, but reference implementation)
  - **ScreenToGif** with optimization
  - **Flameshot** with scroll capture plugin

## Testing Results

### Test 1: Static content, slow scroll
**Result**: Many duplicate frames, poor stitching
**Frames captured**: 24
**Unique frames**: ~8
**Duplicates**: ~16 (67%)

### Test 2: Fast scroll
**Result**: Fewer duplicates, but still significant
**Better than slow scrolling, but still problematic.**

## Files Modified

- `src/scrolling/mod.rs` - Main scrolling capture implementation
- `src/lib.rs` - Added scrolling module exports
- `src/main.rs` - Added `scroll` command

## Dependencies

- GStreamer (pipewiresrc, videoconvert, videorate, appsink)
- ashpd (Wayland portal)
- image crate (stitching)
- tokio (async runtime)

## Future Work

If continuing development, priority order:
1. **Deduplication during capture** (High impact, low effort)
2. **Optimize overlap detection** (High impact, medium effort)
3. **Better overlap algorithms** (Medium impact, high effort)
4. **Adaptive capture rate** (Medium impact, medium effort)
5. **Real-time preview** (Low impact, high effort)
