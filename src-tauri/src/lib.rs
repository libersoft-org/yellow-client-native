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
    fn label(&self) -> Option<&str>;
}

impl EventExt for Event {
    fn parse_payload(&self) -> Option<Value> {
        serde_json::from_str::<Value>(self.payload()).ok()
    }
    
    fn label(&self) -> Option<&str> {
        // Try to extract the window label from the event
        // This is a simplified implementation since Event doesn't have a direct window() method
        None
    }
    
    fn window_label(&self) -> Option<String> {
        // Try to parse from JSON payload
        if let Some(json) = self.parse_payload() {
            if let Some(window) = json.get("window").and_then(|w| w.as_str()) {
                return Some(window.to_string());
            }
        }
        
        // Try to get window label from event metadata
        if let Some(window_label) = self.label() {
            return Some(window_label.to_string());
        }
        
        // Last fallback to the old method
        let payload = self.payload();
        if payload.starts_with("window-") {
            return Some(payload["window-".len()..].to_string());
        }
        None
    }
    
    fn notification_id(&self) -> Option<String> {
        // Try to get window label from event metadata
        if let Some(window_label) = self.label() {
            return Some(window_label.to_string());
        }
        
        // Fallback to parsing from JSON payload
        if let Some(json) = self.parse_payload() {
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
        //add milliseconds to the logs
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
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
                info!("Received notification-ready event: {}", event.payload());
                
                // Try to get the window label from the event
                if let Some(window_label) = event.window_label() {
                    info!("Received notification-ready event from window: {}", window_label);
                    
                    // Get the window by label
                    if let Some(window) = ready_app_handle.get_webview_window(&window_label) {
                        let state = ready_app_handle.state::<notification::NotificationManagerState>();
                        let manager = state.lock().unwrap();
                        
                        if let Some((_, notification_data)) = manager.get_notification(&window_label) {
                            // Use the stored notification data
                            if let Err(e) = window.emit("notification-data", &notification_data) {
                                error!("Failed to emit notification-data event: {}", e);
                            } else {
                                info!("Successfully emitted notification-data event to window: {}", window_label);
                            }
                        } else {
                            error!("No notification window found for label: {}", window_label);
                        }
                    } else {
                        error!("Could not find window with label: {}", window_label);
                    }
                } else {
                    error!("No window label found in notification-ready event");
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
                
                // Get action from payload
                let mut action = String::from("clicked");
                if let Some(json) = event.parse_payload() {
                    action = json.get("action").and_then(|a| a.as_str()).unwrap_or("clicked").to_string();
                    
                    if let Some(timestamp) = json.get("timestamp").and_then(|t| t.as_str()) {
                        info!("Notification clicked at: {}", timestamp);
                    }
                }
                
                // Try to get the window label from the event
                if let Some(window_label) = event.window_label() {
                    info!("Notification clicked from window: {}", window_label);
                    info!("Action: {}", action);
                    
                    // Get the window by label and close it
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
                        error!("Could not find window with label: {}", window_label);
                    }
                } else {
                    // Fallback to the old method
                    let mut notification_id = None;
                    
                    if let Some(json) = event.parse_payload() {
                        notification_id = json.get("id").and_then(|id| id.as_str()).map(String::from);
                    }
                    
                    // Fallback to event.notification_id() if not found in payload
                    if notification_id.is_none() {
                        notification_id = event.notification_id();
                    }
                    
                    if let Some(id) = &notification_id {
                        info!("Notification ID from payload: {}", id);
                        info!("Action: {}", action);
                        
                        // Close the notification window
                        if let Some(window) = click_app_handle.get_webview_window(id) {
                            if let Err(e) = window.close() {
                                error!("Failed to close notification window: {}", e);
                            } else {
                                info!("Successfully closed notification window: {}", id);
                                
                                // Remove from notification manager
                                let state = click_app_handle.state::<notification::NotificationManagerState>();
                                let mut manager = state.lock().unwrap();
                                if manager.remove_notification(id).is_some() {
                                    manager.reposition_notifications(&click_app_handle);
                                }
                            }
                        } else {
                            error!("No notification window found for ID: {}", id);
                        }
                    } else {
                        error!("Could not determine notification ID from event");
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
