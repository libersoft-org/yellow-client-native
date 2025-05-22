use chrono::prelude::*;
use std::env;

fn main() {
    // Let Tauri set up its stuff
    tauri_build::build();


    // Get current Git branch
    let git_branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get Git branch");
    let git_branch = String::from_utf8(git_branch.stdout).expect("Invalid UTF-8 in Git branch");

    // Get current build time in RFC3339 format
    let build_time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Emit values as compile-time env vars
}
