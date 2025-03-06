use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};
use uuid::Uuid;

// Notification data structure
#[derive(Clone, serde::Serialize)]
struct Notification {
    id: String,
    title: String,
    message: String,
    duration: u64, // in seconds
}

// Position information for a notification
#[derive(Clone, Copy)]
struct NotificationPosition {
    id: usize,  // Unique position ID
    x: u32,
    y: u32,
    height: u32,
}

// Notification manager to keep track of active notifications
struct NotificationManager {
    notifications: HashMap<String, WebviewWindow>,
    positions: HashMap<String, NotificationPosition>, // Map notification ID to position
    next_position_id: usize,
    notification_width: u32,
    notification_height: u32,
    margin: u32,
}

impl NotificationManager {
    fn new() -> Self {
        NotificationManager {
            notifications: HashMap::new(),
            positions: HashMap::new(),
            next_position_id: 0,
            notification_width: 300,
            notification_height: 100,
            margin: 10,
        }
    }

    fn add_notification(&mut self, window: WebviewWindow, x: u32, y: u32) {
        let id = window.label().to_string();
        let position = NotificationPosition {
            id: self.next_position_id,
            x,
            y,
            height: self.notification_height,
        };
        self.next_position_id += 1;
        self.notifications.insert(id.clone(), window);
        self.positions.insert(id, position);
    }

    fn remove_notification(&mut self, id: &str) -> Option<NotificationPosition> {
        self.notifications.remove(id);
        self.positions.remove(id)
    }

    fn get_next_position(&self, screen_width: u32) -> (u32, u32) {
        // Start from top right corner
        let base_x = screen_width - self.notification_width - 20; // 20px margin from right
        let base_y = 20; // Start 20px from top
        
        if self.positions.is_empty() {
            return (base_x, base_y);
        }
        
        // Find the lowest position (highest y value)
        let max_y = self.positions.values()
            .map(|pos| pos.y + pos.height + self.margin)
            .max()
            .unwrap_or(base_y);
            
        (base_x, max_y)
    }

    fn reposition_notifications(&mut self, app_handle: &AppHandle) {
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
                current_y += height + self.margin;
            }
        }
    }
}

// Create a global notification manager
type NotificationManagerState = Arc<Mutex<NotificationManager>>;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Command to create a new notification
#[tauri::command]
async fn create_notification(
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
    };
    
    // Serialize notification data for the window
    let notification_data = serde_json::to_string(&notification)
        .map_err(|e| format!("Failed to serialize notification: {}", e))?;
    
    // Get primary monitor dimensions
    let monitor = app.primary_monitor()
        .map_err(|e| format!("Failed to get primary monitor: {}", e))?
        .ok_or_else(|| "No primary monitor found".to_string())?;
    
    let monitor_size = monitor.size();
    
    // Get position for the notification from the manager
    let (notification_width, notification_height) = {
        let manager = state.lock().unwrap();
        (manager.notification_width, manager.notification_height)
    };
    
    // Calculate position for the notification
    let (x, y) = {
        let manager = state.lock().unwrap();
        manager.get_next_position(monitor_size.width)
    };
    
    // Create a new window for the notification
    let notification_window = tauri::WebviewWindowBuilder::new(
        &app,
        notification_id.clone(),
        tauri::WebviewUrl::App("notification.html".into())
    )
    .title("Notification")
    .inner_size(notification_width as f64, notification_height as f64)
    .decorations(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .build()
    .map_err(|e| format!("Failed to create notification window: {}", e))?;
    
    // Center the window
    notification_window.center()
        .map_err(|e| format!("Failed to center notification window: {}", e))?;
    
    // Position the window
    notification_window.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| format!("Failed to position notification window: {}", e))?;
    
    // Send notification data to the window
    notification_window.emit("notification-data", notification_data)
        .map_err(|e| format!("Failed to send notification  {}", e))?;
    
    // Add to notification manager
    {
        let mut manager = state.lock().unwrap();
        manager.add_notification(notification_window.clone(), x, y);
    }
    
    // Set up auto-close timer
    let notification_id_clone = notification_id.clone();
    let app_handle = app.clone();
    let state_clone = Arc::clone(&state.inner());
    
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(duration));
        
        // Close the notification after duration
        if let Some(window) = app_handle.get_webview_window(&notification_id_clone) {
            let _ = window.close();
        }
        
        // Remove from manager
        let mut manager = state_clone.lock().unwrap();
        if manager.remove_notification(&notification_id_clone).is_some() {
            manager.reposition_notifications(&app_handle);
        }
    });
    
    Ok(notification_id)
}

// Command to close a notification manually
#[tauri::command]
fn close_notification(
    app: AppHandle,
    notification_id: String,
    state: tauri::State<'_, NotificationManagerState>,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&notification_id) {
        window.close().map_err(|e| format!("Failed to close notification: {}", e))?;
    }
    
    // Remove from manager
    let mut manager = state.lock().unwrap();
    if manager.remove_notification(&notification_id).is_some() {
        manager.reposition_notifications(&app);
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let notification_manager = Arc::new(Mutex::new(NotificationManager::new()));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(notification_manager)
        .invoke_handler(tauri::generate_handler![
            greet,
            create_notification,
            close_notification
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
