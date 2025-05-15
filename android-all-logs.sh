#!/bin/bash
# Script to show ALL Android logs without filtering
# Use this when you can't find your app's logs with other filters

# Define ADB if not set
ADB=${ADB:-adb}

# Clear the log first
$ADB logcat -c

# Show ALL logs with timestamps
echo "Showing ALL Android logs. Press Ctrl+C to exit."
echo "Look for org.libersoft.yellow, YellowApp, yellow, Tauri, or Rust messages"
$ADB logcat -v threadtime