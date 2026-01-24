use std::path::PathBuf;
use std::collections::HashMap;
use thiserror::Error;
use zbus::zvariant::OwnedValue;
use gst::prelude::*;
use gstreamer as gst;

#[derive(Debug, Error)]
pub enum RecordError {
    #[error("GStreamer initialization failed: {0}")]
    InitError(String),
    
    #[error("GStreamer error: {0}")]
    GStreamerError(String),
    
    #[error("Wayland portal error: {0}")]
    PortalError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Unsupported backend: {0}")]
    UnsupportedBackend(String),

    #[error("Cancelled by user")]
    Cancelled,
    
    #[error("No suitable video encoder found. Please install gst-plugins-good/ugly/bad.")]
    NoEncoderFound,
}

pub type RecordResult<T> = Result<T, RecordError>;

#[derive(Debug, Clone)]
pub struct RecordingConfig {
    pub output_path: PathBuf,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub x: Option<i32>,
    pub y: Option<i32>,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        let mut path = dirs::video_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("output.mp4");
        Self {
            output_path: path,
            width: None,
            height: None,
            x: None,
            y: None,
        }
    }
}

struct EncoderProfile {
    name: &'static str,
    encoder: &'static str,
    props: &'static str,
    muxer: &'static str,
    extension: &'static str,
}

// Priority list of encoders
const PROFILES: &[EncoderProfile] = &[
    // VP8 (WebM) - Prioritized fallback over H.264 if missing, and better than Theora
    EncoderProfile {
        name: "VP8", 
        encoder: "vp8enc", 
        props: "deadline=1", 
        muxer: "webmmux", 
        extension: "webm"
    }, 
    // VP9 (WebM)
    EncoderProfile {
        name: "VP9", 
        encoder: "vp9enc", 
        props: "deadline=1", 
        muxer: "webmmux", 
        extension: "webm"
    },
    // Standard H.264
    EncoderProfile {
        name: "H.264 (x264)", 
        encoder: "x264enc", 
        props: "speed-preset=ultrafast tune=zerolatency", 
        muxer: "mp4mux", 
        extension: "mp4"
    },
    // Cisco OpenH264
    EncoderProfile {
        name: "H.264 (OpenH264)", 
        encoder: "openh264enc", 
        props: "", 
        muxer: "mp4mux", 
        extension: "mp4"
    },
    // Theora (Ogg) - Last resort
    EncoderProfile {
        name: "Theora", 
        encoder: "theoraenc", 
        props: "", 
        muxer: "oggmux", 
        extension: "ogv"
    },
];

