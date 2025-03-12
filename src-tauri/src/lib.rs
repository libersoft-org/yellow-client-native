#![feature(str_as_str)]

mod commands;
mod notification;

use std::sync::Arc;
use std::sync::Mutex;
use log::{LevelFilter, info, error};
use serde_json::Value;
use tauri::{AppHandle, Event, Listener, Manager, Emitter};
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

// Configuration for notification system
#[derive(Clone, Copy)]
pub struct NotificationConfig {
    pub max_history_size: usize,
    pub cleanup_interval_ms: u64,
    pub notification_width: u32,
    pub notification_height: u32,
    pub margin: u32,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            max_history_size: 100,
            cleanup_interval_ms: 60000, // 1 minute
            notification_width: 400,
            notification_height: 500,
            margin: 10,
        }
    }
}

// Notification manager to keep track of active notifications and windows
#[derive(Clone)]
pub struct NotificationManager {
    // Windows available for displaying notifications
    windows: HashMap<String, WebviewWindow>, // window_id -> WebviewWindow
    // Notifications waiting to be displayed
    notification_queue: Vec<Notification>,
    // Mapping of which notification is assigned to which window
    window_assignments: HashMap<String, String>, // window_id -> notification_id
    // Positions of windows
    positions: HashMap<String, NotificationPosition>, // window_id -> position
    // Window pool - windows that are hidden but ready to be reused
    // Instead of destroying windows when notifications are closed,
    // we hide them and keep them in this pool for future reuse
    window_pool: Vec<String>, // window_ids of pooled windows
    // History of displayed notifications
    notification_history: Vec<Notification>,
    // Configuration
    config: NotificationConfig,
    // Internal state
    next_position_id: usize,
    last_cleanup: u64,
}

impl NotificationManager {
    pub fn new() -> Self {
        let config = NotificationConfig::default();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        NotificationManager {
            windows: HashMap::new(),
            notification_queue: Vec::new(),
            window_assignments: HashMap::new(),
            positions: HashMap::new(),
            window_pool: Vec::new(),
            notification_history: Vec::new(),
            config,
            next_position_id: 0,
            last_cleanup: now,
        }
    }
    
    // Configure the notification manager
    pub fn configure(&mut self, config: NotificationConfig) {
        self.config = config;
    }

    // Add a new notification to the queue
    pub fn add_notification(&mut self, notification: Notification) {
        // Check if we need to run cleanup
        self.maybe_cleanup();
        
        // Add to queue
        self.notification_queue.push(notification);
    }
    
    // Run cleanup if needed
    fn maybe_cleanup(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        // Check if it's time to clean up
        if now - self.last_cleanup > self.config.cleanup_interval_ms {
            self.cleanup();
            self.last_cleanup = now;
        }
    }
    
    // Clean up old notifications and manage window pool
    fn cleanup(&mut self) {
        info!("Running notification cleanup");
        
        // Trim notification history to max size
        if self.notification_history.len() > self.config.max_history_size {
            let excess = self.notification_history.len() - self.config.max_history_size;
            self.notification_history.drain(0..excess);
            info!("Removed {} old notifications from history", excess);
        }
        
        // Clean up any stale window pool entries
        self.window_pool.retain(|window_id| {
            self.windows.contains_key(window_id)
        });
        
        // Remove any notifications that are too old (over 24 hours)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let day_in_seconds = 24 * 60 * 60;
        self.notification_queue.retain(|notification| {
            if let Some(timestamp) = notification.timestamp {
                // Keep if less than 24 hours old
                now - timestamp < day_in_seconds
            } else {
                // Keep if no timestamp (not displayed yet)
                true
            }
        });
    }

