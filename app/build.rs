use std::process::{Command, Stdio};

fn main() {
    // Create icons
    Command::new("cargo")
        .args(&["tauri", "icon", "icon.svg"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()
        .unwrap();

    // Build Tauri
    tauri_build::build()
}
