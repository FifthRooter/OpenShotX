use notify_rust::Notification;

const APP_ID: &str = "openshotx";

pub fn recording_started(format: &str) {
    let body = match format {
        "gif" => "Recording GIF — toggle to stop".to_string(),
        _ => "Recording — toggle to stop".to_string(),
    };
    spawn("openshotx", &body, "media-record");
}

pub fn recording_stopped(path: Option<&std::path::Path>) {
    let body = match path {
        Some(p) => format!("Saved: {}", p.display()),
        None => "Recording stopped".to_string(),
    };
    spawn("openshotx", &body, "media-playback-stop");
}

pub fn recording_failed(reason: &str) {
    spawn("openshotx: failed", reason, "dialog-error");
}

pub fn scroll_started() {
    spawn("openshotx scroll", "Scroll capture running — toggle to stop", "media-record");
}

pub fn scroll_stopped(path: Option<&std::path::Path>) {
    let body = match path {
        Some(p) => format!("Saved: {}", p.display()),
        None => "Scroll capture stopped".to_string(),
    };
    spawn("openshotx scroll", &body, "media-playback-stop");
}

pub fn scroll_failed(reason: &str) {
    spawn("openshotx scroll: failed", reason, "dialog-error");
}

pub fn capture_saved(kind: &str, path: &std::path::Path) {
    let body = format!("{} saved: {}", kind, path.display());
    spawn("openshotx", &body, "camera-photo");
}

fn spawn(summary: &str, body: &str, icon: &str) {
    let summary = summary.to_string();
    let body = body.to_string();
    let icon = icon.to_string();
    // Run on a separate thread so notify-rust's internal block_on()
    // does not collide with the tokio runtime in the main thread.
    std::thread::spawn(move || {
        send(&summary, &body, &icon);
    });
}

fn send(summary: &str, body: &str, icon: &str) {
    let result = Notification::new()
        .summary(summary)
        .body(body)
        .icon(icon)
        .appname(APP_ID)
        .timeout(notify_rust::Timeout::Milliseconds(3000))
        .show();

    if let Err(e) = result {
        eprintln!("Notification failed: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_recording_started_gif_does_not_panic() {
        recording_started("gif");
    }

    #[test]
    fn test_recording_started_video_does_not_panic() {
        recording_started("video");
    }

    #[test]
    fn test_recording_stopped_with_path_does_not_panic() {
        recording_stopped(Some(&PathBuf::from("/tmp/test.gif")));
    }

    #[test]
    fn test_recording_stopped_without_path_does_not_panic() {
        recording_stopped(None);
    }

    #[test]
    fn test_recording_failed_does_not_panic() {
        recording_failed("test error");
    }

    #[test]
    fn test_scroll_lifecycle_does_not_panic() {
        scroll_started();
        scroll_stopped(Some(&PathBuf::from("/tmp/scroll.png")));
        scroll_stopped(None);
        scroll_failed("scroll error");
    }

    #[test]
    fn test_capture_saved_does_not_panic() {
        capture_saved("screenshot", &PathBuf::from("/tmp/cap.png"));
    }
}
