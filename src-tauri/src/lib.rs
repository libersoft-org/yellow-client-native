#![feature(str_as_str)]
mod notification;

use std::sync::Arc;
use std::sync::Mutex;
use log::{LevelFilter, info, error};
use tauri::{Event, Listener, Manager, Emitter};

// Extension trait to get window label from event
trait EventExt {
    fn window_label(&self) -> Option<String>;
}

impl EventExt for Event {
    fn window_label(&self) -> Option<String> {
        // In Tauri 2, we need to extract window label from payload
        let payload = self.payload();
        // payload is already a &str
        if payload.starts_with("window-") {
            return Some(payload["window-".len()..].to_string());
        }
        None
    }
}

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
            let app_handle = app.handle().clone();
            
            // Set up event listener for notification logs
            let log_handle = app_handle.clone();
            log_handle.listen("my-log", move |event| {
                let payload = event.payload();
                info!("Notification log: {}", payload);
            });

            // Listen globally for notification-ready events
            let ready_handle = app_handle.clone();
            let ready_app_handle = app_handle.clone(); // Clone for use inside closure
            ready_handle.listen("notification-ready", move |event| {
                info!("Received notification-ready event");
                // Extract window label from event payload or use a default approach
                if let Some(window_label) = event.window_label() {
                    info!("Received notification-ready event from window: {}", window_label);
                    let state = ready_app_handle.state::<notification::NotificationManagerState>();
                    let manager = state.lock().unwrap();
                    if let Some(window) = manager.get_notification_window(&window_label) {
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

            // Listen for notification-clicked events
            let click_handle = app_handle.clone();
            let click_app_handle = app_handle.clone(); // Clone for use inside closure
            click_handle.clone().listen("notification-clicked", move |event| {
                info!("Received notification-clicked event");
                if let Some(window_label) = event.window_label() {
                    info!("Notification clicked: {}", window_label);
                    // Close the notification window
                    if let Some(window) = click_app_handle.get_webview_window(&window_label) {
                        if let Err(e) = window.close() {
                            error!("Failed to close notification window: {}", e);
                        } else {
                            info!("Successfully closed notification window: {}", window_label);
                            
                            // Remove from notification manager
                            let state = click_app_handle.state::<notification::NotificationManagerState>();
                            let mut manager = state.lock().unwrap();
                            if manager.remove_notification(&window_label).is_some() {
                                manager.reposition_notifications(&click_app_handle);
                            }
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
