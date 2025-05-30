use log::info;

#[cfg(not(target_os = "android"))]
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
#[cfg(not(target_os = "android"))]
use crate::misc;

#[tauri::command]
pub async fn create_notifications_window(_app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    {
        let app2 = _app.clone();
        let w = app2.get_webview_window("notifications");
        if w.is_some() {
            //info!("Notifications window already exists");
            return Ok(());
        }

        //info!("Creating notifications window");

        let _notifications_window = WebviewWindowBuilder::new(
            &_app,
            "notifications",
            WebviewUrl::App("/notifications".into()),
        )
        .initialization_script(&super::misc::get_error_handler_script());

        #[cfg(not(target_os = "macos"))]
        // "Note that on `macOS` this requires the `macos-private-api` feature flag, enabled under `tauri.conf.json > app > macOSPrivateApi`".
        let _notifications_window2 = _notifications_window.transparent(true);
        #[cfg(target_os = "macos")]
        let _notifications_window2 = _notifications_window;

        let _notifications_window3 = _notifications_window2
            .title("Yellow Notifications")
            .inner_size(400.0, 60.0)
            .decorations(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .shadow(false)
            .focused(false)
            .initialization_script(&misc::get_error_handler_script());

        #[cfg(dev)]
        let _notifications_window4 = _notifications_window3.resizable(true);
        #[cfg(not(dev))]
        let _notifications_window4 = _notifications_window3.visible(false);

        _notifications_window4
            .build()
            .map_err(|e| format!("Failed to create notifications window: {}", e))?;

        info!("Notifications window created successfully");
    }

    #[cfg(target_os = "android")]
    {
        info!("Custom notifications window not supported on Android");
    }

    Ok(())
}

#[tauri::command]
pub fn close_notifications_window(_app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    {

        info!("Closing notifications window");

        // Close the notifications window if it exists
        if let Some(window) = _app.get_webview_window("notifications") {
            window
                .hide()
                .map_err(|e| format!("Failed to close notifications window: {}", e))?;
            info!("Notifications window closed successfully");
        } else {
            info!("Notifications window not found");
        }
    }
    Ok(())
}

#[tauri::command]
pub fn show_notifications_window(_app: tauri::AppHandle) -> Result<(), String> {
    info!("show...");

    #[cfg(not(target_os = "android"))]
    {
        if let Some(window) = _app.get_webview_window("notifications") {
            window
                .show()
                .map_err(|e| format!("Failed to show  notifications window: {}", e))?;
            info!("Notifications window shown successfully");
        } else {
            info!("Notifications window not found");
        }
    }

    Ok(())
}


#[tauri::command]
pub fn hide_notifications_window(_app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    {
        info!("Hiding notifications window");

        if let Some(window) = _app.get_webview_window("notifications") {
            window
                .hide()
                .map_err(|e| format!("Failed to hide notifications window: {}", e))?;
            info!("Notifications window hidden successfully");
        } else {
            info!("Notifications window not found");
        }
    }

    Ok(())
}