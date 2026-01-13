//! GTK4 overlay for interactive area selection
//!
//! This module provides a full-screen transparent window that allows users
//! to select a screen area using mouse drag. Only used for X11 backend.

use gtk4::{
    gdk,
    glib::{self, clone},
    prelude::*,
    Application, ApplicationWindow, EventControllerKey, GestureDrag,
};
use gtk4::gdk::Key;
use std::sync::{Arc, Mutex};

/// Selected area coordinates
#[derive(Debug, Clone, Copy)]
pub struct SelectionArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl SelectionArea {
    /// Normalize the selection (handle negative width/height from dragging)
    pub fn normalize(mut self) -> Self {
        if self.width < 0 {
            self.x += self.width;
            self.width = self.width.abs();
        }
        if self.height < 0 {
            self.y += self.height;
            self.height = self.height.abs();
        }
        self
    }

    /// Check if the selection is valid (has positive area)
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// Result of area selection
pub type SelectionResult = Result<Option<SelectionArea>, SelectionError>;

#[derive(Debug, thiserror::Error)]
pub enum SelectionError {
    #[error("GTK initialization failed: {0}")]
    InitError(String),

    #[error("Selection was cancelled by user")]
    Cancelled,
}

/// State for the area selector overlay
struct SelectorState {
    start_x: f64,
    start_y: f64,
    current_x: f64,
    current_y: f64,
    is_dragging: bool,
    cancelled: bool,
    completed: bool,
}

impl Default for SelectorState {
    fn default() -> Self {
        Self {
            start_x: 0.0,
            start_y: 0.0,
            current_x: 0.0,
            current_y: 0.0,
            is_dragging: false,
            cancelled: false,
            completed: false,
        }
    }
}

/// GTK4 overlay window for interactive area selection
pub struct AreaSelector {
    state: Arc<Mutex<SelectorState>>,
}

impl AreaSelector {
    /// Create a new area selector
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SelectorState::default())),
        }
    }

    /// Run the area selection dialog
    ///
    /// Returns `Ok(Some(area))` if user selected an area
    /// Returns `Ok(None)` if user cancelled (ESC)
    /// Returns `Err` if initialization failed
    pub fn run(&self) -> SelectionResult {
        let state = self.state.clone();
        let (result_tx, result_rx) = std::sync::mpsc::channel();

        // Create application
        let app = Application::builder()
            .application_id("com.cleanshitx.screenshot")
            .build();

        // Clone state for the activate handler
        let state_activate = state.clone();
        app.connect_activate(move |application| {
            setup_window(application, state_activate.clone(), result_tx.clone());
        });

        // Run the application
        let _ = app.run_with_args::<String>(&[]);

        // Get the result
        match result_rx.recv() {
            Ok(Ok(area)) => Ok(area),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(SelectionError::InitError("No result received".into())),
        }
    }
}

