//! OpenShotX CLI - Screenshot tool for Linux
//!
//! Usage:
//!   cargo run -- capture screen
//!   cargo run -- capture area
//!   cargo run -- capture window
//!   cargo run -- ocr <image>

use cleanshitx::{
    backend::{X11Backend, WaylandBackend, CaptureData, DisplayBackend},
    capture::{save_capture, SaveConfig, ImageFormat},
    select_area,
    ocr::{extract_text_from_path, OcrConfig},
};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "capture" => {
            if args.len() < 3 {
                eprintln!("Error: missing capture type");
                print_usage();
                std::process::exit(1);
            }
            run_capture(&args);
        }
        "ocr" => {
            if args.len() < 3 {
                eprintln!("Error: missing image path");
                print_usage();
                std::process::exit(1);
            }
            run_ocr(&args);
        }
        "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Error: unknown command '{}'", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("OpenShotX - Screenshot tool for Linux");
    println!();
    println!("Usage: cargo run -- <command> [options]");
    println!();
    println!("Commands:");
    println!("  capture <type>    Capture a screenshot");
    println!("  ocr <image>       Extract text from an image");
    println!();
    println!("Capture types:");
    println!("  screen            Capture the entire screen");
    println!("  area              Capture a selected area (Wayland: interactive)");
    println!("  window            Capture a specific window (Wayland: interactive)");
    println!();
    println!("Capture options:");
    println!("  --output <path>   Save to specific path (default: ~/Pictures)");
    println!("  --no-cursor       Don't include cursor in screenshot");
    println!("  --jpeg [quality]  Save as JPEG with quality 1-100 (default: PNG)");
    println!("  --prefix <text>   Prefix for filename (default: 'screenshot')");
    println!("  --ocr             Run OCR on captured image and copy to clipboard");
    println!();
    println!("OCR options:");
    println!("  --lang <code>     Language(s) for OCR (default: eng)");
    println!("                    Examples: eng, eng+fra, eng+fra+deu");
    println!("  --min-conf <n>    Minimum confidence threshold (default: 50)");
    println!("  --no-clipboard    Don't copy to clipboard");
    println!();
    println!("Examples:");
    println!("  cargo run -- capture screen");
    println!("  cargo run -- capture screen --output ~/Desktop/test.png");
    println!("  cargo run -- capture screen --ocr");
    println!("  cargo run -- ocr screenshot.png");
    println!("  cargo run -- ocr screenshot.png --lang eng+fra --min-conf 60");
}

