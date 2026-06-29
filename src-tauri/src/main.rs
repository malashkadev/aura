// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // The global Win32 keyboard hook is initialized during the Tauri app setup in lib.rs
    glaido_app_lib::run()
}

