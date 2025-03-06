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
            let app_handle = app.handle();
            
            // Set up event listener for notification logs
            app_handle.listen("notification-log", move |event| {
                let payload = event.payload();
                info!("Notification log: {}", payload);
            });

            // Listen globally for notification-ready events
            app_handle.listen_global("notification-ready", move |event| {
                if let Some(window_label) = event.window().map(|w| w.label().to_string()) {
                    info!("Received notification-ready event from window: {}", window_label);
                    let state = app_handle.state::<notification::NotificationManagerState>();
                    let manager = state.lock().unwrap();
                    if let Some(window) = manager.notifications.get(&window_label) {
                        // Retrieve notification data from window label or other storage if needed
                        // For simplicity, assuming notification data is stored or retrievable here
                        // You might need to adjust this logic based on your actual data storage
                        let notification_data = notification::Notification {
                            id: window_label.clone(),
                            title: "Notification".into(),
                            message: "You have a new notification.".into(),
                            duration: 5,
                        };
                        if let Err(e) = window.emit("notification-data", &notification_data) {
                            error!("Failed to emit notification-data event: {}", e);
                        } else {
                            info!("Successfully emitted notification-data event to window: {}", window_label);
                        }
                    } else {
                        error!("No notification window found for label: {}", window_label);
                    }
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
