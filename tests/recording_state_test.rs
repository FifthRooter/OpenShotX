use openshotx::recording::state::{
    clear_state, is_process_alive, read_state, scroll_state_path, state_path, stop_recording,
    write_state, RecordingFormat, RecordingKind, RecordingState, StopOutcome,
};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pid = std::process::id();
    let mut p = std::env::temp_dir();
    p.push(format!("openshotx-state-test-{}-{}-{}", name, pid, nanos));
    p
}

#[test]
fn test_state_and_scroll_paths_are_different() {
    assert_ne!(state_path(), scroll_state_path());
}

#[test]
fn test_state_path_under_tmp() {
    assert!(state_path().starts_with("/tmp/"));
    assert!(scroll_state_path().starts_with("/tmp/"));
    assert!(state_path().to_string_lossy().contains("openshotx-recording"));
    assert!(scroll_state_path().to_string_lossy().contains("openshotx-scroll"));
}

#[test]
fn test_recording_state_serializes_with_all_kinds() {
    for kind in [
        RecordingKind::Screen,
        RecordingKind::Area,
        RecordingKind::Scrolling,
    ] {
        let state = RecordingState::new(kind, RecordingFormat::Mp4);
        let json = serde_json::to_string(&state).expect("serialize");
        let back: RecordingState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.kind, kind);
    }
}

#[test]
fn test_recording_state_serializes_with_all_formats() {
    for format in [
        RecordingFormat::Gif,
        RecordingFormat::Mp4,
        RecordingFormat::Webm,
        RecordingFormat::Ogv,
        RecordingFormat::Png,
    ] {
        let state = RecordingState::new(RecordingKind::Screen, format);
        let json = serde_json::to_string(&state).expect("serialize");
        let back: RecordingState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.format, format);
    }
}

#[test]
fn test_write_then_read_preserves_all_fields() {
    let path = unique_temp_path("roundtrip");
    let state = RecordingState::new(RecordingKind::Area, RecordingFormat::Gif);

    write_state(&state, &path).expect("write");
    let read_back = read_state(&path).expect("read").expect("present");

    assert_eq!(read_back.pid, state.pid);
    assert_eq!(read_back.kind, state.kind);
    assert_eq!(read_back.format, state.format);
    assert_eq!(read_back.started_at, state.started_at);
    assert_eq!(read_back.session_id, state.session_id);

    clear_state(&path).expect("cleanup");
}

#[test]
fn test_atomic_write_does_not_leave_tmp() {
    let path = unique_temp_path("atomic");
    let state = RecordingState::new(RecordingKind::Screen, RecordingFormat::Mp4);
    write_state(&state, &path).expect("write");

    let tmp = path.with_extension("tmp");
    assert!(!tmp.exists(), "temp file leaked: {:?}", tmp);
    assert!(path.exists());

    clear_state(&path).expect("cleanup");
}

#[test]
fn test_read_missing_returns_none() {
    let path = unique_temp_path("missing");
    let result = read_state(&path).expect("read missing should be ok");
    assert!(result.is_none());
}

#[test]
fn test_read_corrupt_returns_error() {
    let path = unique_temp_path("corrupt");
    std::fs::write(&path, "this is not valid json {{{{").expect("write");
    let result = read_state(&path);
    assert!(result.is_err());
    let _ = clear_state(&path);
}

#[test]
fn test_clear_idempotent() {
    let path = unique_temp_path("clear-idem");
    assert!(clear_state(&path).is_ok());
    assert!(clear_state(&path).is_ok());
    assert!(!path.exists());
}

#[test]
fn test_clear_existing() {
    let path = unique_temp_path("clear-existing");
    let state = RecordingState::new(RecordingKind::Screen, RecordingFormat::Mp4);
    write_state(&state, &path).expect("write");
    assert!(path.exists());
    clear_state(&path).expect("clear");
    assert!(!path.exists());
}

#[test]
fn test_is_process_alive_self() {
    assert!(is_process_alive(std::process::id()));
}

#[test]
fn test_is_process_alive_zero() {
    assert!(!is_process_alive(0));
}

#[test]
fn test_is_process_alive_very_high_pid() {
    assert!(!is_process_alive(9_999_999));
}

#[test]
fn test_stop_recording_nothing_to_stop() {
    let path = unique_temp_path("stop-nothing");
    let result = stop_recording(&path, 1).expect("stop");
    assert_eq!(result.outcome, StopOutcome::NothingToStop);
    assert!(result.state.is_none());
}

#[test]
fn test_stop_recording_clears_stale_state() {
    let path = unique_temp_path("stop-stale");
    let mut state = RecordingState::new(RecordingKind::Screen, RecordingFormat::Mp4);
    state.pid = 9_999_998;
    write_state(&state, &path).expect("write");

    let result = stop_recording(&path, 1).expect("stop");
    assert_eq!(result.outcome, StopOutcome::NothingToStop);
    assert!(!path.exists(), "stale state file should be cleaned up");
}

#[test]
fn test_stop_recording_returns_output_path() {
    use std::path::PathBuf;
    let path = unique_temp_path("stop-with-output");
    let mut state = RecordingState::new(RecordingKind::Area, RecordingFormat::Gif);
    state.pid = 9_999_997;
    state.output_path = Some(PathBuf::from("/tmp/some.gif"));
    write_state(&state, &path).expect("write");

    let result = stop_recording(&path, 1).expect("stop");
    assert_eq!(result.outcome, StopOutcome::NothingToStop);
    let returned = result.state.expect("state should be present");
    assert_eq!(returned.output_path, Some(PathBuf::from("/tmp/some.gif")));
}

#[test]
fn test_session_ids_are_unique() {
    let s1 = RecordingState::new(RecordingKind::Screen, RecordingFormat::Mp4);
    let s2 = RecordingState::new(RecordingKind::Screen, RecordingFormat::Mp4);
    assert_ne!(s1.session_id, s2.session_id);
}
