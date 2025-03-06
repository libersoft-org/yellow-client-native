#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(str_as_str)]
fn main() {
    tauri_app_lib::run()
}
