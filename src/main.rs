//! Reset Launcher - Reset Offline Launcher
//! Supports both GUI and CLI modes

#![windows_subsystem = "windows"]

mod cli;
mod gui;

use std::env;
use std::path::Path;
use eframe::egui;
use image::GenericImageView;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Check if we should run GUI mode
    // GUI mode: no args, or --gui flag, or first arg is not a known CLI command
    let should_run_gui = args.len() <= 1
        || args.iter().any(|a| a == "--gui" || a == "-g")
        || (args.len() > 1 && !is_cli_command(&args[1]));

    if should_run_gui {
        // Run GUI mode
        run_gui()
    } else {
        // Run CLI mode
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(cli::run_cli())
    }
}

/// Load the window icon from the ICO file
fn create_window_icon() -> egui::IconData {
    let icon_path = Path::new(r"C:\Users\backu\Downloads\reset launcher icone.ico");

    if icon_path.exists() {
        // Load the ICO image - the image crate supports ICO format
        // ICO files can contain multiple images at different sizes; pick the largest
        let image = image::open(icon_path).expect("Failed to load icon ICO");
        let (width, height) = image.dimensions();

        // Resize to 64x64 if needed
        let resized = if width != 64 || height != 64 {
            image.resize_exact(64, 64, image::imageops::FilterType::Lanczos3)
        } else {
            image
        };

        // Convert to RGBA8
        let rgba = resized.to_rgba8();
        let (new_width, new_height) = rgba.dimensions();

        egui::IconData {
            rgba: rgba.into_raw(),
            width: new_width,
            height: new_height,
        }
    } else {
        // Fallback to a simple red square if ICO not found
        let size = 64;
        let mut pixels = vec![0u8; size * size * 4];
        let r = 0xE0;
        let g = 0x00;
        let b = 0x18;
        let a = 0xFF;

        for i in 0..size * size {
            pixels[i * 4] = r;
            pixels[i * 4 + 1] = g;
            pixels[i * 4 + 2] = b;
            pixels[i * 4 + 3] = a;
        }

        egui::IconData {
            rgba: pixels,
            width: size as u32,
            height: size as u32,
        }
    }
}

/// Check if the argument is a known CLI subcommand
fn is_cli_command(arg: &str) -> bool {
    matches!(arg, "build" | "prepare" | "launch" | "check" | "help" | "--help" | "-h")
}

/// Run the GUI application
fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    // Use a FIXED safe position (100, 100) - guaranteed on-screen for ANY monitor setup
    // This prevents the "slide down" animation where Windows moves window from (0,0) to center
    // which can end up off-screen on high-DPI or multi-monitor setups
    // Also disable resizing to prevent layout shifts
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([700.0, 550.0])
            .with_max_inner_size([900.0, 700.0])  // Fixed size - no resize = no layout shift
            .with_title("Reset Launcher")
            .with_position(eframe::egui::pos2(100.0, 100.0))  // Safe upper-left position
            .with_visible(true)
            .with_resizable(false)  // Prevent resize animations/shifts
            .with_decorations(false)  // Hide Windows default titlebar (we have custom one)
            .with_icon(create_window_icon()),  // Set custom icon from PNG
        persist_window: false,  // Never restore old positions
        ..Default::default()
    };

    eframe::run_native(
        "Reset Launcher",
        options,
        Box::new(|cc| Ok(Box::new(gui::app::ResetLauncherApp::new(cc)))),
    ).map_err(|e| format!("GUI error: {}", e).into())
}