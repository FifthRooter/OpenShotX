#![cfg(test)]
use cleanshitx::backend::wayland::WaylandBackend;
use cleanshitx::backend::{DisplayBackend, DisplayError};
use mockall::predicate::*;
use mockall::*;
use zbus::zvariant::{Value, ObjectPath};
use std::collections::HashMap;

#[test]
fn test_capture_screen_wayland_mocked_success() {
    // TODO: Implement mock test
}

#[test]
fn test_capture_screen_wayland_mocked_failure() {
    // TODO: Implement mock test
}
