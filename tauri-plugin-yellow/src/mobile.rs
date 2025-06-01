use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_yellow);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<Yellow<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("org.libersoft.yellowplugin", "ExamplePlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_yellow)?;
  Ok(Yellow(handle))
}

/// Access to the yellow APIs.
pub struct Yellow<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Yellow<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    self
      .0
      .run_mobile_plugin("ping", payload)
      .map_err(Into::into)
  }

  pub fn check_permissions(&self) -> crate::Result<PermissionStatus> {
    self
      .0
      .run_mobile_plugin("checkPermissions", ())
      .map_err(Into::into)
  }

  pub fn request_permissions(
    &self,
    permissions: Option<Vec<PermissionType>>,
  ) -> crate::Result<PermissionStatus> {
    self
      .0
      .run_mobile_plugin(
        "requestPermissions",
        serde_json::json!({ "permissions": permissions }),
      )
      .map_err(Into::into)
  }
  
  pub fn save_to_downloads(
    &self,
    file_name: String,
    mime_type: String,
    data: String,
  ) -> crate::Result<serde_json::Value> {
    self
      .0
      .run_mobile_plugin(
        "saveToDownloads",
        serde_json::json!({ 
          "fileName": file_name,
          "mimeType": mime_type,
          "data": data 
        }),
      )
      .map_err(Into::into)
  }
  
  pub fn export_file_to_downloads(
    &self,
    file_path: String,
    file_name: String,
    mime_type: String,
  ) -> crate::Result<serde_json::Value> {
    self
      .0
      .run_mobile_plugin(
        "exportFileToDownloads",
        serde_json::json!({ 
          "filePath": file_path,
          "fileName": file_name,
          "mimeType": mime_type
        }),
      )
      .map_err(Into::into)
  }
  
  pub fn create_file(
    &self,
    file_name: String,
    content: String,
  ) -> crate::Result<serde_json::Value> {
    self
      .0
      .run_mobile_plugin(
        "createFile",
        serde_json::json!({ 
          "fileName": file_name,
          "content": content
        }),
      )
      .map_err(Into::into)
  }
  
  pub fn append_to_file(
    &self,
    file_name: String,
    data: String,
  ) -> crate::Result<serde_json::Value> {
    self
      .0
      .run_mobile_plugin(
        "appendToFile",
        serde_json::json!({ 
          "fileName": file_name,
          "data": data
        }),
      )
      .map_err(Into::into)
  }
  
  pub fn rename_file(
    &self,
    old_name: String,
    new_name: String,
  ) -> crate::Result<serde_json::Value> {
    self
      .0
      .run_mobile_plugin(
        "renameFile",
        serde_json::json!({ 
          "oldName": old_name,
          "newName": new_name
        }),
      )
      .map_err(Into::into)
  }
  
  pub fn delete_file(
    &self,
    file_name: String,
  ) -> crate::Result<serde_json::Value> {
    self
      .0
      .run_mobile_plugin(
        "deleteFile",
        serde_json::json!({ 
          "fileName": file_name
        }),
      )
      .map_err(Into::into)
  }
  
  pub fn file_exists(
    &self,
    file_name: String,
  ) -> crate::Result<bool> {
    self
      .0
      .run_mobile_plugin(
        "fileExists",
        serde_json::json!({ 
          "fileName": file_name
        }),
      )
      .map_err(Into::into)
  }
  
  pub fn open_save_dialog(
    &self,
    file_name: String,
    mime_type: String,
  ) -> crate::Result<serde_json::Value> {
    self
      .0
      .run_mobile_plugin(
        "openSaveDialog",
        serde_json::json!({ 
          "fileName": file_name,
          "mimeType": mime_type
        }),
      )
      .map_err(Into::into)
  }
  
  pub fn save_file_to_uri(
    &self,
    file_path: String,
    uri: String,
  ) -> crate::Result<serde_json::Value> {
    self
      .0
      .run_mobile_plugin(
        "saveFileToUri",
        serde_json::json!({ 
          "filePath": file_path,
          "uri": uri
        }),
      )
      .map_err(Into::into)
  }
}
