use chrono::prelude::*;
use std::env;

fn main() {
    // Let Tauri set up its stuff
    tauri_build::build();

    // Get current Git commit hash
    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to get Git commit hash");
    let git_hash = String::from_utf8(git_hash.stdout).expect("Invalid UTF-8 in Git hash");

    // Get current Git branch
    let git_branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get Git branch");
    let git_branch = String::from_utf8(git_branch.stdout).expect("Invalid UTF-8 in Git branch");

    // Get current build time in RFC3339 format
    let build_time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Emit values as compile-time env vars
    println!("cargo:rustc-env=GIT_HASH=\"{}\"", git_hash.trim());
    println!("cargo:rustc-env=GIT_BRANCH=\"{}\"", git_branch.trim());
    println!("cargo:rustc-env=BUILD_TIME=\"{}\"", build_time);

    // Handle platform-specific V8 and C++ library configuration
    let target = env::var("TARGET").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // Android platform configuration
    if target.contains("android") {
        println!("cargo:rustc-link-lib=c++_shared");
        
        // V8 configuration for Android
        configure_v8_for_android(&target_arch);
    }
    
    // iOS platform configuration  
    if target_os == "ios" {
        configure_v8_for_ios(&target_arch);
    }
    
    // ARM cross-compilation configuration
    if target_arch == "aarch64" || target_arch == "arm" {
        configure_v8_for_arm(&target_os, &target_arch);
    }
}

fn configure_v8_for_android(target_arch: &str) {
    println!("cargo:rustc-cfg=v8_target_android");
    
    // Set V8 flags for Android
    println!("cargo:rustc-env=V8_TARGET_OS=android");
    println!("cargo:rustc-env=V8_COMPONENT_BUILD=false");
    
    match target_arch {
        "aarch64" => {
            println!("cargo:rustc-env=V8_TARGET_CPU=arm64");
            println!("cargo:rustc-env=V8_TARGET_ARCH=arm64-v8a");
        },
        "arm" => {
            println!("cargo:rustc-env=V8_TARGET_CPU=arm");
            println!("cargo:rustc-env=V8_TARGET_ARCH=armeabi-v7a");
        },
        "x86_64" => {
            println!("cargo:rustc-env=V8_TARGET_CPU=x64");
            println!("cargo:rustc-env=V8_TARGET_ARCH=x86_64");
        },
        "x86" => {
            println!("cargo:rustc-env=V8_TARGET_CPU=x86");
            println!("cargo:rustc-env=V8_TARGET_ARCH=x86");
        },
        _ => {}
    }
    
    // Enable Android-specific V8 logging
    println!("cargo:rustc-env=V8_ANDROID_LOG_STDOUT=true");
}

fn configure_v8_for_ios(target_arch: &str) {
    println!("cargo:rustc-cfg=v8_target_ios");
    
    // Set V8 flags for iOS
    println!("cargo:rustc-env=V8_TARGET_OS=ios");
    println!("cargo:rustc-env=V8_COMPONENT_BUILD=false");
    println!("cargo:rustc-env=V8_MONOLITHIC=true");
    println!("cargo:rustc-env=V8_JITLESS=true"); // Required for iOS
    println!("cargo:rustc-env=IOS_DEPLOYMENT_TARGET=12.0");
    
    match target_arch {
        "aarch64" => {
            println!("cargo:rustc-env=V8_TARGET_CPU=arm64");
        },
        "x86_64" => {
            // iOS Simulator
            println!("cargo:rustc-env=V8_TARGET_CPU=x64");
            println!("cargo:rustc-cfg=ios_simulator");
        },
        _ => {}
    }
    
    // Disable features not supported on iOS
    println!("cargo:rustc-env=V8_DISABLE_I18N=true");
    println!("cargo:rustc-env=V8_DISABLE_POINTER_COMPRESSION=true");
}

fn configure_v8_for_arm(target_os: &str, target_arch: &str) {
    if target_os != "android" && target_os != "ios" {
        println!("cargo:rustc-cfg=v8_target_arm");
        
        match target_arch {
            "aarch64" => {
                println!("cargo:rustc-env=V8_TARGET_CPU=arm64");
            },
            "arm" => {
                println!("cargo:rustc-env=V8_TARGET_CPU=arm");
                // Be cautious with hard float ABI
                if env::var("V8_ARM_FLOAT_ABI").unwrap_or_default() == "hard" {
                    println!("cargo:rustc-env=V8_ARM_FLOAT_ABI=hard");
                }
            },
            _ => {}
        }
    }
}
