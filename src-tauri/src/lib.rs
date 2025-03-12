#![feature(str_as_str)]
mod notification;

use std::sync::Arc;
use std::sync::Mutex;
use log::{LevelFilter, info, error};
use serde_json::Value;
use tauri::{AppHandle, Event, Listener, Manager};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{PhysicalPosition, WebviewWindow};
use uuid::Uuid;

const MAX_WINDOWS: usize = 4;
// Notification data structure
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub duration: u64, // in seconds
    pub window_label: Option<String>, // Window label for tracking
    pub timestamp: Option<u64>, // Timestamp when notification was first displayed
    pub notification_type: String, // Type of notification (e.g., 'new_message')
}

// Position information for a notification
#[derive(Clone, Copy)]
pub struct NotificationPosition {
    id: usize,  // Unique position ID
    x: u32,
    y: u32,
    height: u32,
}

// Notification manager to keep track of active notifications and windows
pub struct NotificationManager {
    // Windows available for displaying notifications
    windows: HashMap<String, WebviewWindow>, // window_id -> WebviewWindow
    // Notifications waiting to be displayed
    notification_queue: Vec<Notification>,
    // Mapping of which notification is assigned to which window
    window_assignments: HashMap<String, String>, // window_id -> notification_id
    // Positions of windows
    positions: HashMap<String, NotificationPosition>, // window_id -> position
    next_position_id: usize,
    notification_width: u32,
    notification_height: u32,
    margin: u32,
}

impl NotificationManager {
    pub fn new() -> Self {
        NotificationManager {
            windows: HashMap::new(),
            notification_queue: Vec::new(),
            window_assignments: HashMap::new(),
            positions: HashMap::new(),
            next_position_id: 0,
            notification_width: 400,
            notification_height: 500,
            margin: 10,
        }
    }

    // Add a new notification to the queue
    pub fn add_notification(&mut self, notification: Notification) {
        self.notification_queue.push(notification);
    }

    // Register a window that can display notifications
    pub fn register_window(&mut self, window: WebviewWindow, x: u32, y: u32, actual_height: Option<u32>) -> String {
        let window_id = window.label().to_string();
        let height = actual_height.unwrap_or(self.notification_height);
        
        // Create position info
        let position = NotificationPosition {
            id: self.next_position_id,
            x,
            y,
            height,
        };
        self.next_position_id += 1;
        
        // Store window and position
        self.windows.insert(window_id.clone(), window);
        self.positions.insert(window_id.clone(), position);
        
        window_id
    }

    // Remove a notification from a window
    pub fn remove_notification(&mut self, window_id: &str) -> Option<Notification> {
        if let Some(notification_id) = self.window_assignments.remove(window_id) {
            // Find and remove the notification from the queue if it's there
            if let Some(pos) = self.notification_queue.iter().position(|n| n.id == notification_id) {
                let notification = self.notification_queue.remove(pos);
                return Some(notification);
            }
        }
        None
    }

