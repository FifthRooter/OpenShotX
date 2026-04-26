//! Scrolling capture module
//!
//! This module provides functionality to capture scrolling content by:
//! 1. Establishing a PipeWire/GStreamer stream (one portal dialog)
//! 2. Pulling frames at timed intervals while user scrolls
//! 3. Detecting when scrolling has stopped (pixel diff threshold)
//! 4. Stitching overlapping frames into a single tall image

use crate::capture::SaveConfig;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gst::prelude::*;
use image::{imageops, GenericImageView, RgbaImage};
use std::time::{Duration, Instant};
use std::thread;
use thiserror::Error;

/// Errors that can occur during scrolling capture
#[derive(Debug, Error)]
pub enum ScrollError {
    #[error("GStreamer error: {0}")]
    GStreamerError(String),

    #[error("Portal error: {0}")]
    PortalError(String),

    #[error("Image processing error: {0}")]
    ImageError(String),

    #[error("Stitching error: {0}")]
    StitchingError(String),

    #[error("No frames captured")]
    NoFramesCaptured,

    #[error("Capture timeout - no frames captured within {0:?}")]
    CaptureTimeout(Duration),

    #[error("Io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type ScrollResult<T> = Result<T, ScrollError>;

/// Configuration for scrolling capture
#[derive(Debug, Clone)]
pub struct ScrollCaptureConfig {
    /// Interval between frame captures (default: 200ms)
    pub capture_interval: Duration,

    /// Maximum time to wait for first frame (default: 5 seconds)
    pub first_frame_timeout: Duration,

    /// Pixel difference threshold to consider content "stable" (0-255, default: 5)
    /// Lower = more sensitive (stops sooner), Higher = less sensitive
    pub stability_threshold: u8,

    /// Number of consecutive stable frames before stopping (default: 3)
    pub stable_frame_count: usize,

    /// Minimum overlap between frames required for stitching (default: 10%)
    /// Expressed as fraction 0.0-1.0
    pub min_overlap_ratio: f32,

    /// Maximum output height in pixels (None = unlimited)
    pub max_height: Option<u32>,

    /// Save configuration for output image
    pub save_config: SaveConfig,
}

impl Default for ScrollCaptureConfig {
    fn default() -> Self {
        Self {
            capture_interval: Duration::from_millis(200),
            first_frame_timeout: Duration::from_secs(5),
            stability_threshold: 5,
            stable_frame_count: 3,
            min_overlap_ratio: 0.1,
            max_height: Some(20000),
            save_config: SaveConfig::default(),
        }
    }
}

impl ScrollCaptureConfig {
    /// Set the capture interval between frames
    pub fn with_capture_interval(mut self, interval: Duration) -> Self {
        self.capture_interval = interval;
        self
    }

    /// Set the pixel difference threshold for stability detection
    pub fn with_stability_threshold(mut self, threshold: u8) -> Self {
        self.stability_threshold = threshold.clamp(0, 100);
        self
    }

    /// Set the number of stable frames required to stop
    pub fn with_stable_frame_count(mut self, count: usize) -> Self {
        self.stable_frame_count = count.max(1);
        self
    }

    /// Set the minimum overlap ratio for stitching
    pub fn with_min_overlap_ratio(mut self, ratio: f32) -> Self {
        self.min_overlap_ratio = ratio.clamp(0.0, 0.9);
        self
    }

