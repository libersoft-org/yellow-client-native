use crate::{
    NotificationManagerState, Notification, 
    create_notification_window, assign_next_notification_to_window,
    emit_notification_data_event, process_notification_queue
};
use tauri::{AppHandle, Window, State, Manager};
use log::{info, error};
use serde_json::json;

// Command to create a new notification
#[tauri::command]
pub async fn create_notification(
    app: AppHandle,
    title: String,
    message: String,
    duration: Option<u64>,
    notification_type: Option<String>,
    state: State<'_, NotificationManagerState>,
) -> Result<String, String> {
    let notification_id = uuid::Uuid::new_v4().to_string();
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
    process_notification_queue(&app, state.inner().clone())?;

    Ok(notification_id)
}

// Command to assign a notification to a window
#[tauri::command]
pub fn assign_notification(
    app: AppHandle,
    window_id: String,
    notification_id: String,
    state: State<'_, NotificationManagerState>,
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
    window: Window,
    state: State<'_, NotificationManagerState>,
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
pub fn get_window_size(window: Window) -> Result<(u32, u32), String> {
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
pub fn get_scale_factor(window: Window) -> Result<f64, String> {
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
    state: State<'_, NotificationManagerState>,
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
            // No more notifications to display, add window to pool instead of destroying it
            let should_pool = {
                let mut manager = state.lock().unwrap();
                // Only pool if we're under the max windows limit
                if manager.windows.len() <= crate::MAX_WINDOWS {
                    // Add to window pool for future reuse
                    let result = manager.pool_window(&window_id);
                    info!("Adding window {} to pool: {:?}", window_id, result.is_ok());
                    result.is_ok()
                } else {
                    info!("Not pooling window {} because we're at max capacity", window_id);
                    false
                }
            };
            
            if !should_pool {
                // If we didn't pool the window, reposition the remaining windows
                let mut manager = state.lock().unwrap();
                manager.reposition_notifications(&app);
            }
        }
    } else {
        info!("No notification found for window: {}", window_id);
    }
    
    Ok(())
}

// Command to get notification history
#[tauri::command]
pub fn get_notification_history(
    state: State<'_, NotificationManagerState>,
) -> Result<Vec<Notification>, String> {
    let manager = state.lock().unwrap();
    let history = manager.get_notification_history().to_vec();
    Ok(history)
}

// Command to get window pool status
#[tauri::command]
pub fn get_window_pool_status(
    state: State<'_, NotificationManagerState>,
) -> Result<serde_json::Value, String> {
    let manager = state.lock().unwrap();
    let status = json!({
        "pooled_windows": manager.window_pool.len(),
        "total_windows": manager.windows.len(),
        "max_windows": crate::MAX_WINDOWS,
        "window_ids": manager.window_pool.clone(),
    });
    Ok(status)
}
