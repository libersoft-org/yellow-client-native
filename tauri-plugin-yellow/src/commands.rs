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

#[command]
pub(crate) async fn check_file_permissions<R: Runtime>(
    app: AppHandle<R>,
) -> Result<CheckPermissionsResponse> {
    app.yellow().check_file_permissions()
}

#[command]
pub(crate) async fn request_file_permissions<R: Runtime>(
    app: AppHandle<R>,
) -> Result<RequestPermissionsResponse> {
    app.yellow().request_file_permissions()
}
