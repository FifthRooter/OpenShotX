# Progress Tracker

## Phase 1: Complete `capture_screen`

- [x] **Add Image Parsing:**
  - [x] Add `image` crate to `Cargo.toml`.
  - [x] Implement image parsing in `capture_screen`.
  - [x] Map `image` color type to `PixelFormat`.
- [x] **Testing for `capture_screen`:**
  - [x] Create `tests/wayland_backend_test.rs`.
  - [x] Add integration test for `capture_screen`.
  - [x] Add unit test for portal error handling.
  - [x] Add unit test for invalid URI handling.

## Phase 2: Implement `capture_area`

- [x] **Implementation:**
  - [x] Implement `capture_area` using the `interactive` portal option.
- [x] **Testing for `capture_area`:**
  - [x] Document manual test case for `capture_area`.
  - [x] Add unit test for `interactive` option.

## Phase 3: Implement `capture_window`

- [x] **Implementation:**
  - [x] Implement `capture_window` using the `interactive` portal option.
- [x] **Testing for `capture_window`:**
  - [x] Document manual test case for `capture_window`.
  - [x] Add unit test for `capture_window` options.

## Phase 4: Refactoring and Library Integration

- [x] **Use ashpd Library:**
  - [x] Replace manual D-Bus implementation with ashpd
  - [x] Proper async/sync handling with zbus
  - [x] Remove unused dbus_proxy.rs
- [x] **Error Handling:**
  - [x] Ensure consistent error wrapping.
  - [x] Handle user cancellation gracefully.
  - [x] Fix integer overflow bug in test validation.

## Summary

All phases completed. Wayland backend is fully functional using ashpd library for xdg-desktop-portal integration.

**Note:** On Wayland, `capture_area()` and `capture_window()` use interactive mode where the user selects through the portal dialog. The coordinate/window_id parameters are ignored due to Wayland security limitations.
