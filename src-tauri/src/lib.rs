#![feature(str_as_str)]

mod commands;
mod notifications;

use std::sync::Arc;
use std::sync::Mutex;
use log::{LevelFilter, info, error};
use serde_json::Value;
use tauri::{AppHandle, Event, Listener, Emitter};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{PhysicalPosition, WebviewWindow};
use uuid::Uuid;
use std::collections::HashMap;
use notifications::create_notifications_window;
use serde::{Serialize, Deserialize};

// Maximum number of notification windows
const MAX_WINDOWS: usize = 5;

// Notification data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub icon: Option<String>,
    pub timestamp: Option<u64>,
    pub window_label: Option<String>,
    pub actions: Option<Vec<NotificationAction>>,
}

// Notification action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
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
    windows: HashMap<String, WebviewWindow>, // windowId -> WebviewWindow
    // Notifications waiting to be displayed
    notification_queue: Vec<Notification>,
    // Mapping of which notification is assigned to which window
    window_assignments: HashMap<String, String>, // windowId -> notification_id
    // Positions of windows
    positions: HashMap<String, NotificationPosition>, // windowId -> position
    // Window pool - windows that are hidden but ready to be reused
    // Instead of destroying windows when notifications are closed,
    // we hide them and keep them in this pool for future reuse
    window_pool: Vec<String>, // windowIds of pooled windows
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
        self.window_pool.retain(|windowId| {
            self.windows.contains_key(windowId)
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
        let windowId = window.label().to_string();
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
        self.windows.insert(windowId.clone(), window);
        self.positions.insert(windowId.clone(), position);
        
        windowId
    }
    
    // Get a window from the pool or create a new one
    // This is the main entry point for window recycling
    pub fn get_or_create_window(&mut self, app: &AppHandle) -> Result<String, String> {
        // First check if we have a window in the pool
        if let Some(windowId) = self.window_pool.pop() {
            info!("Reusing window from pool: {}", windowId);
            
            // Make sure the window is visible
            if let Some(window) = self.windows.get(&windowId) {
                // Show the hidden window
                window.show().map_err(|e| format!("Failed to show pooled window: {}", e))?;
                
                // Reset window state if needed
                window.set_focus().ok(); // Ignore errors
                
                return Ok(windowId);
            }
        }
        
        // If no pooled window available, create a new one
        create_notification_window(app, Arc::new(Mutex::new(self.clone())))
    }
    
    // Add a window to the pool instead of destroying it
    // This improves performance by reusing existing windows rather than
    // creating new ones for each notification
    pub fn pool_window(&mut self, windowId: &str) -> Result<(), String> {
        // Check if the window exists
        if let Some(window) = self.windows.get(windowId) {
            // Hide the window instead of closing it
            window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
            
            // Add to pool if not already there
            if !self.window_pool.contains(&windowId.to_string()) {
                self.window_pool.push(windowId.to_string());
                info!("Added window to pool: {}", windowId);
            }
            
            Ok(())
        } else {
            Err(format!("Window not found: {}", windowId))
        }
    }

    // Remove a notification from a window
    pub fn remove_notification(&mut self, windowId: &str) -> Option<Notification> {
        if let Some(notification_id) = self.window_assignments.remove(windowId) {
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
    pub fn get_notification_for_window(&self, windowId: &str) -> Option<&Notification> {
        if let Some(notification_id) = self.window_assignments.get(windowId) {
            self.notification_queue.iter().find(|n| &n.id == notification_id)
        } else {
            None
        }
    }

    // Get window by ID
    pub fn get_window(&self, windowId: &str) -> Option<&WebviewWindow> {
        self.windows.get(windowId)
    }

    // Assign a notification to a window
    pub fn assign_notification_to_window(&mut self, windowId: &str, notification_id: &str) -> bool {
        // Check if window exists
        if !self.windows.contains_key(windowId) {
            return false;
        }
        
        // Check if notification exists
        if !self.notification_queue.iter().any(|n| n.id == notification_id) {
            return false;
        }
        
        // Assign notification to window
        self.window_assignments.insert(windowId.to_string(), notification_id.to_string());
        
        // Update notification's window_label
        for notification in &mut self.notification_queue {
            if notification.id == notification_id {
                notification.window_label = Some(windowId.to_string());
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
        for windowId in self.windows.keys() {
            if !self.window_assignments.contains_key(windowId) {
                return Some(windowId.clone());
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
            } else if let Some(windowId) = manager.find_available_window() {
                // Use an existing window
                Some(("assign", windowId))
            } else {
                None
            }
        } else {
            None
        }
    };
    
    // Take action based on what we determined
    match action {
        Some((action_type, window_id)) => {
            match action_type {
                "pool" => {
                    // Use a window from the pool
                    assign_next_notification_to_window(app, &window_id, state.clone())?;
                },
                "create" => {
                    // Create a new window
                    create_notification_window(app, state.clone())?;
                },
                "assign" => {
                    // Assign to existing window
                    assign_next_notification_to_window(app, &window_id, state.clone())?;
                },
                _ => {}
            }
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
    let windowId = format!("notification-window-{}", Uuid::new_v4());
    
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
        windowId.clone(),
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

    info!("Created notification window: {}", windowId);
    
    // The window will call notification_window_ready when it's ready
    // and we'll assign a notification to it then
    
    Ok(windowId)
}

// Assign the next notification in queue to a window
fn assign_next_notification_to_window(
    app: &AppHandle,
    windowId: &str,
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
        if !manager.assign_notification_to_window(windowId, &notification_id) {
            return Err("Failed to assign notification to window".to_string());
        }
    }
    
    // Emit notification data to the window
    emit_notification_data_event(app, windowId, state.clone())?;
    
    Ok(())
}

// Emit notification data to a specific window
fn emit_notification_data_event(
    _app: &AppHandle,
    windowId: &str,
    state: NotificationManagerState,
) -> Result<(), String> {
    let notification = {
        let mut manager = state.lock().unwrap();
        
        // Get the notification assigned to this window
        let notification = manager.get_notification_for_window(windowId)
            .ok_or_else(|| format!("No notification assigned to window {}", windowId))?
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
        manager.get_window(windowId)
            .ok_or_else(|| format!("Window {} not found", windowId))?
            .clone()
    };
    
    // Emit the notification data to the window
    window.emit_to(windowId, "notification-data", &notification)
        .map_err(|e| format!("Failed to emit notification data: {}", e))?;
    
    info!("Emitted notification data to window {}: {:?}", windowId, notification.id);
    
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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Set up event listener for notification logs
            let log_handle = app_handle.clone();
            log_handle.listen("my-log", move |event| {
                let payload = event.payload();
                info!("my-log: {}", payload);
            });
            
            // Create the notifications window
            if let Err(e) = create_notifications_window(&app.handle()) {
                error!("Failed to create notifications window: {}", e);
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_window_size,
            commands::get_scale_factor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


