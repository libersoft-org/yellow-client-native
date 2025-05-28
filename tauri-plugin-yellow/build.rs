const COMMANDS: &[&str] = &["ping", "check_file_permissions", "request_file_permissions", "save_to_downloads"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
