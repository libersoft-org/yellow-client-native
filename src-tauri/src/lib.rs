mod audio;
mod commands;
mod notifications;

use log::{info, LevelFilter};
use tauri::Listener;

#[cfg(desktop)]
use tauri::Manager;

use serde::Deserialize;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_sentry::{minidump, sentry};

// Define the plugin config
#[derive(Deserialize)]
struct Config {}

// Initialize logging
fn setup_logging() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        //add milliseconds to the logs
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .init();
    info!("Logging initialized");
}

#[cfg(desktop)]
fn setup_desktop_notifications(_app: &mut tauri::App) {
    // todo: ensure that notifications window is closed when main window closes, even if the js in main window doesn't call close_notifications_window
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let client = sentry::init(("https://ba775427faea759b72f912167c326660@o4504414737399808.ingest.us.sentry.io/4506954859610112",
                               sentry::ClientOptions {
                                   release: sentry::release_name!(),
                                   auto_session_tracking: true,
                                   attach_stacktrace: true,
                                   trim_backtraces: false,
                                   ..Default::default()
                               },
    ));
    // Caution! Everything before here runs in both app and crash reporter processes
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let _guard = minidump::init(&client);
    // Everything after here runs in only the app process

    setup_logging();
    info!("Starting application");

    #[cfg(desktop)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        //.plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_sentry::init(&client))
        .plugin(tauri_plugin_positioner::init());

    #[cfg(not(desktop))]
    let builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::MacosLauncher;

        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                let _ = app
                    .get_webview_window("main")
                    .expect("no main window")
                    .set_focus();
            }))
            .plugin(tauri_plugin_autostart::init(
                MacosLauncher::LaunchAgent,
                Some(vec!["--flag1", "--flag2"]),
            ))
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_positioner::init())
    }

    info!("Tauri application starting");
    info!("thread id: {:?}", std::thread::current().id());
    info!("thread name: {:?}", std::thread::current().name());

    builder
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Set up event listener for notification logs
            let log_handle = app_handle.clone();
            log_handle.listen("my-log", move |event| {
                let payload = event.payload();
                info!("my-log: {}", payload);
            });

            // Close notifications window when main window closes
            #[cfg(desktop)]
            setup_desktop_notifications(app);

            #[cfg(debug_assertions)]
            let do_open_devtools = std::env::var("TAURI_OPEN_DEVTOOLS")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);

            #[cfg(debug_assertions)]
            if do_open_devtools {
                app.get_webview_window("main").unwrap().open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_window_size,
            commands::get_scale_factor,
            commands::log,
            #[cfg(desktop)]
            commands::get_work_area,
            commands::get_build_commit_hash,
            commands::get_build_ts,
            notifications::close_notifications_window,
            notifications::create_notifications_window,
            notifications::show,
            audio::play_audio,
            audio::stop_audio,
            audio::is_audio_playing
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

