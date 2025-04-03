#![feature(str_as_str)]

mod commands;
mod notifications;

use log::{info, LevelFilter};
use tauri::Listener;

#[cfg(desktop)]
use tauri::Manager;

use serde::Deserialize;
use tauri::plugin::{Builder, TauriPlugin};
use tauri::Runtime;

// Define the plugin config
#[derive(Deserialize)]
struct Config {
    timeout: usize,
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

#[cfg(desktop)]
fn setup_desktop_notifications(app: &mut tauri::App) {
    let app_handle_clone = app.handle().clone();
    let main_window = app.get_webview_window("main").unwrap();
    main_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            info!("Main window is closing, closing notifications window too");
            if let Some(notifications_window) = app_handle_clone.get_webview_window("notifications")
            {
                info!("found, closing notifications window");
                let _ = notifications_window.close();
                info!("notifications window closed");
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_logging();

    info!("Starting application");

    #[cfg(desktop)]
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_log::Builder::new().build());

    #[cfg(not(desktop))]
    let builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
    }

    builder
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
            #[cfg(desktop)]
            setup_desktop_notifications(app);

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
