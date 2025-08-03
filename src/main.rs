// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod errors;
mod handlers;
mod models;
mod utils;

use handlers::WindowHandler;
use handlers::main_window::MainWindowHandler;

slint::include_modules!();

fn main() {
    #[cfg(debug_assertions)]
    print_debug_message();

    #[cfg(windows)]
    std::env::set_var("SLINT_BACKEND", "winit-software");

    // Start the main window
    let mut main_window_handler = MainWindowHandler::new();
    main_window_handler.get_window().upgrade().unwrap().set_win_title("NoPass".into());
    main_window_handler.run();
}

/// Display prominent debug build warning if the debug feature in enabled
#[cfg(debug_assertions)]
fn print_debug_message() {
    println!();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║                                                  ║");
    println!("║               WARNING: DEBUG BUILD               ║");
    println!("║              Not for production use              ║");
    println!("║                                                  ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
}
