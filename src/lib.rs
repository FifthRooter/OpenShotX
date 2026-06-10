pub mod backend;
pub mod capture;
pub mod overlay;
pub mod ocr;
pub mod recording;
pub mod scrolling;
pub mod utils;

// Re-export commonly used types
pub use backend::{DisplayBackend, DisplayError, DisplayResult, CaptureData, PixelFormat};
pub use capture::{save_capture, quick_save, SaveConfig, ImageFormat, SaveError, SaveResult, copy_image_to_clipboard};
pub use overlay::{select_area, AreaSelector, SelectionArea, SelectionError, SelectionResult};
pub use ocr::{OcrConfig, OcrOutput, OcrError, OcrResult, extract_text, extract_text_from_path, copy_to_clipboard};
pub use recording::{RecordingConfig, start_recording, RecordError, RecordResult};
pub use recording::state::{
    clear_state, is_process_alive, read_state, scroll_state_path, send_sigint,
    send_sigterm, state_path, stop_recording, write_state, RecordingFormat,
    RecordingKind, RecordingState, StateError, StateResult, StopOutcome,
};
pub use scrolling::{ScrollCaptureConfig, ScrollCaptureResult, ScrollError, ScrollResult, capture_scrolling_pw, save_scrolling_capture};