    // Get the next available position for a notification window
    pub fn get_next_position(&self, screen_width: u32) -> (u32, u32) {
        // Start from top right corner
        let base_x = screen_width - self.notification_width - 20; // 20px margin from right
        let base_y = 20; // Start 20px from top

        info!("base_x: {}, base_y: {}, self.notification_width: {}, self.notification_height: {}", 
              base_x, base_y, self.notification_width, self.notification_height);

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

    // Reposition all notification windows
    pub fn reposition_notifications(&mut self, app_handle: &AppHandle) {
        // Sort positions by their y coordinate
        let mut positions: Vec<_> = self.positions.iter()
            .map(|(id, pos)| (id.clone(), pos.id, pos.x, pos.height))
            .collect();
        positions.sort_by_key(|(_, pos_id, _, _)| *pos_id);

        // Start from the top
        let mut current_y = 20; // Start 20px from top

        for (id, _, x, height) in positions {
            if let Some(window) = self.windows.get(&id) {
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

    // Get window dimensions
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.notification_width, self.notification_height)
    }

    // Get notification assigned to a window
    pub fn get_notification_for_window(&self, window_id: &str) -> Option<&Notification> {
        if let Some(notification_id) = self.window_assignments.get(window_id) {
            self.notification_queue.iter().find(|n| &n.id == notification_id)
        } else {
            None
        }
    }

    // Get window by ID
    pub fn get_window(&self, window_id: &str) -> Option<&WebviewWindow> {
        self.windows.get(window_id)
    }

    // Assign a notification to a window
    pub fn assign_notification_to_window(&mut self, window_id: &str, notification_id: &str) -> bool {
        // Check if window exists
        if !self.windows.contains_key(window_id) {
            return false;
        }
        
        // Check if notification exists
        if !self.notification_queue.iter().any(|n| n.id == notification_id) {
            return false;
        }
        
        // Assign notification to window
        self.window_assignments.insert(window_id.to_string(), notification_id.to_string());
        
        // Update notification's window_label
        for notification in &mut self.notification_queue {
            if notification.id == notification_id {
                notification.window_label = Some(window_id.to_string());
                break;
            }
        }
        
        true
    }

    // Get next notification from queue that isn't already assigned
    pub fn get_next_unassigned_notification(&self) -> Option<&Notification> {
        let assigned_ids: Vec<String> = self.window_assignments.values().cloned().collect();
        self.notification_queue.iter()
            .find(|n| !assigned_ids.contains(&n.id))
    }

    // Check if we have available windows (under MAX_WINDOWS)
    pub fn has_available_window_slots(&self) -> bool {
        self.windows.len() < MAX_WINDOWS
    }

    // Find an available window that doesn't have a notification assigned
    pub fn find_available_window(&self) -> Option<String> {
        for window_id in self.windows.keys() {
            if !self.window_assignments.contains_key(window_id) {
                return Some(window_id.clone());
            }
        }
        None
    }

    // Set timestamp for a notification when it's first displayed
    pub fn set_notification_timestamp(&mut self, notification_id: &str) {
        for notification in &mut self.notification_queue {
            if notification.id == notification_id && notification.timestamp.is_none() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                notification.timestamp = Some(timestamp);
                break;
            }
        }
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
    notification_type: Option<String>,
    state: tauri::State<'_, NotificationManagerState>,
) -> Result<String, String> {
    let notification_id = Uuid::new_v4().to_string();
    let duration = duration.unwrap_or(5); // Default 5 seconds
    let notification_type = notification_type.unwrap_or_else(|| "new_message".to_string());

    // Create notification data
    let notification = Notification {
        id: notification_id.clone(),
        title,
        message,
        duration,
        window_label: None, // Will be set when assigned to a window
        timestamp: None,    // Will be set when first displayed
        notification_type,
    };

    // Add notification to queue
    {
        let mut manager = state.lock().unwrap();
        manager.add_notification(notification);
    }

    // Try to display the notification if we have available windows
    process_notification_queue(&app, state.clone())?;

    Ok(notification_id)
}

// Process the notification queue, creating windows as needed
fn process_notification_queue(
    app: &AppHandle,
    state: NotificationManagerState,
) -> Result<(), String> {
    let mut create_window = false;
    let mut assign_to_window = None;
    
    // Check if we need to create a new window or use an existing one
    {
        let manager = state.lock().unwrap();
        
        // If we have unassigned notifications and available window slots, create a new window
        if manager.get_next_unassigned_notification().is_some() {
            if manager.has_available_window_slots() {
                create_window = true;
            } else if let Some(window_id) = manager.find_available_window() {
                assign_to_window = Some(window_id);
            }
        }
    }
    
    // Create a new window if needed
    if create_window {
        create_notification_window(app, state.clone())?;
    } else if let Some(window_id) = assign_to_window {
        // Assign notification to existing window
        assign_next_notification_to_window(app, &window_id, state.clone())?;
    }
    
    Ok(())
}

// Create a new notification window
fn create_notification_window(
    app: &AppHandle,
    state: NotificationManagerState,
) -> Result<String, String> {
    // Generate a unique window ID
    let window_id = format!("notification-window-{}", Uuid::new_v4());
    
    // Get primary monitor dimensions
    let monitor = app.primary_monitor()
        .map_err(|e| format!("Failed to get primary monitor: {}", e))?
        .ok_or_else(|| "No primary monitor found".to_string())?;

    let monitor_size = monitor.size();

    // Get position and dimensions for the notification window
    let (notification_width, notification_height) = {
        let manager = state.lock().unwrap();
        manager.get_dimensions()
    };

    // Calculate position for the notification
    let (x, y) = {
        let manager = state.lock().unwrap();
        info!("Getting next position for notification, monitor width: {}, monitor height: {}", 
              monitor_size.width, monitor_size.height);
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
        app,
        window_id.clone(),
        tauri::WebviewUrl::App("/notification".into())
    )
        .title("Notification")
        .inner_size(logical_width, logical_height)
        .decorations(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()
        .map_err(|e| format!("Failed to create notification window: {}", e))?;

    // Get the actual size after creation to account for DPI scaling
    let actual_height = if let Ok(size) = notification_window.inner_size() {
        info!("Actual window inner size after creation: {}x{}", size.width, size.height);
        Some(size.height)
    } else {
        info!("Could not get actual window size after creation");
        None
    };

    // Position the window
    notification_window.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| format!("Failed to position notification window: {}", e))?;

    // Register the window with the notification manager
    {
        let mut manager = state.lock().unwrap();
        manager.register_window(notification_window, x, y, actual_height);
    }

    info!("Created notification window: {}", window_id);
    
    // The window will call notification_window_ready when it's ready
    // and we'll assign a notification to it then
    
    Ok(window_id)
}

// Assign the next notification in queue to a window
fn assign_next_notification_to_window(
    app: &AppHandle,
    window_id: &str,
    state: NotificationManagerState,
) -> Result<(), String> {
    let notification_id = {
        let manager = state.lock().unwrap();
        manager.get_next_unassigned_notification()
            .map(|n| n.id.clone())
            .ok_or_else(|| "No unassigned notifications available".to_string())?
    };
    
    // Assign notification to window
    {
        let mut manager = state.lock().unwrap();
        if !manager.assign_notification_to_window(window_id, &notification_id) {
            return Err("Failed to assign notification to window".to_string());
        }
    }
    
    // Emit notification data to the window
    emit_notification_data_event(app, window_id, state.clone())?;
    
    Ok(())
}

// Emit notification data to a specific window
fn emit_notification_data_event(
    app: &AppHandle,
    window_id: &str,
    state: NotificationManagerState,
) -> Result<(), String> {
    let notification = {
        let mut manager = state.lock().unwrap();
        
        // Get the notification assigned to this window
        let notification = manager.get_notification_for_window(window_id)
            .ok_or_else(|| format!("No notification assigned to window {}", window_id))?
            .clone();
        
        // Set timestamp if not already set
        if notification.timestamp.is_none() {
            manager.set_notification_timestamp(&notification.id);
        }
        
        notification
    };
    
    // Get the window
    let window = {
        let manager = state.lock().unwrap();
        manager.get_window(window_id)
            .ok_or_else(|| format!("Window {} not found", window_id))?
            .clone()
    };
    
    // Emit the notification data to the window
    window.emit("notification-data", &notification)
        .map_err(|e| format!("Failed to emit notification data: {}", e))?;
    
    info!("Emitted notification data to window {}: {:?}", window_id, notification.id);
    
    Ok(())
}

// Command to assign a notification to a window
#[tauri::command]
pub fn assign_notification(
    app: AppHandle,
    window_id: String,
    notification_id: String,
    state: tauri::State<'_, NotificationManagerState>,
) -> Result<(), String> {
    // Assign notification to window
    {
        let mut manager = state.lock().unwrap();
        if !manager.assign_notification_to_window(&window_id, &notification_id) {
            return Err(format!("Failed to assign notification {} to window {}", notification_id, window_id));
        }
    }
    
    // Emit notification data to the window
    emit_notification_data_event(&app, &window_id, state.inner().clone())?;
    
    Ok(())
}



#[tauri::command]
pub fn notification_window_ready(
    app: AppHandle,
    window: tauri::Window,
    state: tauri::State<'_, NotificationManagerState>,
) -> Result<(), String> {
    let window_id = window.label().to_string();
    info!("Notification window ready: {}", window_id);

    // Check if there's already a notification assigned to this window
    let has_notification = {
        let manager = state.lock().unwrap();
        manager.get_notification_for_window(&window_id).is_some()
    };

    if has_notification {
        // If there's already a notification assigned, emit it
        emit_notification_data_event(&app, &window_id, state.inner().clone())?;
    } else {
        // Otherwise, try to assign the next notification in queue
        assign_next_notification_to_window(&app, &window_id, state.inner().clone())?;
    }

    Ok(())
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
    window_id: String,
    state: tauri::State<'_, NotificationManagerState>,
) -> Result<(), String> {
    info!("Closing notification in window: {}", window_id);
    
    // Remove notification from window
    let removed = {
        let mut manager = state.lock().unwrap();
        manager.remove_notification(&window_id)
    };
    
    if removed.is_some() {
        info!("Notification removed from window: {}", window_id);
        
        // Check if there are more notifications in the queue
        let has_more_notifications = {
            let manager = state.lock().unwrap();
            manager.get_next_unassigned_notification().is_some()
        };
        
        if has_more_notifications {
            // Assign next notification to this window
            assign_next_notification_to_window(&app, &window_id, state.inner().clone())?;
        } else {
            // No more notifications, reposition windows
            let mut manager = state.lock().unwrap();
            manager.reposition_notifications(&app);
        }
    } else {
        info!("No notification found for window: {}", window_id);
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
            create_notification,
            close_notification,
            notification_window_ready,
            get_window_size,
            get_scale_factor,
            assign_notification
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


