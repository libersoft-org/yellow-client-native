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
}