/// Start a recording session
pub async fn start_recording(config: RecordingConfig) -> RecordResult<()> {
    // 1. Initialize GStreamer
    gst::init().map_err(|e| RecordError::InitError(e.to_string()))?;

    // 2. Select Encoder Profile
    let (profile, final_path) = select_encoder(&config.output_path)?;
    println!("Using Encoder: {} ({})", profile.name, profile.encoder);
    
    if final_path != config.output_path {
        println!("Note: Output filename changed to match format: {:?}", final_path);
    }

    // 3. Build pipeline description
    let pipeline_str = build_pipeline(&config, profile, &final_path).await?;
    println!("Starting recording to: {:?}", final_path);

    // 4. Create pipeline
    let pipeline = gst::parse::launch(&pipeline_str)
        .map_err(|e| RecordError::GStreamerError(format!("Failed to parse pipeline: {}", e)))
        ?.downcast::<gst::Pipeline>()
        .map_err(|_| RecordError::GStreamerError("Cast to Pipeline failed".into()))?;

    // 5. Start playing
    if let Err(err) = pipeline.set_state(gst::State::Playing) {
        eprintln!("Failed to set pipeline to Playing: {}", err);
        if let Some(bus) = pipeline.bus() {
            while let Some(msg) = bus.pop() {
                if let gst::MessageView::Error(err) = msg.view() {
                    eprintln!("Detailed Error from {}: {}", 
                        err.src().map(|s| s.name()).unwrap_or("unknown".into()), 
                        err.error()
                    );
                    if let Some(debug) = err.debug() {
                        eprintln!("Debug Info: {}", debug);
                    }
                }
            }
        }
        let _ = pipeline.set_state(gst::State::Null);
        return Err(RecordError::GStreamerError(format!("State change failed: {}", err)));
    }

    // 6. Watch for messages and Ctrl+C
    let bus = pipeline.bus().ok_or_else(|| RecordError::GStreamerError("Pipeline has no bus".into()))?;
    
    println!("Recording... Press Ctrl+C to stop.");

    // Handle Ctrl+C
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);
    
    // Phase 1: Recording until Ctrl+C or Error
    let mut stopping = false;
    loop {
        tokio::select! {
            _ = &mut ctrl_c => {
                println!("\nStopping recording... Finalizing file...");
                pipeline.send_event(gst::event::Eos::new());
                stopping = true;
                break;
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                // Poll bus
                for msg in bus.iter_timed(gst::ClockTime::ZERO) {
                    use gst::MessageView;
                    match msg.view() {
                        MessageView::Eos(..) => {
                            println!("End of stream reached (unexpected).");
                            stopping = true;
                            break;
                        }
                        MessageView::Error(err) => {
                            eprintln!("Error from element {:?}: {}", err.src().map(|s| s.name()), err.error());
                            let _ = pipeline.set_state(gst::State::Null);
                            return Err(RecordError::GStreamerError(err.error().to_string()));
                        }
                        _ => (),
                    }
                }
                if stopping { break; }
            }
        }
    }

    // Phase 2: Wait for EOS if we initiated stop
    if stopping {
        let start_wait = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(5); // 5s timeout for finalization
        
        loop {
            if start_wait.elapsed() > timeout {
                eprintln!("Timeout waiting for EOS. Forcing stop.");
                break;
            }

            // Check bus
            let mut eos_received = false;
            for msg in bus.iter_timed(gst::ClockTime::from_mseconds(100)) {
                use gst::MessageView;
                match msg.view() {
                    MessageView::Eos(..) => {
                        println!("File finalized successfully.");
                        eos_received = true;
                        break;
                    }
                    MessageView::Error(err) => {
                        eprintln!("Error during finalization: {}", err.error());
                        eos_received = true; // Stop waiting
                        break;
                    }
                    _ => (),
                }
            }
            if eos_received { break; }
        }
    }

    // 7. Cleanup
    pipeline.set_state(gst::State::Null)
        .map_err(|e| RecordError::GStreamerError(format!("Failed to set state to Null: {}", e)))?;

    println!("Recording saved to {:?}", final_path);
    if let Ok(metadata) = std::fs::metadata(&final_path) {
        println!("File size: {:.2} MB", metadata.len() as f64 / 1024.0 / 1024.0);
    }
    Ok(())
}

fn select_encoder(requested_path: &PathBuf) -> RecordResult<(&'static EncoderProfile, PathBuf)> {
    // Check for x264enc first to warn user if missing
    if gst::ElementFactory::find("x264enc").is_none() {
        println!("\n\x1b[33mWARNING: H.264 encoder (x264enc) not found!\x1b[0m");
        println!("Falling back to inferior encoders (Theora/VP8). For high-quality MP4 recording, please install:");
        println!("  Ubuntu/Debian: \x1b[1msudo apt install gstreamer1.0-plugins-ugly\x1b[0m");
        println!("  Arch:          \x1b[1msudo pacman -S gst-plugins-ugly\x1b[0m");
        println!("  Fedora:        \x1b[1msudo dnf install gstreamer1-plugins-ugly-free\x1b[0m\n");
    }

    if let Some(ext) = requested_path.extension().and_then(|s| s.to_str()) {
        for profile in PROFILES {
            if profile.extension == ext {
                if gst::ElementFactory::find(profile.encoder).is_some() && 
                   gst::ElementFactory::find(profile.muxer).is_some() {
                    return Ok((profile, requested_path.clone()));
                }
            }
        }
        println!("Warning: Requested format '{}' not supported or encoder missing.", ext);
    }

    for profile in PROFILES {
        if gst::ElementFactory::find(profile.encoder).is_some() && 
           gst::ElementFactory::find(profile.muxer).is_some() {
            let mut new_path = requested_path.clone();
            new_path.set_extension(profile.extension);
            return Ok((profile, new_path));
        }
    }

    Err(RecordError::NoEncoderFound)
}

