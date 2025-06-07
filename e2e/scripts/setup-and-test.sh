#!/bin/bash

# Yellow Native Android E2E Test Setup and Runner
# This script sets up the complete testing environment and runs the tests

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if port is in use
port_in_use() {
    lsof -i :$1 >/dev/null 2>&1
}

# Default values
SKIP_BUILD=false
SKIP_APPIUM_INSTALL=false
SKIP_DEVICE_CHECK=false
DEVICE_NAME="emulator-5554"
APP_PACKAGE="org.libersoft.yellow"
APPIUM_PORT=4723
ADB_PATH=""

# Auto-detect ADB if not provided
find_adb() {
    # Check if ADB env var is set
    if [ ! -z "$ADB" ]; then
        echo "$ADB"
        return
    fi
    
    # Check if adb is in PATH
    if command_exists adb; then
        echo "adb"
        return
    fi
    
    # Check common Android SDK locations
    local common_paths=(
        "$HOME/Android/Sdk/platform-tools/adb"
        "$HOME/android-sdk/platform-tools/adb"
        "$ANDROID_HOME/platform-tools/adb"
        "$ANDROID_SDK_ROOT/platform-tools/adb"
        "/opt/android-sdk/platform-tools/adb"
        "/usr/local/android-sdk/platform-tools/adb"
    )
    
    for path in "${common_paths[@]}"; do
        if [ -f "$path" ]; then
            echo "$path"
            return
        fi
    done
    
    # Not found
    echo ""
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-appium-install)
            SKIP_APPIUM_INSTALL=true
            shift
            ;;
        --skip-device-check)
            SKIP_DEVICE_CHECK=true
            shift
            ;;
        --device)
            DEVICE_NAME="$2"
            shift 2
            ;;
        --package)
            APP_PACKAGE="$2"
            shift 2
            ;;
        --port)
            APPIUM_PORT="$2"
            shift 2
            ;;
        --adb-path)
            ADB_PATH="$2"
            shift 2
            ;;
        --help)
            echo "Yellow E2E Test Setup and Runner"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --skip-build           Skip building the Yellow Android APK"
            echo "  --skip-appium-install  Skip Appium installation"
            echo "  --skip-device-check    Skip Android device connectivity check"
            echo "  --device NAME          Android device/emulator name (default: $DEVICE_NAME)"
            echo "  --package NAME         App package name (default: $APP_PACKAGE)"
            echo "  --port PORT            Appium server port (default: $APPIUM_PORT)"
            echo "  --adb-path PATH        Path to adb binary (auto-detected if not provided)"
            echo "  --help                 Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                                    # Full setup and test run"
            echo "  $0 --skip-build                      # Skip APK build"
            echo "  $0 --device pixel_7_api_33           # Use specific emulator"
            echo "  $0 --skip-appium-install --skip-build # Quick test run"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Find ADB path
if [ ! -z "$ADB_PATH" ]; then
    ADB_COMMAND="$ADB_PATH"
else
    ADB_COMMAND=$(find_adb)
fi

if [ -z "$ADB_COMMAND" ]; then
    log_error "Android Debug Bridge (adb) not found."
    log_error "Please install Android SDK or set ADB environment variable:"
    log_error "  export ADB=/path/to/android/sdk/platform-tools/adb"
    log_error "  Or use: $0 --adb-path /path/to/adb"
    exit 1
fi

log_info "🚀 Starting Yellow Native Android E2E Test Setup"
log_info "Device: $DEVICE_NAME | Package: $APP_PACKAGE | Port: $APPIUM_PORT"
log_info "ADB: $ADB_COMMAND"

# Step 1: Check prerequisites
log_info "📋 Checking prerequisites..."

if ! command_exists node; then
    log_error "Node.js is not installed. Please install Node.js 18+ and try again."
    exit 1
fi

if ! command_exists npm; then
    log_error "npm is not installed. Please install npm and try again."
    exit 1
fi

# ADB check now happens earlier, so we can skip this

if ! command_exists java; then
    log_warning "Java not found in PATH. Appium may require Java."
fi

log_success "Prerequisites check passed"

# Step 2: Install E2E test dependencies
log_info "📦 Installing E2E test dependencies..."
npm install

# Step 3: Install Appium (if not skipped)
if [ "$SKIP_APPIUM_INSTALL" = false ]; then
    log_info "🔧 Installing Appium and drivers..."
    
    if ! command_exists appium; then
        npm install -g appium
    else
        log_success "Appium already installed"
    fi
    
    # Install UiAutomator2 driver
    appium driver install uiautomator2 || log_warning "UiAutomator2 driver installation failed or already installed"
    
    log_success "Appium installation completed"
else
    log_info "⏭️  Skipping Appium installation"
fi

