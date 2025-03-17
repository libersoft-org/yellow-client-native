use log::info;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
pub fn create_notifications_window(app: tauri::AppHandle) -> Result<(), String> {

    info!("create_notifications_window...");

    let app2 = app.clone();
    let w = app2.get_webview_window("notifications");
    if w.is_some() {
        info!("Notifications window already exists");
        return Ok(());
    }

    info!("Creating notifications window");

    let _notifications_window = WebviewWindowBuilder::new(
        app,
        "notifications",
        WebviewUrl::App("/notification".into()),
    )
    .title("Notifications")
    .inner_size(400.0, 600.0)
    .decorations(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .transparent(true)
    .build()
    .map_err(|e| format!("Failed to create notifications window: {}", e))?;
    info!("Notifications window created successfully");
    Ok(())
}

#[tauri::command]
pub fn close_notifications_window(app: tauri::AppHandle) -> Result<(), String> {
    info!("Closing notifications window");

    // Close the notifications window
    app.get_webview_window("notifications")
        .ok_or_else(|| "Notifications window not found".to_string())?
        .close()
        .map_err(|e| format!("Failed to close notifications window: {}", e))?;

    info!("Notifications window closed successfully");

    Ok(())
}
