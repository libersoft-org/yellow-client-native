mod audio;
mod commands;
mod misc;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod notifications;

use log::{info, LevelFilter};
use tauri::Listener;

#[cfg(desktop)]
use tauri::Manager;

use tauri::{WebviewUrl, WebviewWindowBuilder};

use serde::Deserialize;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_sentry::{minidump, sentry};

// Define the plugin config
#[derive(Deserialize)]
struct Config {}

// Initialize logging
fn setup_logging() {

}




#[cfg(desktop)]
fn setup_desktop_notifications(_app: &mut tauri::App) {
    // todo: ensure that notifications window is closed when main window closes, even if the js in main window doesn't call close_notifications_window
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    let mut builder = tauri::Builder::default();
    // Plugins that should be available on all platforms
    let builder = builder
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init());


    
    builder.setup(|app| {
            let app_handle = app.handle().clone();

            // Set up event listener for logs
            let log_handle = app_handle.clone();
            log_handle.listen("my-log", move |event| {
                let payload = event.payload();
                info!("my-log: {}", payload);
            });

        Ok(())

        })
        .invoke_handler(tauri::generate_handler![
            commands::get_window_size,
            commands::get_scale_factor,
            commands::log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
