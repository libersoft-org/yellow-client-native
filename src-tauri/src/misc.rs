
/// Inject our global JS error handlers on every navigation
fn install_error_handlers(window: &Window) {
    // This script runs before any of your app’s own JS,
    // so it’ll catch everything, including initial-load errors.
    let js = r#"
    console.log('[init] installing global error handlers');

    // catch sync errors
    window.addEventListener('error', event => {
      console.error('window.onerror:', event.error?.message, event.error?.stack);
    });

    // catch promise rejections
    window.addEventListener('unhandledrejection', event => {
      console.error('onunhandledrejection:', event.reason, event.reason?.stack);
    });
  "#;

    window
        .eval(js)
        .expect("failed to inject error handler script");
}
