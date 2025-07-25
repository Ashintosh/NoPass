use slint_build;
use std::fs;
use std::path::{ Path, PathBuf };

fn main() {
    let ui_root = PathBuf::from("ui/");

    // Compile the main slint entry file
    compile_main_ui(&ui_root);

    // Set up cargo to recompile when any .slint file changes
    setup_cargo_recompile_triggers(&ui_root);
}

/// Compile the entry slint UI file 
fn compile_main_ui(ui_root: &PathBuf) {
    // Use native styling configuration
    let config = slint_build::CompilerConfiguration::new()
        .with_style("native".into());

    slint_build::compile_with_config(
        ui_root.join("app.slint"),
        config
    ).expect("Failed to compile slint entry file with config");
}

/// Sets up cargo to recompile when any .slint file in the given directory changes
fn setup_cargo_recompile_triggers(ui_root: &PathBuf) {
    for path in find_slint_files(ui_root) {
        if let Some(path_str) = path.to_str() {
            println!("cargo:rerun-if-changed={}", path_str);
        }
    }
}

/// Recursively find all files with '.slint' extension in the given directory
fn find_slint_files(directory: &PathBuf) -> Vec<PathBuf> {
    let mut slint_files = Vec::new();

    // Read directory entries, return empty vec if directory can't be read
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return slint_files,
    };

    // Process each entry in the directory
    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // Recursively search subdirectories
            slint_files.extend(find_slint_files(&path));
        } else if is_slint_file(&path) {
            slint_files.push(path);
        }
    }

    slint_files
}

/// Check if a given path points to a .slint file
fn is_slint_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| ext == "slint")
}
