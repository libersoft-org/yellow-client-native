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

// Configuration for notification system
#[derive(Clone, Copy)]
pub struct NotificationConfig {
    pub max_history_size: usize,
    pub cleanup_interval_ms: u64,
    pub notification_width: u32,
    pub notification_height: u32,
    pub margin: u32,
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


