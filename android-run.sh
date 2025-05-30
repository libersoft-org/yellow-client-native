#!/bin/bash
# Script to build and run the Android app with proper environment variables

# Set environment variables
export ANDROID_HOME=~/Android/Sdk
export NDK_HOME=~/Android/Sdk/ndk/29.0.13113456
export ANDROID_NDK_HOME=~/Android/Sdk/ndk/29.0.13113456

# Copy C++ libraries only if MANUAL_CXX_LIB feature is enabled
if [ "$MANUAL_CXX_LIB" = "1" ]; then
    echo "Copying C++ shared libraries (MANUAL_CXX_LIB enabled)..."
    ./copy-cxx-lib.sh
else
    echo "Skipping C++ library copy (MANUAL_CXX_LIB not enabled)"
fi

# Clean previous builds to ensure dependencies are correctly processed
echo "Cleaning previous builds..."
rm -rf src-tauri/target/x86_64-linux-android/debug

# Build and run
echo "Building and running Android app..."
bun run tauri android dev

# This process will keep running, so we need to show logs in a new terminal
# You can manually run this command in a separate terminal:
echo "To view logs, run the following in a separate terminal:"
echo "adb logcat -v time '*:S' yellow:V YellowApp:V"