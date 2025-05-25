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

  pub fn check_file_permissions(&self) -> crate::Result<CheckPermissionsResponse> {
    self
      .0
      .run_mobile_plugin("checkFilePermissions", ())
      .map_err(Into::into)
  }

  pub fn request_file_permissions(&self) -> crate::Result<RequestPermissionsResponse> {
    self
      .0
      .run_mobile_plugin("requestFilePermissions", ())
      .map_err(Into::into)
  }
}