    /// Set the maximum output height
    pub fn with_max_height(mut self, height: u32) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set no maximum height limit
    pub fn with_unlimited_height(mut self) -> Self {
        self.max_height = None;
        self
    }
}

/// A captured frame with metadata
#[derive(Debug, Clone)]
struct CapturedFrame {
    /// The image data
    image: RgbaImage,
}

impl CapturedFrame {
    /// Calculate the difference between this frame and another
    /// Returns a value 0-255 representing average pixel difference
    fn calculate_diff(&self, other: &CapturedFrame) -> u8 {
        // Compare overlapping region only
        let width = self.image.width().min(other.image.width());
        let height = self.image.height().min(other.image.height());

        if width == 0 || height == 0 {
            return 255; // Maximum difference
        }

        let mut total_diff: u64 = 0;
        let pixel_count = (width * height) as u64;

        for y in 0..height {
            for x in 0..width {
                let p1 = self.image.get_pixel(x, y);
                let p2 = other.image.get_pixel(x, y);

                // Calculate per-channel difference
                let diff = (p1[0] as i16 - p2[0] as i16).abs()
                    + (p1[1] as i16 - p2[1] as i16).abs()
                    + (p1[2] as i16 - p2[2] as i16).abs();

                total_diff += diff as u64;
            }
        }

        // Average difference per pixel per channel (0-255)
        (total_diff / (pixel_count * 3)) as u8
    }

