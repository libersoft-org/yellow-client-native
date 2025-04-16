use log::info;

#[cfg(not(target_os = "android"))]
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

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
        );

        #[cfg(not(target_os = "macos"))]
        let _notifications_window2 = _notifications_window.transparent(true);
        #[cfg(target_os = "macos")]
        let _notifications_window2 = _notifications_window;

        _notifications_window2
        .title("Yellow Notifications")
        .inner_size(400.0, 60.0)
        .decorations(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .resizable(true)
        .shadow(false)
        //.visible(false)
        .focused(false)
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
    info!("Closing notifications window");

    #[cfg(not(target_os = "android"))]
    {
        // Close the notifications window if it exists
        if let Some(window) = _app.get_webview_window("notifications") {
            window
                .close()
                .map_err(|e| format!("Failed to close notifications window: {}", e))?;
            info!("Notifications window closed successfully");
        } else {
            info!("Notifications window not found");
        }
    }

    #[cfg(target_os = "android")]
    {
        info!("Notifications window not supported on Android");
    }

    Ok(())
}

#[tauri::command]
pub fn show(_window: tauri::Window) -> Result<(), String> {
    info!("show...");

    #[cfg(not(target_os = "android"))]
    {
        _window
            .show()
            .map_err(|e| format!("Failed to show window: {}", e))?;
    }

    #[cfg(target_os = "android")]
    {
        info!("Window show method not used on Android");
    }

    Ok(())
}
