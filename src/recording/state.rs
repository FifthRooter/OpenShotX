use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const RECORDING_STATE_PATH: &str = "/tmp/openshotx-recording.pid";
const SCROLL_STATE_PATH: &str = "/tmp/openshotx-scroll.pid";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingKind {
    Screen,
    Area,
    Scrolling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingFormat {
    Gif,
    Mp4,
    Webm,
    Ogv,
    Png,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingState {
    pub pid: u32,
    pub kind: RecordingKind,
    pub format: RecordingFormat,
    pub started_at: u64,
    pub session_id: String,
    #[serde(default)]
    pub output_path: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Process not found")]
    ProcessNotFound,

    #[error("Failed to send signal: {0}")]
    SignalError(String),
}

pub type StateResult<T> = Result<T, StateError>;

#[derive(Debug)]
pub struct StopResult {
    pub outcome: StopOutcome,
    pub state: Option<RecordingState>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StopOutcome {
    Stopped,
    NothingToStop,
    ForceKillRequired(u32),
}

impl RecordingState {
    pub fn new(kind: RecordingKind, format: RecordingFormat) -> Self {
        let pid = std::process::id();
        let started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let session_id = generate_session_id(pid, started_at);

        Self {
            pid,
            kind,
            format,
            started_at,
            session_id,
            output_path: None,
        }
    }

    pub fn with_output_path(mut self, path: PathBuf) -> Self {
        self.output_path = Some(path);
        self
    }
}

fn generate_session_id(pid: u32, timestamp: u64) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);

    let mut hasher = DefaultHasher::new();
    pid.hash(&mut hasher);
    timestamp.hash(&mut hasher);
    counter.hash(&mut hasher);

    format!("{:x}-{:x}", timestamp, hasher.finish())
}

pub fn state_path() -> PathBuf {
    PathBuf::from(RECORDING_STATE_PATH)
}

pub fn scroll_state_path() -> PathBuf {
    PathBuf::from(SCROLL_STATE_PATH)
}

pub fn write_state(state: &RecordingState, path: &Path) -> StateResult<()> {
    let json = serde_json::to_string_pretty(state)?;
    let tmp_path = path.with_extension("tmp");

    std::fs::write(&tmp_path, json)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn read_state(path: &Path) -> StateResult<Option<RecordingState>> {
    if !path.exists() {
        return Ok(None);
    }

    let json = std::fs::read_to_string(path)?;
    let state: RecordingState = serde_json::from_str(&json)?;
    Ok(Some(state))
}

pub fn clear_state(path: &Path) -> StateResult<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn is_process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    Path::new(&format!("/proc/{}", pid)).exists()
}

pub fn send_signal(pid: u32, signal: i32) -> StateResult<()> {
    let result = unsafe { libc::kill(pid as libc::pid_t, signal) };
    if result == 0 {
        Ok(())
    } else {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ESRCH) {
            Err(StateError::ProcessNotFound)
        } else {
            Err(StateError::SignalError(err.to_string()))
        }
    }
}

pub fn send_sigint(pid: u32) -> StateResult<()> {
    send_signal(pid, libc::SIGINT)
}

pub fn send_sigterm(pid: u32) -> StateResult<()> {
    send_signal(pid, libc::SIGTERM)
}

