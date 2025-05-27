use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<Yellow<R>> {
  Ok(Yellow(app.clone()))
}

/// Access to the yellow APIs.
pub struct Yellow<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Yellow<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }

  pub fn check_permissions(&self) -> crate::Result<PermissionStatus> {
    // Desktop platforms don't need special file permissions
    Ok(PermissionStatus {
      write_external_storage: tauri::plugin::PermissionState::Granted,
      read_external_storage: tauri::plugin::PermissionState::Granted,
    })
  }

  pub fn request_permissions(
    &self,
    _permissions: Option<Vec<PermissionType>>,
  ) -> crate::Result<PermissionStatus> {
    // Desktop platforms don't need special file permissions
    Ok(PermissionStatus {
      write_external_storage: tauri::plugin::PermissionState::Granted,
      read_external_storage: tauri::plugin::PermissionState::Granted,
    })
  }
}
