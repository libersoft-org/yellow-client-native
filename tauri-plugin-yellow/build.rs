const COMMANDS: &[&str] = &[
    "ping",
    "check_file_permissions",
    "request_file_permissions",
    "save_to_downloads",
    "export_file_to_downloads",
    "create_file",
    "append_to_file",
    "rename_file",
    "delete_file",
    "file_exists",
    "get_file_size",
    "open_save_dialog",
    "save_file_to_uri",
    "save_accounts_config",
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
