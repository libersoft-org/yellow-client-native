use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod desktop;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use desktop::Yellow;
#[cfg(any(target_os = "android", target_os = "ios"))]
use mobile::Yellow;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the yellow APIs.
pub trait YellowExt<R: Runtime> {
  fn yellow(&self) -> &Yellow<R>;
}

impl<R: Runtime, T: Manager<R>> crate::YellowExt<R> for T {
  fn yellow(&self) -> &Yellow<R> {
    self.state::<Yellow<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("yellow")
    .invoke_handler(tauri::generate_handler![
      commands::ping,
      commands::check_file_permissions,
      commands::request_file_permissions,
      commands::save_to_downloads
    ])
    .setup(|app, api| {
      #[cfg(any(target_os = "android", target_os = "ios"))]
      let yellow = mobile::init(app, api)?;
      #[cfg(not(any(target_os = "android", target_os = "ios")))]
      let yellow = desktop::init(app, api)?;
      app.manage(yellow);
      Ok(())
    })
    .build()
}
