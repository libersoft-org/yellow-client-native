#![feature(str_as_str)]
mod notification;

use std::sync::Arc;
use std::sync::Mutex;
use log::{LevelFilter, info, error};
use serde_json::Value;
use tauri::{Event, Listener, Manager, Emitter};

// Extension trait to get window label and id from event
trait EventExt {
    fn window_label(&self) -> Option<String>;
    fn notification_id(&self) -> Option<String>;
    fn parse_payload(&self) -> Option<Value>;
}

impl EventExt for Event {
    fn parse_payload(&self) -> Option<Value> {
        serde_json::from_str::<Value>(self.payload()).ok()
    }
    
    fn window_label(&self) -> Option<String> {
        // First try to parse as JSON
        if let Some(json) = self.parse_payload() {
            // Try to get window label from JSON
            if let Some(window) = json.get("window").and_then(|w| w.as_str()) {
                return Some(window.to_string());
            }
        }
        
        // Fallback to the old method
        let payload = self.payload();
        if payload.starts_with("window-") {
            return Some(payload["window-".len()..].to_string());
        }
        None
    }
    
    fn notification_id(&self) -> Option<String> {
        // Try to parse payload as JSON and extract id
        if let Some(json) = self.parse_payload() {
            // Try to get notification id from JSON
            if let Some(id) = json.get("id").and_then(|id| id.as_str()) {
                return Some(id.to_string());
            }
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
                info!("my-log: {}", payload);
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
                    if let Some((window, notification_data)) = manager.get_notification(&window_label) {
                        // Use the stored notification data
                        if let Err(e) = window.emit("notification-data", &notification_data) {
                            error!("Failed to emit notification-data event: {}", e);
                        } else {
                            info!("Successfully emitted notification-data event to window: {}", window_label);
                        }
                    } else {
                        error!("No notification window found for label: {:?}", window_label);
                    }
                }
            });

            // Listen for notification-data-received acknowledgments
            let ack_handle = app_handle.clone();
            ack_handle.listen("notification-data-received", move |event| {
                if let Some(json) = event.parse_payload() {
                    if let Some(id) = json.get("id").and_then(|id| id.as_str()) {
                        let status = json.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
                        info!("Notification data received acknowledgment: id={}, status={}", id, status);
                        
                        if status != "success" {
                            if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
                                error!("Error in notification {}: {}", id, error);
                            }
                        }
                    }
                }
            });
            
            // Listen for notification-clicked events
            let click_handle = app_handle.clone();
            let click_app_handle = app_handle.clone(); // Clone for use inside closure
            click_handle.clone().listen("notification-clicked", move |event| {
                info!("Received notification-clicked event");
                info!("Notification clicked payload: {}", event.payload());
                
                // Parse the payload to get structured data
                let mut notification_id = None;
                let mut action = "clicked";
                
                if let Some(json) = event.parse_payload() {
                    notification_id = json.get("id").and_then(|id| id.as_str()).map(String::from);
                    action = json.get("action").and_then(|a| a.as_str()).unwrap_or("clicked");
                    
                    if let Some(timestamp) = json.get("timestamp").and_then(|t| t.as_str()) {
                        info!("Notification clicked at: {}", timestamp);
                    }
                }
                
                // Fallback to event.notification_id() if not found in payload
                if notification_id.is_none() {
                    notification_id = event.notification_id();
                }
                
                if let Some(id) = &notification_id {
                    info!("Notification ID from payload: {}", id);
                    info!("Action: {}", action);
                    
                    // Get window label as fallback
                    let window_label = event.window_label();
                    
                    // Use notification ID if available, otherwise use window label
                    let id_to_use = id;
                    
                    // Close the notification window
                    if let Some(window) = click_app_handle.get_webview_window(id_to_use) {
                        if let Err(e) = window.close() {
                            error!("Failed to close notification window: {}", e);
                        } else {
                            info!("Successfully closed notification window: {:?}", window_label);
                            
                            // Remove from notification manager
                            let state = click_app_handle.state::<notification::NotificationManagerState>();
                            let mut manager = state.lock().unwrap();
                            if manager.remove_notification(id_to_use).is_some() {
                                manager.reposition_notifications(&click_app_handle);
                            }
                        }
                    } else {
                        error!("No notification window found for label: {:?}", window_label);
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
