#![feature(str_as_str)]

mod commands;
mod notifications;

use log::{info, LevelFilter};
use tauri::{Listener, Manager};

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
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
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

            // Close notifications window when main window closes
            let app_handle_clone = app.handle().clone();
            let main_window = app.get_webview_window("main").unwrap();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    info!("Main window is closing, closing notifications window too");
                    if let Some(notifications_window) =
                        app_handle_clone.get_webview_window("notifications")
                    {
                        let _ = notifications_window.close();
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_window_size,
            commands::get_scale_factor,
            commands::log,
            notifications::close_notifications_window,
            notifications::create_notifications_window,
            notifications::show
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    info!("Tauri application started");
}
