mod notification;

use std::sync::Arc;
use std::sync::Mutex;
use log::{LevelFilter, info};
use tauri::Listener;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Initialize logging
fn setup_logging() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();
    info!("Logging initialized");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set up logging
    setup_logging();
    
    info!("Starting application");
    let notification_manager = Arc::new(Mutex::new(notification::NotificationManager::new()));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(notification_manager)
        .setup(|app| {
            // Set up event listener for notification logs
            let app_handle = app.handle();
            app_handle.listen("notification-log", move |event| {
                if let Some(payload) = event.payload() {
                    info!("Notification log: {}", payload);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            notification::create_notification,
            notification::close_notification
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