fn run_capture(args: &[String]) {
    // Parse capture type
    let capture_type = args[2].as_str();

    // Parse options
    let mut output_path: Option<PathBuf> = None;
    let mut include_cursor = true;
    let mut use_jpeg = false;
    let mut jpeg_quality = 85;
    let mut prefix: Option<String> = None;
    let mut run_ocr = false;
    let mut ocr_lang: Option<String> = None;
    let mut ocr_min_conf: Option<i32> = None;
    let mut ocr_clipboard = true;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --output requires a path");
                    std::process::exit(1);
                }
                output_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--no-cursor" => {
                include_cursor = false;
                i += 1;
            }
            "--jpeg" => {
                use_jpeg = true;
                // Check if next arg is a number
                if i + 1 < args.len() {
                    if let Ok(q) = args[i + 1].parse::<u8>() {
                        jpeg_quality = q;
                        i += 2;
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            "--prefix" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --prefix requires text");
                    std::process::exit(1);
                }
                prefix = Some(args[i + 1].clone());
                i += 2;
            }
            "--ocr" => {
                run_ocr = true;
                i += 1;
            }
            "--lang" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --lang requires a language code");
                    std::process::exit(1);
                }
                ocr_lang = Some(args[i + 1].clone());
                i += 2;
            }
            "--min-conf" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --min-conf requires a number");
                    std::process::exit(1);
                }
                let value: i32 = match args[i + 1].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Error: --min-conf requires a valid number");
                        std::process::exit(1);
                    }
                };
                ocr_min_conf = Some(value);
                i += 2;
            }
            "--no-clipboard" => {
                ocr_clipboard = false;
                i += 1;
            }
            _ => {
                eprintln!("Error: unknown option '{}'", args[i]);
                std::process::exit(1);
            }
        }
    }

    // Select backend
    let capture: CaptureData = if WaylandBackend::is_supported() {
        println!("Using Wayland backend...");
        let backend = WaylandBackend::new().expect("Failed to initialize Wayland backend");

        match capture_type {
            "screen" => backend.capture_screen().expect("Screen capture failed"),
            "area" => {
                println!("Note: On Wayland, area capture requires user interaction via portal dialog");
                backend.capture_area(0, 0, 0, 0).expect("Area capture failed")
            }
            "window" => {
                println!("Note: On Wayland, window capture requires user interaction via portal dialog");
                backend.capture_window(0).expect("Window capture failed")
            }
            _ => {
                eprintln!("Error: unknown capture type '{}'", capture_type);
                print_usage();
                std::process::exit(1);
            }
        }
    } else if X11Backend::is_supported() {
        println!("Using X11 backend...");
        let backend = X11Backend::new().expect("Failed to initialize X11 backend");

        match capture_type {
            "screen" => backend.capture_screen().expect("Screen capture failed"),
            "area" => {
                // Show GTK overlay for area selection
                println!("Select an area by dragging the mouse. Press ESC to cancel.");
                let selection = select_area()
                    .expect("Failed to show area selection overlay");

                match selection {
                    Some(area) => {
                        backend.capture_area(area.x, area.y, area.width, area.height)
                            .expect("Area capture failed")
                    }
                    None => {
                        eprintln!("Selection cancelled");
                        std::process::exit(0);
                    }
                }
            }
            "window" => {
                eprintln!("Error: window capture by ID not yet supported via CLI");
                eprintln!("Use 'capture screen' and crop manually");
                std::process::exit(1);
            }
            _ => {
                eprintln!("Error: unknown capture type '{}'", capture_type);
                print_usage();
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Error: No supported display backend found");
        eprintln!("This application requires X11 or Wayland");
        std::process::exit(1);
    };

    println!("Captured: {}x{}", capture.width, capture.height);
    println!("Format: {:?} ({} bpp)", capture.format, capture.format.bits_per_pixel);
    if capture.cursor.is_some() {
        println!("Cursor: captured ({})", if include_cursor { "will include" } else { "will exclude" });
    }

    // Build save config
    let format = if use_jpeg {
        ImageFormat::Jpeg { quality: jpeg_quality }
    } else {
        ImageFormat::Png
    };

    let mut config = SaveConfig::default()
        .with_format(format)
        .with_cursor(include_cursor);

    if let Some(path) = output_path {
        config = config.with_output_dir(path);
    }

    if let Some(p) = prefix {
        config = config.with_prefix(p);
    }

    // Save the capture
    let saved_path = match save_capture(&capture, &config) {
        Ok(path) => {
            println!("Saved to: {}", path.display());
            path
        }
        Err(e) => {
            eprintln!("Error saving capture: {}", e);
            std::process::exit(1);
        }
    };

    // Run OCR if requested
    if run_ocr {
        println!("Running OCR...");
        let mut ocr_config = OcrConfig::default()
            .with_clipboard(ocr_clipboard);

        if let Some(lang) = ocr_lang {
            ocr_config = ocr_config.with_language(lang);
        }

        if let Some(conf) = ocr_min_conf {
            ocr_config = ocr_config.with_min_confidence(conf);
        }

        match extract_text_from_path(&saved_path, &ocr_config) {
            Ok(result) => {
                println!("OCR successful!");
                println!("Confidence: {}%", result.confidence);
                println!("Extracted text:");
                println!("{}", "-".repeat(40));
                println!("{}", result.text);
                println!("{}", "-".repeat(40));
                if result.copied_to_clipboard {
                    println!("Text copied to clipboard");
                }
            }
            Err(e) => {
                eprintln!("OCR failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn run_ocr(args: &[String]) {
    let image_path = &args[2];

    // Parse OCR options
    let mut ocr_lang: Option<String> = None;
    let mut ocr_min_conf: Option<i32> = None;
    let mut ocr_clipboard = true;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--lang" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --lang requires a language code");
                    std::process::exit(1);
                }
                ocr_lang = Some(args[i + 1].clone());
                i += 2;
            }
            "--min-conf" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --min-conf requires a number");
                    std::process::exit(1);
                }
                let value: i32 = match args[i + 1].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Error: --min-conf requires a valid number");
                        std::process::exit(1);
                    }
                };
                ocr_min_conf = Some(value);
                i += 2;
            }
            "--no-clipboard" => {
                ocr_clipboard = false;
                i += 1;
            }
            _ => {
                eprintln!("Error: unknown option '{}'", args[i]);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    // Build OCR config
    let mut ocr_config = OcrConfig::default()
        .with_clipboard(ocr_clipboard);

    if let Some(lang) = ocr_lang {
        ocr_config = ocr_config.with_language(lang);
    }

    if let Some(conf) = ocr_min_conf {
        ocr_config = ocr_config.with_min_confidence(conf);
    }

    // Run OCR
    println!("Running OCR on: {}", image_path);
    match extract_text_from_path(image_path, &ocr_config) {
        Ok(result) => {
            println!("OCR successful!");
            println!("Confidence: {}%", result.confidence);
            println!("Extracted text:");
            println!("{}", "-".repeat(40));
            println!("{}", result.text);
            println!("{}", "-".repeat(40));
            if result.copied_to_clipboard {
                println!("Text copied to clipboard");
            }
        }
        Err(e) => {
            eprintln!("OCR failed: {}", e);
            std::process::exit(1);
        }
    }
}
