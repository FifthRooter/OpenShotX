pub mod backend;
pub mod capture;
pub mod utils;

// Re-export commonly used types
pub use backend::{DisplayBackend, DisplayError, DisplayResult, CaptureData, PixelFormat};
