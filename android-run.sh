#!/bin/bash
# Script to build and run the Android app with proper environment variables

# Set environment variables
export ANDROID_HOME=~/Android/Sdk
export NDK_HOME=~/Android/Sdk/ndk/29.0.13113456
export ANDROID_NDK_HOME=~/Android/Sdk/ndk/29.0.13113456

# Copy C++ libraries (still needed for some core dependencies)
echo "Copying C++ shared libraries..."
./copy-cxx-lib.sh

# Clean previous builds to ensure dependencies are correctly processed
echo "Cleaning previous builds..."
rm -rf src-tauri/target/x86_64-linux-android/debug

# Build and run
echo "Building and running Android app..."
bun run tauri android dev