/// Setup the overlay window (standalone function to avoid lifetime issues)
fn setup_window(
    app: &Application,
    state: Arc<Mutex<SelectorState>>,
    result_tx: std::sync::mpsc::Sender<SelectionResult>,
) {
    // Get the display and monitor for screen dimensions
    let display = match gdk::Display::default() {
        Some(d) => d,
        None => {
            let _ = result_tx.send(Err(SelectionError::InitError("No display found".into())));
            return;
        }
    };

    // Get screen dimensions from the first monitor
    let monitor = {
        let monitors = display.monitors();
        let n = monitors.n_items();
        if n == 0 {
            let _ = result_tx.send(Err(SelectionError::InitError("No monitor found".into())));
            return;
        }
        // Get the first monitor from the list model
        match monitors.item(0) {
            Some(obj) => match obj.downcast::<gdk::Monitor>() {
                Ok(m) => m,
                Err(_) => {
                    let _ = result_tx.send(Err(SelectionError::InitError("Failed to get monitor".into())));
                    return;
                }
            },
            None => {
                let _ = result_tx.send(Err(SelectionError::InitError("No monitor at index 0".into())));
                return;
            }
        }
    };

    let geometry = monitor.geometry();
    let screen_width = geometry.width();
    let screen_height = geometry.height();

    // Create the window
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(screen_width)
        .default_height(screen_height)
        .decorated(false)
        .resizable(false)
        .css_classes(["overlay", "transparent"])
        .build();

    // Set window to be fullscreen
    window.set_fullscreened(true);

    // Get the surface for cursor control
    let surface = window.surface();

    // Set cursor to crosshair when hovering over the window
    if let Some(ref surface) = surface {
        let cursor = gdk::Cursor::from_name("crosshair", None);
        surface.set_cursor(cursor.as_ref());
    }

    // Create a drawing area for rendering the selection
    let drawing_area = gtk4::DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .build();

    let state_draw = state.clone();
    drawing_area.set_draw_func(move |_, context, width, height| {
        draw_overlay(context, width, height, &state_draw);
    });

    // Set the drawing area as the child
    window.set_child(Some(&drawing_area));

    // Setup drag gesture for area selection
    let drag_gesture = GestureDrag::builder()
        .propagation_phase(gtk4::PropagationPhase::Capture)
        .build();

    let state_drag = state.clone();
    let window_weak = window.downgrade();
    let drawing_area_weak = drawing_area.downgrade();
    let result_tx_drag = result_tx.clone();

    // Note: connect_drag_begin takes 3 params (gesture, x, y)
    drag_gesture.connect_drag_begin(clone!(
        #[strong]
        state_drag,
        #[strong]
        drawing_area_weak,
        move |_gesture, x, y| {
            let mut st = state_drag.lock().unwrap();
            st.start_x = x;
            st.start_y = y;
            st.current_x = x;
            st.current_y = y;
            st.is_dragging = true;
            drop(st);

            if let Some(drawing_area) = drawing_area_weak.upgrade() {
                drawing_area.queue_draw();
            }
        }
    ));

    drag_gesture.connect_drag_update(clone!(
        #[strong]
        state_drag,
        #[strong]
        drawing_area_weak,
        move |_gesture, x, y| {
            let mut st = state_drag.lock().unwrap();
            st.current_x = st.start_x + x;
            st.current_y = st.start_y + y;
            drop(st);

            if let Some(drawing_area) = drawing_area_weak.upgrade() {
                drawing_area.queue_draw();
            }
        }
    ));

    drag_gesture.connect_drag_end(clone!(
        #[strong]
        state_drag,
        #[strong]
        window_weak,
        #[strong]
        result_tx_drag,
        move |_gesture, x, y| {
            let mut st = state_drag.lock().unwrap();
            st.current_x = st.start_x + x;
            st.current_y = st.start_y + y;
            st.is_dragging = false;
            st.completed = true;

            // Calculate the selection area
            let area = SelectionArea {
                x: st.start_x as i32,
                y: st.start_y as i32,
                width: (st.current_x - st.start_x) as i32,
                height: (st.current_y - st.start_y) as i32,
            }
            .normalize();

            drop(st);

            // Send the result
            let result = if area.is_valid() {
                Ok(Some(area))
            } else {
                Ok(None) // Invalid area treated as cancel
            };

            let _ = result_tx_drag.send(result);

            // Close the window
            if let Some(window) = window_weak.upgrade() {
                window.close();
            }
        }
    ));

    drawing_area.add_controller(drag_gesture);

    // Setup keyboard controller for ESC key
    let key_controller = EventControllerKey::builder()
        .propagation_phase(gtk4::PropagationPhase::Capture)
        .build();

    let state_key = state.clone();
    let window_weak_esc = window.downgrade();
    let result_tx_esc = result_tx.clone();

    key_controller.connect_key_pressed(clone!(
        #[strong]
        state_key,
        move |_, key, _, _| {
            if key == Key::Escape {
                let mut st = state_key.lock().unwrap();
                st.cancelled = true;
                drop(st);

                let _ = result_tx_esc.send(Ok(None));

                if let Some(window) = window_weak_esc.upgrade() {
                    window.close();
                }

                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        }
    ));

    drawing_area.add_controller(key_controller);

    // Show the window
    window.present();
}

/// Draw the overlay (darken background + selection rectangle)
fn draw_overlay(
    context: &gtk4::cairo::Context,
    _width: i32,
    _height: i32,
    state: &Arc<Mutex<SelectorState>>,
) {
    let st = state.lock().unwrap();

    // Get screen dimensions
    let display = match gdk::Display::default() {
        Some(d) => d,
        None => return,
    };

    let monitor = {
        let monitors = display.monitors();
        let n = monitors.n_items();
        if n == 0 {
            return;
        }
        // Get the first monitor from the list model
        match monitors.item(0) {
            Some(obj) => match obj.downcast::<gdk::Monitor>() {
                Ok(m) => m,
                Err(_) => return,
            },
            None => return,
        }
    };

    let geometry = monitor.geometry();
    let screen_width = geometry.width() as f64;
    let screen_height = geometry.height() as f64;

    // Clear to transparent
    context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    let _ = context.paint();

    if st.is_dragging || st.completed {
        // Calculate selection rectangle
        let x = st.start_x.min(st.current_x);
        let y = st.start_y.min(st.current_y);
        let width = (st.current_x - st.start_x).abs();
        let height = (st.current_y - st.start_y).abs();

        // Darken the area outside the selection
        context.set_source_rgba(0.0, 0.0, 0.0, 0.5);

        // Top rectangle
        context.rectangle(0.0, 0.0, screen_width, y);
        let _ = context.fill();

        // Bottom rectangle
        context.rectangle(0.0, y + height, screen_width, screen_height - y - height);
        let _ = context.fill();

        // Left rectangle
        context.rectangle(0.0, y, x, height);
        let _ = context.fill();

        // Right rectangle
        context.rectangle(x + width, y, screen_width - x - width, height);
        let _ = context.fill();

        // Draw selection border (white)
        context.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        context.set_line_width(2.0);
        context.rectangle(x, y, width, height);
        let _ = context.stroke();

        // Draw dimensions text
        let text = format!("{}×{}", width as i32, height as i32);

        // Set up text rendering
        context.set_font_size(14.0);
        context.set_source_rgba(1.0, 1.0, 1.0, 1.0);

        // Get text extents (call methods with parentheses)
        let extents = context.text_extents(&text).unwrap();

        // Draw text background (semi-transparent black)
        let padding = 8.0;
        let text_x = x + width / 2.0 - extents.width() / 2.0 - extents.x_bearing();
        let text_y = y - 10.0;

        context.set_source_rgba(0.0, 0.0, 0.0, 0.7);
        context.rectangle(
            text_x - padding,
            text_y + extents.y_bearing() - padding,
            extents.width() + padding * 2.0,
            extents.height() + padding * 2.0,
        );
        let _ = context.fill();

        // Draw the text
        context.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        context.move_to(text_x, text_y);
        let _ = context.show_text(&text);
    } else {
        // Not dragging - darken entire screen slightly
        context.set_source_rgba(0.0, 0.0, 0.0, 0.3);
        let _ = context.paint();
    }
}

impl Default for AreaSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to run area selection
pub fn select_area() -> SelectionResult {
    let selector = AreaSelector::new();
    selector.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_normalize() {
        // Normal case (no normalization needed)
        let area = SelectionArea { x: 100, y: 100, width: 200, height: 150 };
        let normalized = area.normalize();
        assert_eq!(normalized.x, 100);
        assert_eq!(normalized.y, 100);
        assert_eq!(normalized.width, 200);
        assert_eq!(normalized.height, 150);

        // Negative width (dragged left)
        let area = SelectionArea { x: 300, y: 100, width: -200, height: 150 };
        let normalized = area.normalize();
        assert_eq!(normalized.x, 100);
        assert_eq!(normalized.y, 100);
        assert_eq!(normalized.width, 200);
        assert_eq!(normalized.height, 150);

        // Negative height (dragged up)
        let area = SelectionArea { x: 100, y: 250, width: 200, height: -150 };
        let normalized = area.normalize();
        assert_eq!(normalized.x, 100);
        assert_eq!(normalized.y, 100);
        assert_eq!(normalized.width, 200);
        assert_eq!(normalized.height, 150);

        // Both negative (dragged up-left)
        let area = SelectionArea { x: 300, y: 250, width: -200, height: -150 };
        let normalized = area.normalize();
        assert_eq!(normalized.x, 100);
        assert_eq!(normalized.y, 100);
        assert_eq!(normalized.width, 200);
        assert_eq!(normalized.height, 150);
    }

    #[test]
    fn test_selection_is_valid() {
        // Valid selection
        let area = SelectionArea { x: 100, y: 100, width: 200, height: 150 };
        assert!(area.is_valid());

        // Zero width
        let area = SelectionArea { x: 100, y: 100, width: 0, height: 150 };
        assert!(!area.is_valid());

        // Zero height
        let area = SelectionArea { x: 100, y: 100, width: 200, height: 0 };
        assert!(!area.is_valid());

        // Negative (before normalization)
        let area = SelectionArea { x: 100, y: 100, width: -200, height: 150 };
        assert!(!area.is_valid());
    }
}