    /// Find the vertical overlap offset with another frame
    fn find_overlap(&self, other: &CapturedFrame, min_overlap_ratio: f32) -> Option<(u32, u32)> {
        let self_height = self.image.height() as i32;
        let other_height = other.image.height() as i32;

        if self_height <= 0 || other_height <= 0 {
            return None;
        }

        // If frames are identical (calculated earlier), don't try to stitch
        // Just return None so we append the full frame
        let quick_diff = self.calculate_diff(other);
        if quick_diff < 3 {
            // Frames are too similar, might be duplicates
            return None;
        }

        let min_overlap = (self_height as f32 * min_overlap_ratio) as i32;
        let max_overlap = self_height.min(other_height) - 10;

        for overlap in (min_overlap..max_overlap).rev() {
            let self_start = self_height - overlap;
            let mut total_diff: u64 = 0;
            let compare_pixels = (overlap * self.image.width() as i32) as u64;

            for y in 0..overlap {
                let self_y = (self_start + y) as u32;
                let other_y = y as u32;

                for x in 0..self.image.width().min(other.image.width()) {
                    let p1 = self.image.get_pixel(x, self_y);
                    let p2 = other.image.get_pixel(x, other_y);

                    let diff = (p1[0] as i16 - p2[0] as i16).abs()
                        + (p1[1] as i16 - p2[1] as i16).abs()
                        + (p1[2] as i16 - p2[2] as i16).abs();

                    total_diff += diff as u64;
                }
            }

            let avg_diff = (total_diff / (compare_pixels * 3)) as u8;

            // Stricter threshold for overlap detection
            if avg_diff < 10 {
                return Some((overlap as u32, overlap as u32));
            }
        }

        None
    }
}

/// Result of a scrolling capture operation
#[derive(Debug)]
pub struct ScrollCaptureResult {
    /// The stitched output image
    pub image: RgbaImage,
    /// Number of frames captured
    pub frame_count: usize,
    /// Total time taken for capture
    pub duration: Duration,
    /// Final output height
    pub output_height: u32,
}

/// Capture scrolling content using PipeWire/GStreamer
///
/// This is the main entry point for scrolling capture. It:
/// 1. Initializes GStreamer
/// 2. Sets up PipeWire screencast (one portal dialog)
/// 3. Captures frames at timed intervals
/// 4. Detects when scrolling stops
/// 5. Stitch frames into a single image
pub async fn capture_scrolling_pw(config: &ScrollCaptureConfig) -> ScrollResult<ScrollCaptureResult> {
    use std::collections::HashMap;
    use zbus::zvariant::OwnedValue;
    use ashpd::desktop::screencast::{Screencast, CursorMode, SourceType};
    use ashpd::desktop::PersistMode;
    use zbus::zvariant::Value;

    let start_time = Instant::now();

    // Initialize GStreamer
    gst::init()
        .map_err(|e| ScrollError::GStreamerError(format!("Failed to init GStreamer: {}", e)))?;

    println!("Setting up scrolling capture...");
    println!("Please select the area you want to capture scrolling content from...");

    // Setup PipeWire screencast portal
    let proxy = Screencast::new().await
        .map_err(|e| ScrollError::PortalError(format!("Failed to create Screencast proxy: {}", e)))?;

    let session = proxy.create_session().await
        .map_err(|e| ScrollError::PortalError(format!("Failed to create session: {}", e)))?;

    let connection = proxy.connection();

    // Select sources
    proxy.select_sources(
        &session,
        CursorMode::Embedded,
        SourceType::Monitor | SourceType::Window,
        false,
        None,
        PersistMode::DoNot,
    ).await.map_err(|e| ScrollError::PortalError(format!("Failed to select sources: {}", e)))?;

    // Start the screencast
    let msg = connection.call_method(
        Some("org.freedesktop.portal.Desktop"),
        "/org/freedesktop/portal/desktop",
        Some("org.freedesktop.portal.ScreenCast"),
        "Start",
        &(&session, "", HashMap::<String, Value>::new()),
    ).await.map_err(|e| ScrollError::PortalError(format!("Start call failed: {}", e)))?;

    let request_path: zbus::zvariant::OwnedObjectPath = msg.body().deserialize()
        .map_err(|e| ScrollError::PortalError(format!("Failed to parse Start response: {}", e)))?;

    let results: HashMap<String, OwnedValue> = wait_for_response(connection, &request_path).await?;

    let streams_value = results.get("streams")
        .ok_or_else(|| ScrollError::PortalError("No streams in portal response".into()))?;

    let streams: Vec<(u32, HashMap<String, OwnedValue>)> = streams_value.try_clone().unwrap()
        .try_into()
        .map_err(|e| ScrollError::PortalError(format!("Invalid streams format: {}", e)))?;

    let stream = streams.first()
        .ok_or_else(|| ScrollError::PortalError("No streams returned".into()))?;

    let node_id = stream.0;
    println!("✓ Connected to PipeWire stream (Node ID: {})", node_id);
    println!("\n=== SCROLLING CAPTURE ===");
    println!("Start scrolling now!");
    println!("Press ENTER when you're done (capture will stop automatically).\n");

    // Build GStreamer pipeline: PipeWire -> videoconvert -> videorate -> appsink
    // Using the same pipeline structure as GIF recording which works
    let pipeline_str = format!(
        "pipewiresrc path={} do-timestamp=true ! videoconvert ! videorate ! video/x-raw,format=RGBA,framerate=25/1 ! appsink name=sink emit-signals=true sync=false drop=false max-buffers=200",
        node_id
    );

    println!("Pipeline: {}", pipeline_str);

    let pipeline = gst::parse::launch(&pipeline_str)
        .map_err(|e| ScrollError::GStreamerError(format!("Failed to parse pipeline: {}", e)))?
        .downcast::<gst::Pipeline>()
        .map_err(|_| ScrollError::GStreamerError("Cast to Pipeline failed".into()))?;

    let appsink = pipeline.by_name("sink")
        .ok_or_else(|| ScrollError::GStreamerError("AppSink not found".into()))?
        .downcast::<gst_app::AppSink>()
        .map_err(|_| ScrollError::GStreamerError("Cast to AppSink failed".into()))?;

    // Start the pipeline with better error handling
    if let Err(err) = pipeline.set_state(gst::State::Playing) {
        eprintln!("Failed to set pipeline to Playing: {}", err);

        // Check the bus for error messages
        if let Some(bus) = pipeline.bus() {
            while let Some(msg) = bus.pop() {
                if let gst::MessageView::Error(err) = msg.view() {
                    eprintln!("Detailed Error from {}: {}",
                        err.src().map(|s| s.name()).unwrap_or("unknown".into()),
                        err.error()
                    );
                    if let Some(debug) = err.debug() {
                        eprintln!("Debug Info: {}", debug);
                    }
                }
            }
        }

        let _ = pipeline.set_state(gst::State::Null);
        return Err(ScrollError::GStreamerError(format!("State change failed: {}", err)));
    }

    // Capture frames
    let mut frames: Vec<CapturedFrame> = Vec::new();
    let mut activity_detected = false; // Phase 1: wait for movement
    let first_frame_deadline = Instant::now() + config.first_frame_timeout;
    let mut last_capture_time = Instant::now();

    // Spawn a thread to listen for ENTER key
    let enter_pressed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let enter_pressed_clone = enter_pressed.clone();

    thread::spawn(move || {
        use std::io;
        let mut line = String::new();
        // Just wait for one line (ENTER key)
        let _ = io::stdin().read_line(&mut line);
        enter_pressed_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    // Handle Ctrl+C
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        // Check if ENTER was pressed
        if enter_pressed.load(std::sync::atomic::Ordering::SeqCst) {
            if activity_detected {
                println!("\n✓ ENTER pressed - stopping capture!");
            } else {
                println!("\n✓ ENTER pressed - no scrolling detected, using captured frames.");
            }
            break;
        }

        // Pull samples continuously (like GIF recording) but only process at intervals
        tokio::select! {
            _ = &mut ctrl_c => {
                println!("\nCtrl+C received, stopping capture...");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(10)) => {
                // Pull a sample from the stream (very short timeout to get latest frame)
                match appsink.try_pull_sample(gst::ClockTime::from_mseconds(10)) {
                    Some(sample) => {
                        let buffer = sample.buffer()
                            .ok_or_else(|| ScrollError::GStreamerError("No buffer in sample".into()))?;

                        let map = buffer.map_readable()
                            .map_err(|_| ScrollError::GStreamerError("Failed to map buffer".into()))?;

                        let caps = sample.caps()
                            .ok_or_else(|| ScrollError::GStreamerError("No caps".into()))?;

                        let structure = caps.structure(0)
                            .ok_or_else(|| ScrollError::GStreamerError("No structure".into()))?;

                        let width = structure.get::<i32>("width")
                            .map_err(|_| ScrollError::GStreamerError("No width".into()))? as u32;

                        let height = structure.get::<i32>("height")
                            .map_err(|_| ScrollError::GStreamerError("No height".into()))? as u32;

                        // Convert RGBA buffer to RgbaImage
                        let pixel_data = map.as_slice();
                        let image = RgbaImage::from_raw(width, height, pixel_data.to_vec())
                            .ok_or_else(|| ScrollError::ImageError("Failed to create image from buffer".into()))?;

                        let current_frame = CapturedFrame { image };

                        // Only process frames at the target interval (e.g., every 200ms)
                        let time_since_last = last_capture_time.elapsed();
                        if time_since_last < config.capture_interval && !frames.is_empty() {
                            // Skip this frame, not enough time has passed
                            continue;
                        }

                        last_capture_time = Instant::now();

                        // Check if we have the first frame
                        if frames.is_empty() {
                            println!("✓ First frame captured ({}x{})", width, height);
                            println!("Waiting for you to start scrolling...\n");
                            frames.push(current_frame);
                            continue;
                        }

                        // Compare with last frame
                        let diff = frames.last()
                            .map(|last| last.calculate_diff(&current_frame))
                            .unwrap_or(255);

                        // Phase 1: Wait for scrolling to start (detect activity)
                        if !activity_detected {
                            // Higher threshold for activity detection (must be SIGNIFICANT movement)
                            if diff > 15 {
                                activity_detected = true;
                                println!("● Activity detected - capturing scroll...");
                                println!("   Press ENTER when finished scrolling\n");
                                // Don't continue - fall through to capture this frame
                            } else {
                                // Still waiting, don't count this frame
                                if diff > 0 {
                                    println!("  Waiting for scrolling... (diff: {} - need more movement)", diff);
                                } else {
                                    println!("  Waiting for scrolling... (diff: {})", diff);
                                }
                                frames[0] = current_frame; // Update reference frame
                                continue;
                            }
                        }

                        // Phase 2: Scrolling started - just capture continuously
                        // User will press ENTER to finish
                        println!("  Frame {} - diff: {}", frames.len() + 1, diff);
                        frames.push(current_frame);

                        // Check max height
                        if let Some(max_h) = config.max_height {
                            let estimated_height = estimate_height(&frames);
                            if estimated_height > max_h {
                                println!("✓ Reached maximum height limit ({})", max_h);
                                break;
                            }
                        }

                        // Check timeout
                        if frames.is_empty() && Instant::now() > first_frame_deadline {
                            return Err(ScrollError::CaptureTimeout(config.first_frame_timeout));
                        }
                    }
                    None => {
                        // No sample available yet, continue waiting
                        if frames.is_empty() && Instant::now() > first_frame_deadline {
                            return Err(ScrollError::CaptureTimeout(config.first_frame_timeout));
                        }
                    }
                }
            }
        }
    }

    // Stop the pipeline
    pipeline.set_state(gst::State::Null)
        .map_err(|e| ScrollError::GStreamerError(format!("Failed to stop pipeline: {}", e)))?;

    if frames.is_empty() {
        return Err(ScrollError::NoFramesCaptured);
    }

    println!("✓ Captured {} frames in {:?}", frames.len(), start_time.elapsed());

    // Stitch frames
    let result = stitch_frames(&frames, config)?;

    println!("✓ Stitched into {}x{} image",
        result.image.width(),
        result.image.height()
    );

    Ok(result)
}

async fn wait_for_response(
    connection: &zbus::Connection,
    path: &zbus::zvariant::ObjectPath<'_>,
) -> ScrollResult<std::collections::HashMap<String, zbus::zvariant::OwnedValue>> {
    use futures_util::StreamExt;

    let match_rule = format!(
        "type='signal',interface='org.freedesktop.portal.Request',member='Response',path='{}'",
        path
    );

    let rule: zbus::MatchRule = match_rule.as_str().try_into()
        .map_err(|e| ScrollError::PortalError(format!("Invalid match rule: {}", e)))?;

    let mut stream = zbus::MessageStream::for_match_rule(
        rule,
        connection,
        Some(1),
    ).await.map_err(|e| ScrollError::PortalError(format!("Failed to create message stream: {}", e)))?;

    let message = stream.next().await
        .ok_or_else(|| ScrollError::PortalError("No response from portal".into()))?
        .map_err(|e| ScrollError::PortalError(format!("Signal error: {}", e)))?;

    let (status, results): (u32, std::collections::HashMap<String, zbus::zvariant::OwnedValue>) = message.body().deserialize()
        .map_err(|e| ScrollError::PortalError(format!("Failed to deserialize portal response: {}", e)))?;

    if status != 0 {
        return Err(ScrollError::PortalError("User cancelled the portal dialog".into()));
    }

    Ok(results)
}

fn estimate_height(frames: &[CapturedFrame]) -> u32 {
    let mut total = frames[0].image.height();
    for frame in frames.iter().skip(1) {
        total = total.saturating_add(frame.image.height() / 2);
    }
    total
}

/// Stitch multiple captured frames into a single tall image
fn stitch_frames(
    frames: &[CapturedFrame],
    config: &ScrollCaptureConfig,
) -> ScrollResult<ScrollCaptureResult> {
    if frames.is_empty() {
        return Err(ScrollError::NoFramesCaptured);
    }

    if frames.len() == 1 {
        return Ok(ScrollCaptureResult {
            image: frames[0].image.clone(),
            frame_count: 1,
            duration: Duration::ZERO,
            output_height: frames[0].image.height(),
        });
    }

    let width = frames[0].image.width();
    let start_time = Instant::now();
    let mut stitched = frames[0].image.clone();

    println!("Stitching {} frames...", frames.len());

    for (i, frame) in frames.iter().enumerate().skip(1) {
        if frame.image.width() != width {
            return Err(ScrollError::StitchingError(
                format!("Frame {} has different width ({} vs {})",
                    i + 1,
                    frame.image.width(),
                    width
                )
            ));
        }

        let temp_frame = CapturedFrame {
            image: stitched.clone(),
        };

        match temp_frame.find_overlap(frame, config.min_overlap_ratio) {
            Some((offset, _)) => {
                let new_portion = frame.image.view(0, offset as u32, width, frame.image.height() - offset);
                let new_height = stitched.height() + new_portion.height();
                let mut new_stitched = RgbaImage::new(width, new_height);

                imageops::replace(&mut new_stitched, &stitched, 0, 0);

                for y in 0..new_portion.height() {
                    for x in 0..width {
                        let pixel = new_portion.get_pixel(x, y);
                        new_stitched.put_pixel(x, stitched.height() + y, image::Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]));
                    }
                }

                stitched = new_stitched;
            }
            None => {
                let new_height = stitched.height() + frame.image.height();
                let mut new_stitched = RgbaImage::new(width, new_height);

                imageops::replace(&mut new_stitched, &stitched, 0, 0);

                for y in 0..frame.image.height() {
                    for x in 0..width {
                        let pixel = frame.image.get_pixel(x, y);
                        new_stitched.put_pixel(x, stitched.height() + y, image::Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]));
                    }
                }

                stitched = new_stitched;
            }
        }
    }

    let output_height = stitched.height();

    Ok(ScrollCaptureResult {
        image: stitched,
        frame_count: frames.len(),
        duration: start_time.elapsed(),
        output_height,
    })
}

