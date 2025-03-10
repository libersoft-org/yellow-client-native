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
        // We can't directly return a reference from window_label()
        // since it returns an owned String
        None
    }
    
    fn window_label(&self) -> Option<String> {
        // First try to parse from JSON payload
        if let Some(json) = self.parse_payload() {
            if let Some(window) = json.get("window").and_then(|w| w.as_str()) {
                return Some(window.to_string());
            }
        }
        
        // Try to get from event payload if it's a window event
        // This is based on the RunEvent::WindowEvent structure in Tauri
        let payload = self.payload();
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) {
            if let Some(label) = value.get("label").and_then(|l| l.as_str()) {
                return Some(label.to_string());
            }
        }
        
        // Last fallback to the payload format
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
            
            // Listen for notification-clicked and notification-timeout events
            let click_handle = app_handle.clone();
            let click_app_handle = app_handle.clone(); // Clone for use inside closure
            
            // Helper function to handle notification close events
            let handle_notification_close = move |event: Event, action: &str| {
                info!("Received notification-{} event", action);
                info!("Notification {} payload: {}", action, event.payload());
                
                // Try to get the window label from the event
                if let Some(window_label) = event.window_label() {
                    info!("Notification {} from window: {}", action, window_label);
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
            };
            
            // Listen for notification-clicked events
            click_handle.clone().listen("notification-clicked", move |event| {
                handle_notification_close(event, "clicked");
            });
            
            // Listen for notification-timeout events
            let timeout_handle = app_handle.clone();
            let timeout_app_handle = app_handle.clone(); // Clone for use inside closure
            timeout_handle.listen("notification-timeout", move |event| {
                handle_notification_close(event, "timeout");
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            notification::create_notification,
            notification::close_notification,
            notification::notification_ready,
            notification::get_window_size,
            notification::get_scale_factor
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
