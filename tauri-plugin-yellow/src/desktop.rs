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

  pub fn check_file_permissions(&self) -> crate::Result<CheckPermissionsResponse> {
    // Desktop platforms don't need special file permissions
    Ok(CheckPermissionsResponse {
      write_external_storage: "granted".to_string(),
    })
  }

  pub fn request_file_permissions(&self) -> crate::Result<RequestPermissionsResponse> {
    // Desktop platforms don't need special file permissions
    Ok(RequestPermissionsResponse {
      write_external_storage: "granted".to_string(),
    })
  }
}