    // Register a window that can display notifications
    pub fn register_window(&mut self, window: WebviewWindow, x: u32, y: u32, actual_height: Option<u32>) -> String {
        let window_id = window.label().to_string();
        let height = actual_height.unwrap_or(self.config.notification_height);
        
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
    
    // Get a window from the pool or create a new one
    // This is the main entry point for window recycling
    pub fn get_or_create_window(&mut self, app: &AppHandle) -> Result<String, String> {
        // First check if we have a window in the pool
        if let Some(window_id) = self.window_pool.pop() {
            info!("Reusing window from pool: {}", window_id);
            
            // Make sure the window is visible
            if let Some(window) = self.windows.get(&window_id) {
                // Show the hidden window
                window.show().map_err(|e| format!("Failed to show pooled window: {}", e))?;
                
                // Reset window state if needed
                window.set_focus().ok(); // Ignore errors
                
                return Ok(window_id);
            }
        }
        
        // If no pooled window available, create a new one
        create_notification_window(app, Arc::new(Mutex::new(self.clone())))
    }
    
    // Add a window to the pool instead of destroying it
    // This improves performance by reusing existing windows rather than
    // creating new ones for each notification
    pub fn pool_window(&mut self, window_id: &str) -> Result<(), String> {
        // Check if the window exists
        if let Some(window) = self.windows.get(window_id) {
            // Hide the window instead of closing it
            window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
            
            // Add to pool if not already there
            if !self.window_pool.contains(&window_id.to_string()) {
                self.window_pool.push(window_id.to_string());
                info!("Added window to pool: {}", window_id);
            }
            
            Ok(())
        } else {
            Err(format!("Window not found: {}", window_id))
        }
    }

    // Remove a notification from a window
    pub fn remove_notification(&mut self, window_id: &str) -> Option<Notification> {
        if let Some(notification_id) = self.window_assignments.remove(window_id) {
            // Find and remove the notification from the queue if it's there
            if let Some(pos) = self.notification_queue.iter().position(|n| n.id == notification_id) {
                let notification = self.notification_queue.remove(pos);
                
                // Add to history
                self.notification_history.push(notification.clone());
                
                // Check if we need cleanup
                self.maybe_cleanup();
                
                return Some(notification);
            }
        }
        None
    }
    
    // Get notification history
    pub fn get_notification_history(&self) -> &[Notification] {
        &self.notification_history
    }

    // Get the next available position for a notification window
    pub fn get_next_position(&self, screen_width: u32) -> (u32, u32) {
        // Start from top right corner
        let base_x = screen_width - self.config.notification_width - 20; // 20px margin from right
        let base_y = 20; // Start 20px from top

        info!("base_x: {}, base_y: {}, notification_width: {}, notification_height: {}", 
              base_x, base_y, self.config.notification_width, self.config.notification_height);

        if self.positions.is_empty() {
            return (base_x, base_y);
        }

        // Find the lowest position (highest y value)
        let max_y = self.positions.values()
            .map(|pos| pos.y + pos.height + self.config.margin)
            .max()
            .unwrap_or(base_y);

        info!("max_y: {}", max_y);

        (base_x, max_y)
    }

    // Reposition all notification windows
    pub fn reposition_notifications(&mut self, _app_handle: &AppHandle) {
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
                current_y += height + self.config.margin;
            }
        }
    }

    // Get window dimensions
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.config.notification_width, self.config.notification_height)
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


// Process the notification queue, creating windows as needed
fn process_notification_queue(
    app: &AppHandle,
    state: NotificationManagerState,
) -> Result<(), String> {
    let action = {
        let mut manager = state.lock().unwrap();
        
        // Run cleanup if needed
        manager.maybe_cleanup();
        
        // If we have unassigned notifications
        if manager.get_next_unassigned_notification().is_some() {
            // First check if we have a pooled window
            if !manager.window_pool.is_empty() {
                // Use a pooled window
                Some(("pool", manager.window_pool[0].clone()))
            } else if manager.has_available_window_slots() {
                // Create a new window
                Some(("create", String::new()))
            } else if let Some(window_id) = manager.find_available_window() {
                // Use an existing window
                Some(("assign", window_id))
            } else {
                None
            }
        } else {
            None
        }
    };
    
    // Take action based on what we determined
    match action {
        Some(("pool", window_id)) => {
            // Use a window from the pool
            assign_next_notification_to_window(app, &window_id, state.clone())?;
        },
        Some(("create", _)) => {
            // Create a new window
            create_notification_window(app, state.clone())?;
        },
        Some(("assign", window_id)) => {
            // Assign to existing window
            assign_next_notification_to_window(app, &window_id, state.clone())?;
        },
        _ => {}
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
    window.emit_to(window_id, "notification-data", &notification)
        .map_err(|e| format!("Failed to emit notification data: {}", e))?;
    
    info!("Emitted notification data to window {}: {:?}", window_id, notification.id);
    
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
    let notification_manager = Arc::new(Mutex::new(NotificationManager::new()));
    
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
            let _click_handle = app_handle.clone();
            let _click_app_handle = app_handle.clone(); // Clone for use inside closure
            
            // Helper closure to handle notification close events
            let _handle_notification_close = |event: Event, action: &str, app_handle: &AppHandle| {
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
                            let state = app_handle.state::<NotificationManagerState>();
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
                                let state = app_handle.state::<NotificationManagerState>();
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
            commands::create_notification,
            commands::close_notification,
            commands::notification_window_ready,
            commands::get_window_size,
            commands::get_scale_factor,
            commands::assign_notification,
            commands::get_notification_history,
            commands::get_window_pool_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