async fn build_pipeline(config: &RecordingConfig, profile: &EncoderProfile, output_path: &PathBuf) -> RecordResult<String> {
    let output_str = output_path.to_string_lossy();
    
    // Get video source
    let video_source = if std::env::var("WAYLAND_DISPLAY").is_ok() {
        get_wayland_source().await?
    } else {
        get_x11_source(config)?
    };

    // Note: Reverted audio muxing for now as it caused negotiation issues with pipewiresrc.
    Ok(format!(
        "{} ! videoconvert ! videorate ! queue ! {} {} ! {} ! filesink location=\"{}\"",
        video_source,
        profile.encoder, profile.props, profile.muxer, output_str
    ))
}

async fn get_wayland_source() -> RecordResult<String> {
    use ashpd::desktop::screencast::Screencast;
    use zbus::zvariant::Value;

    println!("Requesting Wayland ScreenCast session...");
    
    let proxy = Screencast::new().await
        .map_err(|e| RecordError::PortalError(e.to_string()))?;

    let session = proxy.create_session().await
        .map_err(|e| RecordError::PortalError(e.to_string()))?;

    let connection = proxy.connection();

    // 1. Select Sources
    proxy.select_sources(
        &session,
        ashpd::desktop::screencast::CursorMode::Embedded,
        ashpd::desktop::screencast::SourceType::Monitor | ashpd::desktop::screencast::SourceType::Window,
        false, // multiple
        None,
        ashpd::desktop::PersistMode::DoNot,
    ).await.map_err(|e| RecordError::PortalError(e.to_string()))?;

    println!("Please select a screen or window to record...");
    
    // 2. Start
    let msg = connection.call_method(
        Some("org.freedesktop.portal.Desktop"),
        "/org/freedesktop/portal/desktop",
        Some("org.freedesktop.portal.ScreenCast"),
        "Start",
        &(&session, "", HashMap::<String, Value>::new()),
    ).await.map_err(|e| RecordError::PortalError(format!("Start call failed: {}", e)))?;
    
    let request_path: zbus::zvariant::OwnedObjectPath = msg.body().deserialize()
        .map_err(|e| RecordError::PortalError(format!("Failed to parse Start response: {}", e)))?;
        
    let results: HashMap<String, OwnedValue> = wait_for_response(connection, &request_path).await?;

    let streams_value = results.get("streams")
        .ok_or_else(|| RecordError::PortalError("No streams in portal response".into()))?;

    let streams: Vec<(u32, HashMap<String, OwnedValue>)> = streams_value.try_clone().unwrap()
        .try_into()
        .map_err(|e| RecordError::PortalError(format!("Invalid streams format: {}", e)))?;

    let stream = streams.first()
        .ok_or_else(|| RecordError::PortalError("No streams returned".into()))?;

    let node_id = stream.0;
    println!("Got PipeWire Node ID: {}", node_id);

    Ok(format!("pipewiresrc path={} do-timestamp=true", node_id))
}

async fn wait_for_response(
    connection: &zbus::Connection, 
    path: &zbus::zvariant::ObjectPath<'_>
) -> RecordResult<HashMap<String, OwnedValue>> {
    use futures_util::StreamExt;
    
    let match_rule = format!(
        "type='signal',interface='org.freedesktop.portal.Request',member='Response',path='{}'",
        path
    );
    
    let rule: zbus::MatchRule = match_rule.as_str().try_into()
        .map_err(|e| RecordError::PortalError(format!("Invalid match rule: {}", e)))?;

    let mut stream = zbus::MessageStream::for_match_rule(
        rule,
        connection,
        Some(1),
    ).await.map_err(|e| RecordError::PortalError(format!("Failed to create message stream: {}", e)))?;

    let message = stream.next().await
        .ok_or_else(|| RecordError::PortalError("No response from portal".into()))?
        .map_err(|e| RecordError::PortalError(format!("Signal error: {}", e)))?;

    // Response signal signature: (ua{sv})
    let (status, results): (u32, HashMap<String, OwnedValue>) = message.body().deserialize()
        .map_err(|e| RecordError::PortalError(format!("Failed to deserialize portal response: {}", e)))?;

    if status != 0 {
        return Err(RecordError::Cancelled);
    }
    
    Ok(results)
}

fn get_x11_source(config: &RecordingConfig) -> RecordResult<String> {
    let mut source = String::from("ximagesrc show-pointer=true use-damage=false");
    
    if let (Some(x), Some(y), Some(w), Some(h)) = (config.x, config.y, config.width, config.height) {
        source.push_str(&format!(" startx={} starty={} endx={} endy={}", x, y, x + w as i32 - 1, y + h as i32 - 1));
    }

    Ok(source)
}
