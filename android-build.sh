#!/bin/bash
# Script to build and run the Android app with proper environment variables

# Set environment variables
export ANDROID_HOME=~/Android/Sdk
export NDK_HOME=~/Android/Sdk/ndk/29.0.13113456
export ANDROID_NDK_HOME=~/Android/Sdk/ndk/29.0.13113456

# Build and run
echo "Building and running Android app..."
bun run tauri android build

# This process will keep running, so we need to show logs in a new terminal
# You can manually run this command in a separate terminal:
echo "To view logs, run the following in a separate terminal:"
echo "adb logcat -v time '*:S' yellow:V YellowApp:V"