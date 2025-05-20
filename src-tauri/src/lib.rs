mod audio;
mod commands;
mod misc;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod notifications;

use log::{info, LevelFilter};
use tauri::Listener;

#[cfg(desktop)]
use tauri::Manager;

use tauri::{WebviewUrl, WebviewWindowBuilder};

use serde::Deserialize;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_sentry::{minidump, sentry};

// Define the plugin config
#[derive(Deserialize)]
struct Config {}

// Initialize logging
fn setup_logging() {
    #[cfg(target_os = "android")]
    {
        use android_logger::Config as AndroidConfig;
        android_logger::init_once(
            AndroidConfig::default()
                .with_max_level(LevelFilter::Info)
                .with_tag("YellowApp")
        );
        
        // Log several messages at different levels for testing
        log::trace!("TRACE: Android logging initialized with YellowApp tag");
        log::debug!("DEBUG: Android logging initialized with YellowApp tag");
        log::info!("INFO: Android logging initialized with YellowApp tag");
        log::warn!("WARN: Android logging initialized with YellowApp tag");
        log::error!("ERROR: Android logging initialized with YellowApp tag");
    }

    #[cfg(not(target_os = "android"))]
    {
        env_logger::Builder::new()
            .filter_level(LevelFilter::Debug)
            //add milliseconds to the logs
            .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
            .init();
        info!("Desktop logging initialized");
    }
}




#[cfg(desktop)]
fn setup_desktop_notifications(_app: &mut tauri::App) {
    // todo: ensure that notifications window is closed when main window closes, even if the js in main window doesn't call close_notifications_window
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set up a simplified panic hook on Android to avoid thread issues
    #[cfg(target_os = "android")]
    {
        use std::panic;
        panic::set_hook(Box::new(|panic_info| {
            if let Some(location) = panic_info.location() {
                log::error!(
                    "PANIC: at {}:{}: {}",
                    location.file(),
                    location.line(),
                    panic_info
                );
            } else {
                log::error!("PANIC: {}", panic_info);
            }
        }));
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let client = sentry::init(("https://3d18b31f479eb4d197cf54e7ef5c4291@o4509327469772800.ingest.de.sentry.io/4509327534981200",
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

    // Print Android-specific info for debugging
    #[cfg(target_os = "android")]
    {
        info!("Running on Android");
        info!("Android environment info:");
        if let Ok(home) = std::env::var("HOME") {
            info!("HOME: {}", home);
        }
        if let Ok(path) = std::env::var("PATH") {
            info!("PATH: {}", path);
        }
        info!("Current dir: {:?}", std::env::current_dir());

        // Try to manually load the C++ standard library (only when cfg flag is enabled)
        #[cfg(all(feature = "manual_cxx_lib", target_arch = "x86_64"))]
        unsafe {
            use std::ffi::CString;
            info!("Attempting to load libc++_shared.so for x86_64");
            let lib_path = CString::new("/system/lib64/libc++_shared.so").unwrap();
            let result = libc::dlopen(lib_path.as_ptr(), libc::RTLD_LAZY);
            if result.is_null() {
                let error = std::ffi::CStr::from_ptr(libc::dlerror());
                info!("Failed to load libc++_shared.so: {:?}", error);
            } else {
                info!("Successfully loaded libc++_shared.so");
            }
        }
    }

    #[cfg(desktop)]
    let mut builder = tauri::Builder::default()
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
            .plugin(tauri_plugin_store::Builder::new().build())
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

    // Plugins that should be available on all platforms
    let builder = builder
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init());


    // Core plugins that should run only on non-Android platforms
    //#[cfg(not(target_os = "android"))]
    //let builder = builder
    //    .plugin(tauri_plugin_opener::init())

    
    builder.setup(|app| {
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

            // Create main window explicitly with initialization script

            info!("Creating main window with initialization script");

            let main_window_builder =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::App("/".into()))
                    .initialization_script(&misc::get_error_handler_script())
                    .zoom_hotkeys_enabled(true);

            #[cfg(desktop)]
            let main_window_builder2 = main_window_builder
                .title("Yellow")
                .inner_size(1000.0, 800.0)
                .center();

            #[cfg(not(desktop))]
            let main_window_builder2 = main_window_builder;

            let main_window = main_window_builder2
                .build()
                .expect("Failed to create main window");

            #[cfg(debug_assertions)]
            {
                let do_open_devtools = std::env::var("TAURI_OPEN_DEVTOOLS")
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
                if do_open_devtools {
                    main_window.open_devtools();
                }
            }
            #[cfg(not(debug_assertions))]
            {
                let _ = &main_window;
            }

            //  todo pub fn background_throttling(mut self, policy: Option<BackgroundThrottlingPolicy>) -> Self {

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_window_size,
            commands::get_scale_factor,
            commands::log,
            commands::is_debug_mode,
            #[cfg(desktop)]
            commands::get_work_area,
            commands::get_build_commit_hash,
            commands::get_build_branch,
            commands::get_build_ts,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            notifications::close_notifications_window,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            notifications::create_notifications_window,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            notifications::show_notifications_window,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            notifications::hide_notifications_window,
            audio::play_audio,
            audio::stop_audio,
            audio::is_audio_playing
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
