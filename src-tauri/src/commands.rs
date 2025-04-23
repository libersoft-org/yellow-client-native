use log::info;
use tauri::Window;
use monitor_work_area::{Area, get_work_area_tauri};

// Command to get window size
#[tauri::command]
pub fn get_window_size(window: Window) -> Result<(u32, u32), String> {
    match window.inner_size() {
        Ok(size) => {
            info!(
                "Window {} size: {}x{}",
                window.label(),
                size.width,
                size.height
            );
            Ok((size.width, size.height))
        }
        Err(e) => Err(format!("Failed to get window size: {}", e)),
    }
}

// Command to get the monitor's scale factor
#[tauri::command]
pub fn get_scale_factor(window: Window) -> Result<f64, String> {
    match window.scale_factor() {
        Ok(scale) => {
            info!("Window {} scale factor: {}", window.label(), scale);
            Ok(scale)
        }
        Err(e) => Err(format!("Failed to get scale factor: {}", e)),
    }
}

#[tauri::command]
pub fn log(message: String) {
    info!("{}", message);
}


#[tauri::command]
pub async fn get_work_area(monitor_name: String, window: tauri::Window) -> Result<Area, String> {
    let area = get_work_area_tauri(monitor_name, window)
        .await
        .map_err(|e| format!("Failed to get work area: {}", e));
    area
}
