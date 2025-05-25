use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Yellow;
#[cfg(mobile)]
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
      commands::request_file_permissions
    ])
    .setup(|app, api| {
      #[cfg(mobile)]
      let yellow = mobile::init(app, api)?;
      #[cfg(desktop)]
      let yellow = desktop::init(app, api)?;
      app.manage(yellow);
      Ok(())
    })
    .build()
}
