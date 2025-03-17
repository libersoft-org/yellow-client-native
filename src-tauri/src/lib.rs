#![feature(str_as_str)]

mod commands;
mod notifications;

use log::{error, info, LevelFilter};
use notifications::create_notifications_window;
use tauri::Listener;

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
        .plugin(tauri_plugin_store::Builder::new().build())
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
            if let Err(e) = create_notifications_window(app.handle().clone()) {
                error!("Failed to create notifications window: {}", e);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_window_size,
            commands::get_scale_factor,
            commands::log,
            notifications::close_notifications_window,
            notifications::create_notifications_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
