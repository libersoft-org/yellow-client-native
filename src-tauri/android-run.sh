#!/bin/bash
# Script to build and run the Android app with proper environment variables

# Set environment variables
export ANDROID_HOME=~/Android/Sdk
export NDK_HOME=~/Android/Sdk/ndk/29.0.13113456
export ANDROID_NDK_HOME=~/Android/Sdk/ndk/29.0.13113456

# Copy C++ libraries
echo "Copying C++ shared libraries..."
./copy-cxx-lib.sh

# Build and run
echo "Building and running Android app..."
bun run tauri android dev