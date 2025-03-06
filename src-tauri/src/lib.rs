use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, Window};
use uuid::Uuid;

// Notification data structure
#[derive(Clone, serde::Serialize)]
struct Notification {
    id: String,
    title: String,
    message: String,
    duration: u64, // in seconds
}

// Notification manager to keep track of active notifications
struct NotificationManager {
    notifications: HashMap<String, Window>,
    positions: Vec<(u32, u32)>, // (x, y) positions that are currently occupied
}

impl NotificationManager {
    fn new() -> Self {
        NotificationManager {
            notifications: HashMap::new(),
            positions: Vec::new(),
        }
    }

    fn add_notification(&mut self, window: Window, position: (u32, u32)) {
        let id = window.label().to_string();
        self.notifications.insert(id, window);
        self.positions.push(position);
    }

    fn remove_notification(&mut self, id: &str) -> Option<(u32, u32)> {
        if let Some(window) = self.notifications.remove(id) {
            // Find the position of this notification
            if let Some(index) = self.positions.iter().position(|&pos| {
                // We would need to store position with notification ID to make this accurate
                // This is a simplification
                true
            }) {
                let position = self.positions.remove(index);
                return Some(position);
            }
        }
        None
    }

    fn reposition_notifications(&mut self) {
        // This would reposition remaining notifications after one is closed
        // Implementation depends on your specific UI requirements
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
    
    // Calculate position for the notification
    // Start from top right corner
    let notification_width = 300;
    let notification_height = 100;
    let mut x = monitor_size.width - notification_width - 20; // 20px margin from right
    let mut y = 20; // Start 20px from top
    
    // Check existing notifications and stack this one below
    {
        let manager = state.lock().unwrap();
        y += (manager.positions.len() as u32) * (notification_height + 10);
    }
    
    // Create a new window for the notification
    let notification_window = tauri::WindowBuilder::new(
        &app,
        notification_id.clone(),
        tauri::WindowUrl::App("notification.html".into())
    )
    .title("Notification")
    .inner_size(notification_width, notification_height)
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
        manager.add_notification(notification_window.clone(), (x, y));
    }
    
    // Set up auto-close timer
    let notification_id_clone = notification_id.clone();
    let app_handle = app.clone();
    let state_clone = state.clone();
    
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(duration));
        
        // Close the notification after duration
        if let Some(window) = app_handle.get_window(&notification_id_clone) {
            let _ = window.close();
        }
        
        // Remove from manager
        let mut manager = state_clone.lock().unwrap();
        if let Some(_) = manager.remove_notification(&notification_id_clone) {
            manager.reposition_notifications();
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
    if let Some(window) = app.get_window(&notification_id) {
        window.close().map_err(|e| format!("Failed to close notification: {}", e))?;
    }
    
    // Remove from manager
    let mut manager = state.lock().unwrap();
    if let Some(_) = manager.remove_notification(&notification_id) {
        manager.reposition_notifications();
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
