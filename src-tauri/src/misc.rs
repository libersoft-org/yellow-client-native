/// Returns a JavaScript script that installs global error handlers
pub fn get_error_handler_script() -> &'static str {
    r#"
    console.log('[init] installing global error handlers');

    // catch sync errors
    window.addEventListener('error', event => {
      console.error('window.onerror:', event.error?.message, event.error?.stack);
    });

    // catch promise rejections
    window.addEventListener('unhandledrejection', event => {
      console.error('onunhandledrejection:', event.reason, event.reason?.stack);
    });
    "#
}
