#!/bin/bash
# Script to view Android logs for the Yellow app

# Define ADB if not set
ADB=${ADB:-adb}

# Clear the log first (optional)
$ADB logcat -c

# Get the package name for filtering
PACKAGE="org.libersoft.yellow"
echo "Showing logs for $PACKAGE"

# Run a more comprehensive logcat command
echo "Running logcat with broad filters to catch all app logs..."

# Option 1: By package name (this will show logs from your app's process)
$ADB logcat -v threadtime | grep -i --color=auto "$PACKAGE\|yellow\|rust\|tauri\|Error\|Exception"

# If the above command doesn't show logs, manually uncomment and try one of these:
# Option 2: Show all Rust/Tauri related logs
# $ADB logcat -v threadtime "*:D" | grep -i --color=auto "yellow\|rust\|tauri"

# Option 3: Show all logs without filtering
# $ADB logcat -v threadtime