# Step 4: Check Android device connectivity (if not skipped)
if [ "$SKIP_DEVICE_CHECK" = false ]; then
    log_info "📱 Checking Android device connectivity..."
    
    if ! $ADB_COMMAND devices | grep -q "device$"; then
        log_error "No Android devices found or device not authorized."
        log_error "Please:"
        log_error "1. Connect your Android device via USB and enable USB debugging"
        log_error "2. OR start an Android emulator"
        log_error "3. Run '$ADB_COMMAND devices' to verify connection"
        log_error ""
        log_info "To start an emulator, you can run:"
        log_info "  emulator -list-avds                    # List available emulators"
        log_info "  emulator -avd <avd_name> &             # Start specific emulator"
        exit 1
    fi
    
    # Check if specific device is available
    if ! $ADB_COMMAND devices | grep -q "$DEVICE_NAME"; then
        log_warning "Specified device '$DEVICE_NAME' not found."
        log_info "Available devices:"
        $ADB_COMMAND devices
        
        # Use first available device
        DEVICE_NAME=$($ADB_COMMAND devices | grep "device$" | head -1 | cut -f1)
        log_info "Using device: $DEVICE_NAME"
    fi
    
    log_success "Android device connectivity verified"
else
    log_info "⏭️  Skipping device connectivity check"
fi

# Step 5: Build Yellow service file
log_info "🔨 Building Yellow messages service..."
cd ../yellow-client  # Go to yellow-client submodule
if [ ! -f "package.json" ]; then
    log_error "yellow-client submodule not found. Please run: git submodule update --init"
    exit 1
fi
bun run build:messages-service
log_success "Messages service built"

# Step 6: Build Yellow Android APK (if not skipped)
if [ "$SKIP_BUILD" = false ]; then
    log_info "🏗️  Building Yellow Android APK..."
    
    # Go back to client-native directory for APK build
    cd ..
    
    # Check if we're in the right directory
    if [ ! -f "src-tauri/Cargo.toml" ]; then
        log_error "Not in Yellow client-native directory. Please run this script from the e2e directory."
        exit 1
    fi
    
    # Build the APK
    if command_exists bun; then
        bun run tauri android build
    else
        npm run tauri android build
    fi
    
    # Check if APK was created
    APK_PATH="src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk"
    if [ ! -f "$APK_PATH" ]; then
        log_error "APK build failed. Expected file not found: $APK_PATH"
        exit 1
    fi
    
    log_success "Yellow Android APK built successfully"
else
    log_info "⏭️  Skipping APK build"
    
    # Go back to client-native directory to check APK
    cd ..
    
    # Still check if APK exists
    APK_PATH="src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk"
    if [ ! -f "$APK_PATH" ]; then
        log_warning "APK not found at $APK_PATH"
        log_warning "Tests may fail. Consider running without --skip-build"
    fi
fi

# Go back to e2e directory  
cd e2e

# Step 7: Start Appium server
log_info "🌐 Starting Appium server..."

# Kill any existing Appium process on the port
if port_in_use $APPIUM_PORT; then
    log_info "Port $APPIUM_PORT is in use. Attempting to kill existing process..."
    lsof -ti :$APPIUM_PORT | xargs kill -9 2>/dev/null || true
    sleep 2
fi

# Start Appium in background
appium --port $APPIUM_PORT > appium.log 2>&1 &
APPIUM_PID=$!

# Wait for Appium to start
log_info "Waiting for Appium server to start..."
for i in {1..30}; do
    if port_in_use $APPIUM_PORT; then
        log_success "Appium server started on port $APPIUM_PORT"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        log_error "Appium server failed to start within 30 seconds"
        log_error "Check appium.log for details:"
        cat appium.log
        exit 1
    fi
done

# Function to cleanup on exit
cleanup() {
    log_info "🧹 Cleaning up..."
    if [ ! -z "$APPIUM_PID" ]; then
        kill $APPIUM_PID 2>/dev/null || true
        log_info "Appium server stopped"
    fi
}

# Set trap to cleanup on script exit
trap cleanup EXIT

# Step 8: Update test configuration with actual device
log_info "⚙️  Updating test configuration..."

# Update the global setup with actual device name
sed -i "s/'appium:deviceName': 'emulator-5554'/'appium:deviceName': '$DEVICE_NAME'/g" setup/global.setup.ts
sed -i "s/'appium:appPackage': 'org.libersoft.yellow'/'appium:appPackage': '$APP_PACKAGE'/g" setup/global.setup.ts

# Step 9: Run the tests
log_info "🧪 Running E2E tests..."

# Create test results directory
mkdir -p test-results

# Run tests with detailed output
if npm run test:android -- --verbose --detectOpenHandles; then
    log_success "🎉 All E2E tests passed!"
    
    # Show test summary
    log_info "📊 Test Summary:"
    log_info "  Device: $DEVICE_NAME"
    log_info "  Package: $APP_PACKAGE"
    log_info "  APK: Built and tested"
    log_info "  Service: Built and tested"
    
else
    log_error "❌ Some E2E tests failed"
    log_info "📋 Debugging information:"
    log_info "  Appium logs: $(pwd)/appium.log"
    log_info "  Device logs: $ADB_COMMAND -s $DEVICE_NAME logcat"
    log_info "  Test configuration: $(pwd)/setup/global.setup.ts"
    
    # Show recent Appium logs
    log_info "Recent Appium logs:"
    tail -20 appium.log || true
    
    exit 1
fi

log_success "✅ E2E test run completed successfully!"