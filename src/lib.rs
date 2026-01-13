pub mod backend;
pub mod capture;
pub mod overlay;
pub mod utils;

// Re-export commonly used types
pub use backend::{DisplayBackend, DisplayError, DisplayResult, CaptureData, PixelFormat};
pub use capture::{save_capture, quick_save, SaveConfig, ImageFormat, SaveError, SaveResult};
pub use overlay::{select_area, AreaSelector, SelectionArea, SelectionError, SelectionResult};
