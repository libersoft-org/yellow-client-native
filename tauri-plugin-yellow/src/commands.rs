use tauri::{AppHandle, command, Runtime};
use serde::Deserialize;

use crate::models::*;
use crate::Result;
use crate::YellowExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.yellow().ping(payload)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn check_file_permissions<R: Runtime>(
    app: AppHandle<R>,
) -> Result<PermissionStatus> {
    app.yellow().check_permissions()
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn request_file_permissions<R: Runtime>(
    app: AppHandle<R>,
    permissions: Option<Vec<PermissionType>>,
) -> Result<PermissionStatus> {
    app.yellow().request_permissions(permissions)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn check_file_permissions<R: Runtime>(
    _app: AppHandle<R>,
) -> Result<PermissionStatus> {
    // Desktop always has permissions
    Ok(PermissionStatus {
        write_external_storage: tauri::plugin::PermissionState::Granted,
        read_external_storage: tauri::plugin::PermissionState::Granted,
    })
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn request_file_permissions<R: Runtime>(
    _app: AppHandle<R>,
    _permissions: Option<Vec<PermissionType>>,
) -> Result<PermissionStatus> {
    // Desktop always has permissions
    Ok(PermissionStatus {
        write_external_storage: tauri::plugin::PermissionState::Granted,
        read_external_storage: tauri::plugin::PermissionState::Granted,
    })
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn save_to_downloads<R: Runtime>(
    app: AppHandle<R>,
    file_name: String,
    mime_type: String,
    data: String,
) -> Result<serde_json::Value> {
    app.yellow().save_to_downloads(file_name, mime_type, data)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn save_to_downloads<R: Runtime>(
    _app: AppHandle<R>,
    _file_name: String,
    _mime_type: String,
    _data: String,
) -> Result<serde_json::Value> {
    // Desktop doesn't need this - use regular file system
    Err(crate::Error::String("save_to_downloads is only for mobile platforms".to_string()))
}
