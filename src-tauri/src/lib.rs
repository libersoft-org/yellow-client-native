#![feature(str_as_str)]
mod notification;

use std::sync::Arc;
use std::sync::Mutex;
use log::{LevelFilter, info, error};
use serde_json::Value;
use tauri::{AppHandle, Event, Listener, Manager};


use std::collections::HashMap;
use std::random::random;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};
use uuid::Uuid;
use log::{info, error};

const max_windows = 4;




// Notification data structure
#[derive(Clone, serde::Serialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub duration: u64, // in seconds
    pub window_label: String, // Window label for tracking
}

// Position information for a notification
#[derive(Clone, Copy)]
pub struct NotificationPosition {
    id: usize,  // Unique position ID
    x: u32,
    y: u32,
    height: u32,
}

// Notification manager to keep track of active notifications
pub struct NotificationManager {
    notifications: HashMap<String, (WebviewWindow, Notification)>,
    positions: HashMap<String, NotificationPosition>, // Map notification ID to position
    next_position_id: usize,
    notification_width: u32,
    notification_height: u32,
    margin: u32,
}

impl NotificationManager {
    pub fn new() -> Self {
        NotificationManager {
            notifications: HashMap::new(),
            positions: HashMap::new(),
            next_position_id: 0,
            notification_width: 400,
            notification_height: 500,
            margin: 10,
        }
    }

    pub fn add_notification(&mut self, window: WebviewWindow, notification: Notification, x: u32, y: u32, actual_height: Option<u32>) {
        let id = window.label().to_string();
        let height = actual_height.unwrap_or(self.notification_height);
        let position = NotificationPosition {
            id: self.next_position_id,
            x,
            y,
            height,
        };
        self.next_position_id += 1;
        self.notifications.insert(id.clone(), (window, notification));
        self.positions.insert(id, position);
    }

    pub fn remove_notification(&mut self, id: &str) -> Option<NotificationPosition> {
        self.notifications.remove(id);
        self.positions.remove(id)
    }

    pub fn get_next_position(&self, screen_width: u32) -> (u32, u32) {
        // Start from top right corner
        let base_x = screen_width - self.notification_width - 20; // 20px margin from right
        let base_y = 20; // Start 20px from top

        info!("base_x: {}, base_y: {}, self.notification_width: {}, self.notification_height: {}", base_x, base_y, self.notification_width, self.notification_height);

        if self.positions.is_empty() {
            return (base_x, base_y);
        }

        // Find the lowest position (highest y value)
        let max_y = self.positions.values()
            .map(|pos| pos.y + pos.height + self.margin)
            .max()
            .unwrap_or(base_y);

        info!("max_y: {}", max_y);

        (base_x, max_y)
    }

    pub fn reposition_notifications(&mut self, app_handle: &AppHandle) {
        // Sort positions by their y coordinate
        let mut positions: Vec<_> = self.positions.iter().map(|(id, pos)| (id.clone(), pos.id, pos.x, pos.height)).collect();
        positions.sort_by_key(|(_, pos_id, _, _)| *pos_id);

        // Start from the top
        let mut current_y = 20; // Start 20px from top

        for (id, _, x, height) in positions {
            if let Some(window) = app_handle.get_webview_window(&id) {
                // Update position in our map
                if let Some(pos) = self.positions.get_mut(&id) {
                    pos.y = current_y;
                }

                // Update window position
                let _ = window.set_position(PhysicalPosition::new(x, current_y));

                // Log the repositioning
                info!("Repositioned notification {} to y={}, height={}", id, current_y, height);

                // Move to next position
                current_y += height + self.margin;
            }
        }
    }

    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.notification_width, self.notification_height)
    }

    // Get both window and notification data
    pub fn get_notification(&self, id: &str) -> Option<(&WebviewWindow, &Notification)> {
        self.notifications.get(id).map(|(window, notification)| (window, notification))
    }
}

// Create a global notification manager
pub type NotificationManagerState = Arc<Mutex<NotificationManager>>;

// Command to create a new notification
#[tauri::command]
pub async fn create_notification(
    app: AppHandle,
    title: String,
    message: String,
    duration: Option<u64>,
    state: tauri::State<'_, NotificationManagerState>,
) -> Result<String, String> {
    let notification_id = Uuid::new_v4().to_string();
    let duration = duration.unwrap_or(5); // Default 5 seconds

    // Create notification data
    let notification = Notification {
        id: notification_id.clone(),
        title,
        message,
        duration,
        window_label: notification_id.clone(), // Window label is the same as notification ID
    };

    // Get primary monitor dimensions
    let monitor = app.primary_monitor()
        .map_err(|e| format!("Failed to get primary monitor: {}", e))?
        .ok_or_else(|| "No primary monitor found".to_string())?;

    let monitor_size = monitor.size();

    // Get position for the notification from the manager
    let (notification_width, notification_height) = {
        let manager = state.lock().unwrap();
        manager.get_dimensions()
    };

    // Calculate position for the notification
    let (x, y) = {
        let manager = state.lock().unwrap();
        info!("Getting next position for notification, monitor width: {}, monitor height: {}", monitor_size.width, monitor_size.height);
        manager.get_next_position(monitor_size.width)
    };

    // Log the requested size
    info!("Creating notification window with requested size: {}x{}", notification_width, notification_height);

    // Get the monitor's scale factor
    let scale_factor = monitor.scale_factor();
    info!("Monitor scale factor: {}", scale_factor);

    // Adjust size for DPI scaling
    let logical_width = notification_width as f64;
    let logical_height = notification_height as f64;

    info!("Creating notification window with logical size: {}x{}", logical_width, logical_height);

    // Create a new window for the notification
    let notification_window = tauri::WebviewWindowBuilder::new(
        &app,
        notification_id.clone(),
        tauri::WebviewUrl::App("/notification".into())
    )
        .title("Notification")
        .inner_size(logical_width, logical_height)
        .decorations(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()
        .map_err(|e| format!("Failed to create notification window: {}", e))?;

    let label = notification_window.label().to_string();

    // Get the actual size after creation to account for DPI scaling
    let actual_height = if let Ok(size) = notification_window.inner_size() {
        info!("Actual window inner size after creation: {}x{}", size.width, size.height);
        Some(size.height)
    } else {
        info!("Could not get actual window size after creation");
        None
    };

    info!("Created notification window: {}", label);

    // No need for event listeners - we'll use commands instead

    // Store the notification data in the manager so it can be sent when the window is ready
    // The actual emission is handled by the notification-ready event listener in lib.rs

    // Position the window
    notification_window.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| format!("Failed to position notification window: {}", e))?;

    // Store notification data; actual emission handled globally in setup

    // Add to notification manager
    {
        let mut manager = state.lock().unwrap();
        manager.add_notification(notification_window.clone(), notification.clone(), x, y, actual_height);
    }

    // We don't need the auto-close timer here anymore
    // The frontend will handle the timeout and emit an event
    // which will trigger the close_notification command

    Ok(notification_id)
}


