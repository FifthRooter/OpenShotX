//! Wayland backend implementation using xdg-desktop-portal via ashpd
//!
//! ## Important Wayland Limitations
//!
//! Unlike X11, Wayland's security model intentionally does not allow programmatic
//! screen/area/window capture without user interaction. The xdg-desktop-portal API:
//!
//! - Does NOT support coordinate-based area capture
//! - Does NOT support capturing windows by ID
//! - DOES require user interaction for every capture
//! - Shows a dialog for every capture (even non-interactive on some compositors)
//!
//! For this reason:
//! - `capture_area(x, y, w, h)` - parameters are ignored, user selects area interactively
//! - `capture_window(id)` - id is ignored, user selects window interactively
//! - `capture_screen()` - may still show dialog depending on compositor
//!
//! Compositor behavior varies:
//! - **GNOME**: Always shows dialog, ignores `interactive=false`
//! - **KDE**: Better spec compliance, respects flags
//! - **Sway/Hyprland**: Varies by portal implementation

use super::{DisplayBackend, DisplayError, DisplayResult, CaptureData, PixelFormat};
use ashpd::desktop::screenshot::Screenshot;
use image::GenericImageView;

pub struct WaylandBackend;

impl WaylandBackend {
    /// Internal implementation using ashpd
    async fn capture_impl(interactive: bool) -> DisplayResult<CaptureData> {
        // Request screenshot through portal
        let request = Screenshot::request()
            .interactive(interactive);

        let proxy = request
            .send()
            .await
            .map_err(|e| DisplayError::PortalError(format!("Failed to request screenshot: {}", e)))?;

        // response() is SYNCHRONOUS, not async
        let response = proxy
            .response()
            .map_err(|e| DisplayError::PortalError(format!("Failed to get screenshot response: {}", e)))?;

        // Get URI from response (uri() returns &Url, not Option)
        let uri = response.uri();

        // Convert URI to file path
        let path = uri
            .to_file_path()
            .map_err(|_| DisplayError::PortalError("Screenshot URI is not a valid file path".to_string()))?;

        // Read the image file
        let image_data: Vec<u8> = tokio::fs::read(&path)
            .await
            .map_err(|e| DisplayError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read screenshot file {}: {}", path.display(), e),
            )))?;

        // Parse the image
        let img = image::load_from_memory(&image_data)
            .map_err(|e| DisplayError::CaptureError(format!("Failed to parse screenshot image: {}", e)))?;

        let (width, height) = img.dimensions();
        let color_type = img.color();
        let pixels = img.into_bytes();
        let pixel_format = pixel_format_from_color_type(color_type)?;

        Ok(CaptureData::new(pixels, width, height, pixel_format))
    }
}

/// Convert image crate color type to our PixelFormat
fn pixel_format_from_color_type(color_type: image::ColorType) -> Result<PixelFormat, DisplayError> {
    match color_type {
        image::ColorType::Rgb8 => Ok(PixelFormat::RGB24),
        image::ColorType::Rgba8 => Ok(PixelFormat::RGBA32),
        // image 0.24.9 doesn't have Bgr8/Bgra8 - need to handle differently
        image::ColorType::L8 => Err(DisplayError::UnsupportedBackend(
            "Grayscale images not supported".to_string()
        )),
        _ => {
            // For unknown formats, assume RGB24 as most portals return this
            Ok(PixelFormat::RGB24)
        }
    }
}

impl DisplayBackend for WaylandBackend {
    fn new() -> DisplayResult<Self> {
        // WaylandBackend is a zero-sized type, no initialization needed
        Ok(WaylandBackend)
    }

    fn capture_screen(&self) -> DisplayResult<CaptureData> {
        // Create a runtime for the async call
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| DisplayError::InitializationError(format!("Failed to create tokio runtime: {}", e)))?;

        // Run the async capture
        rt.block_on(async {
            Self::capture_impl(false).await
        })
    }

    fn capture_area(&self, _x: i32, _y: i32, _width: i32, _height: i32) -> DisplayResult<CaptureData> {
        // NOTE: On Wayland, area capture parameters are ignored
        //
        // The xdg-desktop-portal Screenshot API does NOT support coordinate-based
        // area capture. The user must interactively select the area through the
        // portal dialog.
        //
        // This is a security feature of Wayland - applications cannot capture
        // arbitrary screen regions without user consent.

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| DisplayError::InitializationError(format!("Failed to create tokio runtime: {}", e)))?;

        rt.block_on(async {
            Self::capture_impl(true).await
        })
    }

    fn capture_window(&self, _window_id: u64) -> DisplayResult<CaptureData> {
        // NOTE: On Wayland, window_id is ignored
        //
        // Unlike X11, Wayland does not expose window IDs to applications.
        // The xdg-desktop-portal API requires the user to interactively
        // select which window to capture.
        //
        // This is a security feature of Wayland - applications cannot enumerate
        // or capture windows without user consent.

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| DisplayError::InitializationError(format!("Failed to create tokio runtime: {}", e)))?;

        rt.block_on(async {
            Self::capture_impl(true).await
        })
    }

    fn is_supported() -> bool {
        // Check if we're running on Wayland
        if std::env::var("XDG_SESSION_TYPE")
            .map(|s| s.to_lowercase() != "wayland")
            .unwrap_or(true)
        {
            return false;
        }

        // Try to verify the portal is available
        // This is a basic check - actual availability depends on the compositor
        std::env::var("WAYLAND_DISPLAY").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported() {
        let supported = WaylandBackend::is_supported();
        println!("Wayland backend supported: {}", supported);
    }

    #[test]
    fn test_pixel_format_conversion() {
        // Test RGB format
        let format = pixel_format_from_color_type(image::ColorType::Rgb8).unwrap();
        assert_eq!(format.bits_per_pixel, 24);
        assert_eq!(format.bytes_per_pixel, 3);

        // Test RGBA format
        let format = pixel_format_from_color_type(image::ColorType::Rgba8).unwrap();
        assert_eq!(format.bits_per_pixel, 32);
        assert_eq!(format.bytes_per_pixel, 4);

        // Test unsupported format (grayscale)
        let result = pixel_format_from_color_type(image::ColorType::L8);
        assert!(result.is_err());
    }

    #[test]
    fn test_backend_creation() {
        let backend = WaylandBackend::new();
        assert!(backend.is_ok());
    }
}
