#!/bin/bash
# This script copies the C++ shared library to the appropriate location for Android builds

# Ensure the NDK path is provided
if [ -z "$ANDROID_NDK_HOME" ]; then
  echo "ANDROID_NDK_HOME environment variable not set"
  # Try to find it from the SDK location
  if [ -n "$ANDROID_SDK_ROOT" ]; then
    POSSIBLE_NDK="$ANDROID_SDK_ROOT/ndk-bundle"
    if [ -d "$POSSIBLE_NDK" ]; then
      ANDROID_NDK_HOME="$POSSIBLE_NDK"
    else
      # Try to find the newest version in the NDK directory
      if [ -d "$ANDROID_SDK_ROOT/ndk" ]; then
        NEWEST_NDK=$(find "$ANDROID_SDK_ROOT/ndk" -maxdepth 1 -type d | sort -r | head -1)
        if [ -n "$NEWEST_NDK" ]; then
          ANDROID_NDK_HOME="$NEWEST_NDK"
        fi
      fi
    fi
  fi
  
  if [ -z "$ANDROID_NDK_HOME" ]; then
    echo "Could not find Android NDK. Please set ANDROID_NDK_HOME."
    exit 1
  fi
fi

echo "Using NDK at: $ANDROID_NDK_HOME"

# Directory where we'll copy the library
TARGET_DIR="src-tauri/gen/android/app/src/main/jniLibs"
mkdir -p "$TARGET_DIR/x86_64"
mkdir -p "$TARGET_DIR/arm64-v8a"
mkdir -p "$TARGET_DIR/armeabi-v7a"
mkdir -p "$TARGET_DIR/x86"

# Find C++ shared library in NDK
find "$ANDROID_NDK_HOME" -name "libc++_shared.so" | while read -r lib; do
  # Determine architecture from path
  if [[ "$lib" == *"/x86_64/"* ]]; then
    cp "$lib" "$TARGET_DIR/x86_64/"
    echo "Copied x86_64 library to $TARGET_DIR/x86_64/"
  elif [[ "$lib" == *"/arm64-v8a/"* || "$lib" == *"/aarch64-"* ]]; then
    cp "$lib" "$TARGET_DIR/arm64-v8a/"
    echo "Copied arm64-v8a library to $TARGET_DIR/arm64-v8a/"
  elif [[ "$lib" == *"/armeabi-v7a/"* || "$lib" == *"/armv7a-"* ]]; then
    cp "$lib" "$TARGET_DIR/armeabi-v7a/"
    echo "Copied armeabi-v7a library to $TARGET_DIR/armeabi-v7a/"
  elif [[ "$lib" == *"/x86/"* || "$lib" == *"/i686-"* ]]; then
    cp "$lib" "$TARGET_DIR/x86/"
    echo "Copied x86 library to $TARGET_DIR/x86/"
  fi
done

echo "C++ shared libraries have been copied to the Android project"