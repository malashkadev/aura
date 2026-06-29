// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod keyboard_hook;
pub mod audio_recorder;


#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            keyboard_hook::start_hook(move |is_down| {
                use tauri::Emitter;
                let event_name = "shortcut-state";
                let payload = if is_down { "down" } else { "up" };
                let _ = app_handle.emit(event_name, payload);
                println!("Alt+N state: {}", payload);
            }).expect("Failed to start global Win32 keyboard hook");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

