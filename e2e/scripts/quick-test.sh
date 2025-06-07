#!/bin/bash

# Quick E2E Test Runner (assumes setup is already done)
# Use this for rapid development/testing cycles

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

APPIUM_PORT=4723
DEVICE_NAME="emulator-5554"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --device)
            DEVICE_NAME="$2"
            shift 2
            ;;
        --help)
            echo "Quick E2E Test Runner"
            echo "Usage: $0 [--device DEVICE_NAME]"
            echo ""
            echo "This script assumes:"
            echo "  - Appium is already installed"
            echo "  - Android device/emulator is connected"
            echo "  - Yellow APK is already built"
            echo ""
            echo "For full setup, use: ./scripts/setup-and-test.sh"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1. Use --help for usage."
            exit 1
            ;;
    esac
done

log_info "🚀 Quick E2E Test Run"

# Check if device is connected
if ! adb devices | grep -q "device$"; then
    log_error "No Android devices found. Please connect a device or start an emulator."
    exit 1
fi

# Update device in test config
if [ -f "setup/global.setup.ts" ]; then
    sed -i "s/'appium:deviceName': '[^']*'/'appium:deviceName': '$DEVICE_NAME'/g" setup/global.setup.ts
fi

# Build latest service
log_info "🔨 Building latest service..."
cd .. && bun run build:messages-service && cd e2e

# Start Appium if not running
if ! lsof -i :$APPIUM_PORT >/dev/null 2>&1; then
    log_info "🌐 Starting Appium server..."
    appium --port $APPIUM_PORT > appium.log 2>&1 &
    APPIUM_PID=$!
    
    # Wait for startup
    for i in {1..15}; do
        if lsof -i :$APPIUM_PORT >/dev/null 2>&1; then
            break
        fi
        sleep 1
    done
    
    # Cleanup on exit
    trap 'kill $APPIUM_PID 2>/dev/null || true' EXIT
else
    log_info "✅ Appium server already running"
fi

# Run specific test or all tests
if [ $# -eq 1 ] && [[ $1 == *.test.ts ]]; then
    log_info "🧪 Running specific test: $1"
    npx jest "$1" --config jest.config.android.js --verbose
else
    log_info "🧪 Running all E2E tests..."
    npm run test:android -- --verbose
fi

log_success "✅ Quick test run completed!"