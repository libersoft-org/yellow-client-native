use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};
use log::info;

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
    info!("ping: {:?}", payload);
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