#[tauri::command]
pub fn add_notification(data: any) -> Result<(), String> {
    notifications[data.id] = data;
    Ok(())
}

#[tauri::command]
pub fn assign_notification(window_idx, notification_id) -> Result<(), String> {
    if (len(windows) <= window_idx) {
        let window = tauri::Window::new(window_idx, data);
        window.create().map_err(|e| format!("Failed to create window: {}", e));
        window_notifications.set(window_idx, notification_id);
    }
}



#[tauri::command]
pub fn notification_window_ready(
    _app: AppHandle,
    window: tauri::Window,
    state: tauri::State<'_, NotificationManagerState>,
) -> Result<Notification, String> {
    let window_label = window.label().to_string();
    info!("Notification window ready: {}", window_label);

    // Get the notification data from the manager
    let manager = state.lock().unwrap();
    if let Some((_, notification_data)) = manager.get_notification(&window_label) {
        // Return the notification data to the window
        Ok(notification_data.clone())
    } else {
        error!("No notification data found for window: {}", window_label);
        Err(format!("No notification data found for window: {}", window_label))
    }
}

// Command to get window size
#[tauri::command]
pub fn get_window_size(window: tauri::Window) -> Result<(u32, u32), String> {
    match window.inner_size() {
        Ok(size) => {
            info!("Window {} size: {}x{}", window.label(), size.width, size.height);
            Ok((size.width, size.height))
        },
        Err(e) => Err(format!("Failed to get window size: {}", e))
    }
}

// Command to get the monitor's scale factor
#[tauri::command]
pub fn get_scale_factor(window: tauri::Window) -> Result<f64, String> {
    match window.scale_factor() {
        Ok(scale) => {
            info!("Window {} scale factor: {}", window.label(), scale);
            Ok(scale)
        },
        Err(e) => Err(format!("Failed to get scale factor: {}", e))
    }
}

// Command to close a notification manually
#[tauri::command]
pub fn close_notification(
    app: AppHandle,
    notification_id: String,
    state: tauri::State<'_, NotificationManagerState>,
) -> Result<(), String> {
    info!("Closing notification: {}", notification_id);
    if let Some(window) = app.get_webview_window(&notification_id) {
        window.close().map_err(|e| format!("Failed to close notification: {}", e))?;
    }

    // Remove from manager
    let mut manager = state.lock().unwrap();
    if manager.remove_notification(&notification_id).is_some() {
        info!("Notification removed from manager: {}", notification_id);
        manager.reposition_notifications(&app);
    } else {
        info!("Notification not found in manager: {}", notification_id);
    }

    Ok(())
}



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
            
            // Helper closure to handle notification close events
            let handle_notification_close = |event: Event, action: &str, app_handle: &AppHandle| {
                info!("Received notification-{} event", action);
                info!("Notification {} payload: {}", action, event.payload());
                
                // Try to get the window label from the event
                if let Some(window_label) = event.window_label() {
                    info!("Notification {} from window: {}", action, window_label);
                    info!("Action: {}", action);
                    
                    // Get the window by label and close it
                    if let Some(window) = app_handle.get_webview_window(&window_label) {
                        if let Err(e) = window.close() {
                            error!("Failed to close notification window: {}", e);
                        } else {
                            info!("Successfully closed notification window: {}", window_label);
                            
                            // Remove from notification manager
                            let state = app_handle.state::<notification::NotificationManagerState>();
                            let mut manager = state.lock().unwrap();
                            if manager.remove_notification(&window_label).is_some() {
                                manager.reposition_notifications(app_handle);
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
                        if let Some(window) = app_handle.get_webview_window(id) {
                            if let Err(e) = window.close() {
                                error!("Failed to close notification window: {}", e);
                            } else {
                                info!("Successfully closed notification window: {}", id);
                                
                                // Remove from notification manager
                                let state = app_handle.state::<notification::NotificationManagerState>();
                                let mut manager = state.lock().unwrap();
                                if manager.remove_notification(id).is_some() {
                                    manager.reposition_notifications(app_handle);
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            notification::create_notification,
            notification::close_notification,
            notification::notification_window_ready,
            notification::get_window_size,
            notification::get_scale_factor
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


