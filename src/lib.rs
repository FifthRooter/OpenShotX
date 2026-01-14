pub mod backend;
pub mod capture;
pub mod overlay;
pub mod ocr;
pub mod utils;

// Re-export commonly used types
pub use backend::{DisplayBackend, DisplayError, DisplayResult, CaptureData, PixelFormat};
pub use capture::{save_capture, quick_save, SaveConfig, ImageFormat, SaveError, SaveResult};
pub use overlay::{select_area, AreaSelector, SelectionArea, SelectionError, SelectionResult};
pub use ocr::{OcrConfig, OcrOutput, OcrError, OcrResult, extract_text, extract_text_from_path, copy_to_clipboard};