pub fn stop_recording(path: &Path, timeout_secs: u64) -> StateResult<StopResult> {
    let state = match read_state(path)? {
        Some(s) => s,
        None => {
            return Ok(StopResult {
                outcome: StopOutcome::NothingToStop,
                state: None,
            });
        }
    };

    if !is_process_alive(state.pid) {
        let _ = clear_state(path);
        return Ok(StopResult {
            outcome: StopOutcome::NothingToStop,
            state: Some(state),
        });
    }

    if let Err(e) = send_sigint(state.pid) {
        eprintln!("Warning: Failed to send SIGINT: {}", e);
        let _ = clear_state(path);
        return Ok(StopResult {
            outcome: StopOutcome::ForceKillRequired(state.pid),
            state: Some(state),
        });
    }

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if !is_process_alive(state.pid) {
            let _ = clear_state(path);
            return Ok(StopResult {
                outcome: StopOutcome::Stopped,
                state: Some(state),
            });
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if !is_process_alive(state.pid) {
        let _ = clear_state(path);
        return Ok(StopResult {
            outcome: StopOutcome::Stopped,
            state: Some(state),
        });
    }

    if let Err(e) = send_sigterm(state.pid) {
        eprintln!("Warning: Failed to send SIGTERM: {}", e);
    }

    let sigterm_start = std::time::Instant::now();
    let sigterm_timeout = std::time::Duration::from_secs(2);

    while sigterm_start.elapsed() < sigterm_timeout {
        if !is_process_alive(state.pid) {
            let _ = clear_state(path);
            return Ok(StopResult {
                outcome: StopOutcome::Stopped,
                state: Some(state),
            });
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if is_process_alive(state.pid) {
        return Ok(StopResult {
            outcome: StopOutcome::ForceKillRequired(state.pid),
            state: Some(state),
        });
    }

    let _ = clear_state(path);
    Ok(StopResult {
        outcome: StopOutcome::Stopped,
        state: Some(state),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_path(name: &str) -> PathBuf {
        let mut p = env::temp_dir();
        p.push(format!(
            "openshotx-test-{}-{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        p
    }

    #[test]
    fn test_state_path_constants() {
        assert_eq!(state_path(), PathBuf::from("/tmp/openshotx-recording.pid"));
        assert_eq!(scroll_state_path(), PathBuf::from("/tmp/openshotx-scroll.pid"));
    }

    #[test]
    fn test_recording_state_new() {
        let state = RecordingState::new(RecordingKind::Screen, RecordingFormat::Mp4);
        assert_eq!(state.pid, std::process::id());
        assert_eq!(state.kind, RecordingKind::Screen);
        assert_eq!(state.format, RecordingFormat::Mp4);
        assert!(state.started_at > 0);
        assert!(!state.session_id.is_empty());
    }

    #[test]
    fn test_session_id_uniqueness() {
        let s1 = RecordingState::new(RecordingKind::Screen, RecordingFormat::Mp4);
        let s2 = RecordingState::new(RecordingKind::Area, RecordingFormat::Gif);
        assert_ne!(s1.session_id, s2.session_id);
    }

    #[test]
    fn test_write_and_read_state() {
        let path = temp_path("write-read");
        let state = RecordingState::new(RecordingKind::Area, RecordingFormat::Gif);

        write_state(&state, &path).unwrap();
        let read_back = read_state(&path).unwrap().unwrap();

        assert_eq!(read_back.pid, state.pid);
        assert_eq!(read_back.kind, state.kind);
        assert_eq!(read_back.format, state.format);
        assert_eq!(read_back.session_id, state.session_id);

        clear_state(&path).unwrap();
    }

    #[test]
    fn test_read_state_missing_file() {
        let path = temp_path("missing");
        let result = read_state(&path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_clear_state_missing_file() {
        let path = temp_path("clear-missing");
        assert!(clear_state(&path).is_ok());
    }

    #[test]
    fn test_clear_state_existing_file() {
        let path = temp_path("clear-existing");
        let state = RecordingState::new(RecordingKind::Screen, RecordingFormat::Mp4);
        write_state(&state, &path).unwrap();
        assert!(path.exists());

        clear_state(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_is_process_alive_current() {
        assert!(is_process_alive(std::process::id()));
    }

    #[test]
    fn test_is_process_alive_zero() {
        assert!(!is_process_alive(0));
    }

    #[test]
    fn test_is_process_alive_fake() {
        assert!(!is_process_alive(999_999));
    }

    #[test]
    fn test_stop_recording_nothing_to_stop() {
        let path = temp_path("stop-nothing");
        let result = stop_recording(&path, 1).unwrap();
        assert_eq!(result.outcome, StopOutcome::NothingToStop);
        assert!(result.state.is_none());
    }

    #[test]
    fn test_stop_recording_stale_pid() {
        let path = temp_path("stop-stale");
        let mut state = RecordingState::new(RecordingKind::Screen, RecordingFormat::Mp4);
        state.pid = 999_999;

        write_state(&state, &path).unwrap();
        let result = stop_recording(&path, 1).unwrap();
        assert_eq!(result.outcome, StopOutcome::NothingToStop);
        assert!(!path.exists());
    }

    #[test]
    fn test_stop_recording_returns_state_with_output_path() {
        let path = temp_path("stop-with-path");
        let mut state = RecordingState::new(RecordingKind::Screen, RecordingFormat::Mp4);
        state.pid = 999_998; // dead pid
        state.output_path = Some(PathBuf::from("/tmp/fake-output.mp4"));
        write_state(&state, &path).unwrap();

        let result = stop_recording(&path, 1).unwrap();
        assert_eq!(result.outcome, StopOutcome::NothingToStop);
        assert!(result.state.is_some());
        let returned_state = result.state.unwrap();
        assert_eq!(returned_state.output_path, Some(PathBuf::from("/tmp/fake-output.mp4")));
    }

    #[test]
    fn test_generate_session_id_format() {
        let id = generate_session_id(1234, 5678);
        assert!(id.contains('-'));
        assert!(id.starts_with("162e"));
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert!(u64::from_str_radix(parts[0], 16).is_ok());
        assert!(u64::from_str_radix(parts[1], 16).is_ok());
    }

    #[test]
    fn test_generate_session_id_increments() {
        let id1 = generate_session_id(1234, 5678);
        let id2 = generate_session_id(1234, 5678);
        let id3 = generate_session_id(1234, 5678);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }
}
