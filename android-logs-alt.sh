#!/bin/bash
# Alternative logging script that uses process ID filtering

# Define ADB if not set
ADB=${ADB:-adb}

# Clear the log first
$ADB logcat -c

# Package name
PACKAGE="org.libersoft.yellow"

# Get the PID of our app
PID=$($ADB shell ps -A | grep $PACKAGE | awk '{print $2}')

if [ -z "$PID" ]; then
  echo "Could not find process for $PACKAGE"
  echo "Showing all logs with yellow/tauri/rust filter instead..."
  $ADB logcat -v threadtime | grep -i --color=auto "yellow\|tauri\|rust\|Error\|Exception"
else
  echo "Found process ID: $PID for $PACKAGE"
  echo "Showing logs for process $PID..."
  $ADB logcat -v threadtime --pid=$PID
fi