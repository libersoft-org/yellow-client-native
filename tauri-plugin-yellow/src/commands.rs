use tauri::{AppHandle, command, Runtime};

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

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn export_file_to_downloads<R: Runtime>(
    app: AppHandle<R>,
    file_path: String,
    file_name: String,
    mime_type: String,
) -> Result<serde_json::Value> {
    // Pass the file path directly to Android to handle streaming
    app.yellow().export_file_to_downloads(file_path, file_name, mime_type)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn create_file<R: Runtime>(
    app: AppHandle<R>,
    file_name: String,
    content: String,
) -> Result<serde_json::Value> {
    app.yellow().create_file(file_name, content)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn append_to_file<R: Runtime>(
    app: AppHandle<R>,
    file_name: String,
    data: String,
) -> Result<serde_json::Value> {
    app.yellow().append_to_file(file_name, data)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn rename_file<R: Runtime>(
    app: AppHandle<R>,
    old_name: String,
    new_name: String,
) -> Result<serde_json::Value> {
    app.yellow().rename_file(old_name, new_name)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn delete_file<R: Runtime>(
    app: AppHandle<R>,
    file_name: String,
) -> Result<serde_json::Value> {
    app.yellow().delete_file(file_name)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn file_exists<R: Runtime>(
    app: AppHandle<R>,
    file_name: String,
) -> Result<bool> {
    app.yellow().file_exists(file_name)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn get_file_size<R: Runtime>(
    app: AppHandle<R>,
    file_name: String,
) -> Result<i64> {
    app.yellow().get_file_size(file_name)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn open_save_dialog<R: Runtime>(
    app: AppHandle<R>,
    file_name: String,
    mime_type: String,
) -> Result<serde_json::Value> {
    app.yellow().open_save_dialog(file_name, mime_type)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[command]
pub(crate) async fn save_file_to_uri<R: Runtime>(
    app: AppHandle<R>,
    file_path: String,
    uri: String,
) -> Result<serde_json::Value> {
    app.yellow().save_file_to_uri(file_path, uri)
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

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn export_file_to_downloads<R: Runtime>(
    _app: AppHandle<R>,
    _file_path: String,
    _file_name: String,
    _mime_type: String,
) -> Result<serde_json::Value> {
    // Desktop doesn't need this - use regular file system
    Err(crate::Error::String("export_file_to_downloads is only for mobile platforms".to_string()))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn create_file<R: Runtime>(
    _app: AppHandle<R>,
    _file_name: String,
    _content: String,
) -> Result<serde_json::Value> {
    Err(crate::Error::String("create_file is only for mobile platforms".to_string()))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn append_to_file<R: Runtime>(
    _app: AppHandle<R>,
    _file_name: String,
    _data: String,
) -> Result<serde_json::Value> {
    Err(crate::Error::String("append_to_file is only for mobile platforms".to_string()))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn rename_file<R: Runtime>(
    _app: AppHandle<R>,
    _old_name: String,
    _new_name: String,
) -> Result<serde_json::Value> {
    Err(crate::Error::String("rename_file is only for mobile platforms".to_string()))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn delete_file<R: Runtime>(
    _app: AppHandle<R>,
    _file_name: String,
) -> Result<serde_json::Value> {
    Err(crate::Error::String("delete_file is only for mobile platforms".to_string()))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn file_exists<R: Runtime>(
    _app: AppHandle<R>,
    _file_name: String,
) -> Result<bool> {
    Err(crate::Error::String("file_exists is only for mobile platforms".to_string()))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn get_file_size<R: Runtime>(
    _app: AppHandle<R>,
    _file_name: String,
) -> Result<i64> {
    Err(crate::Error::String("get_file_size is only for mobile platforms".to_string()))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn open_save_dialog<R: Runtime>(
    _app: AppHandle<R>,
    _file_name: String,
    _mime_type: String,
) -> Result<serde_json::Value> {
    Err(crate::Error::String("open_save_dialog is only for mobile platforms".to_string()))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[command]
pub(crate) async fn save_file_to_uri<R: Runtime>(
    _app: AppHandle<R>,
    _file_path: String,
    _uri: String,
) -> Result<serde_json::Value> {
    Err(crate::Error::String("save_file_to_uri is only for mobile platforms".to_string()))
}