/// Save a scrolling capture result to disk
pub fn save_scrolling_capture(
    result: &ScrollCaptureResult,
    config: &ScrollCaptureConfig,
) -> ScrollResult<std::path::PathBuf> {
    let save_config = &config.save_config;

    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let prefix = save_config.filename_prefix.as_deref().unwrap_or("scroll");
    let filename = format!("{}{}.{}", prefix, timestamp, "png");

    let output_dir = if let Some(dir) = &save_config.output_dir {
        dir.clone()
    } else {
        dirs::picture_dir()
            .ok_or_else(|| ScrollError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine Pictures directory"
            )))?
    };

    std::fs::create_dir_all(&output_dir)?;

    let output_path = output_dir.join(&filename);

    result.image.save(&output_path)
        .map_err(|e| ScrollError::ImageError(format!("Failed to save image: {}", e)))?;

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ScrollCaptureConfig::default();
        assert_eq!(config.capture_interval, Duration::from_millis(200));
        assert_eq!(config.stability_threshold, 5);
        assert_eq!(config.stable_frame_count, 3);
    }

    #[test]
    fn test_config_builder() {
        let config = ScrollCaptureConfig::default()
            .with_capture_interval(Duration::from_millis(100))
            .with_stability_threshold(10)
            .with_stable_frame_count(5);

        assert_eq!(config.capture_interval, Duration::from_millis(100));
        assert_eq!(config.stability_threshold, 10);
        assert_eq!(config.stable_frame_count, 5);
    }

    #[test]
    fn test_stability_threshold_clamping() {
        let config1 = ScrollCaptureConfig::default().with_stability_threshold(150);
        assert_eq!(config1.stability_threshold, 100);

        let config2 = ScrollCaptureConfig::default().with_stability_threshold(0);
        assert_eq!(config2.stability_threshold, 0);
    }

    #[test]
    fn test_min_overlap_ratio_clamping() {
        let config1 = ScrollCaptureConfig::default().with_min_overlap_ratio(1.5);
        assert_eq!(config1.min_overlap_ratio, 0.9);

        let config2 = ScrollCaptureConfig::default().with_min_overlap_ratio(-0.1);
        assert_eq!(config2.min_overlap_ratio, 0.0);
    }
}
