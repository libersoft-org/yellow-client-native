/// Returns a JavaScript script that installs global error handlers
pub fn get_error_handler_script() -> String {
    let debug_mode = cfg!(debug_assertions);
    let mut res = String::from(r#"
    console.log('[init] installing global error handlers');

    // catch sync errors
    window.addEventListener('error', event => {
      console.error('window.onerror:', event.error?.message, event.error?.stack);
    });

    // catch promise rejections
    window.addEventListener('unhandledrejection', event => {
      console.error('onunhandledrejection:', event.reason, event.reason?.stack);
    });

    "#);

    if debug_mode {
        res.push_str(r#"
        window.__TAURI_DEBUG_MODE__ = true;
        "#);
    }

    return res;
}
