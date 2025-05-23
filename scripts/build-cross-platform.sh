#!/bin/bash
# Cross-platform build script for V8/rustyscript targets

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
TARGET=""
RELEASE=""
VERBOSE=""

# Show usage
show_usage() {
    echo -e "${BLUE}Usage: $0 [OPTIONS]${NC}"
    echo ""
    echo -e "${YELLOW}Options:${NC}"
    echo "  -t, --target TARGET    Target platform to build for"
    echo "  -r, --release          Build in release mode"
    echo "  -v, --verbose          Verbose output"
    echo "  -h, --help             Show this help message"
    echo ""
    echo -e "${YELLOW}Supported targets:${NC}"
    echo "  android-arm64          Android ARM64 (aarch64-linux-android)"
    echo "  android-arm            Android ARM (armv7-linux-androideabi)"
    echo "  android-x86_64         Android x86_64"
    echo "  android-x86            Android x86"
    echo "  linux-arm64            Linux ARM64 (aarch64-unknown-linux-gnu)"
    echo "  linux-arm              Linux ARM (armv7-unknown-linux-gnueabihf)"
    echo "  ios-arm64              iOS ARM64 (aarch64-apple-ios)"
    echo "  ios-simulator          iOS Simulator (x86_64-apple-ios)"
    echo "  all-android            Build for all Android targets"
    echo "  all-linux-arm          Build for all Linux ARM targets"
    echo "  all-ios                Build for all iOS targets (macOS only)"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo "  $0 -t android-arm64 -r"
    echo "  $0 -t all-android"
    echo "  $0 -t ios-arm64 -v"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--target)
            TARGET="$2"
            shift 2
            ;;
        -r|--release)
            RELEASE="--release"
            shift
            ;;
        -v|--verbose)
            VERBOSE="--verbose"
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_usage
            exit 1
            ;;
    esac
done

if [ -z "$TARGET" ]; then
    echo -e "${RED}Error: Target must be specified${NC}"
    show_usage
    exit 1
fi

# Map friendly target names to Rust targets
map_target() {
    case $1 in
        android-arm64)
            echo "aarch64-linux-android"
            ;;
        android-arm)
            echo "armv7-linux-androideabi"
            ;;
        android-x86_64)
            echo "x86_64-linux-android"
            ;;
        android-x86)
            echo "i686-linux-android"
            ;;
        linux-arm64)
            echo "aarch64-unknown-linux-gnu"
            ;;
        linux-arm)
            echo "armv7-unknown-linux-gnueabihf"
            ;;
        ios-arm64)
            echo "aarch64-apple-ios"
            ;;
        ios-simulator)
            echo "x86_64-apple-ios"
            ;;
        *)
            echo ""
            ;;
    esac
}

# Build for a specific target
build_target() {
    local target=$1
    local rust_target=$(map_target "$target")
    
    if [ -z "$rust_target" ]; then
        echo -e "${RED}Error: Unknown target: $target${NC}"
        return 1
    fi
    
    echo -e "${GREEN}Building for $target ($rust_target)...${NC}"
    
    # Source V8 environment if it exists
    if [ -f "v8-env.sh" ]; then
        source v8-env.sh
    fi
    
    # Perform platform-specific setup
    case $target in
        android-*)
            if [ -z "$ANDROID_NDK_HOME" ]; then
                echo -e "${RED}Error: ANDROID_NDK_HOME not set${NC}"
                echo -e "${YELLOW}Please run: export ANDROID_NDK_HOME=/path/to/android-ndk${NC}"
                return 1
            fi
            
            # Copy C++ libraries for Android
            if [ -f "copy-cxx-lib.sh" ]; then
                echo -e "${BLUE}Copying C++ libraries...${NC}"
                ./copy-cxx-lib.sh
            fi
            ;;
        ios-*)
            if [[ "$OSTYPE" != "darwin"* ]]; then
                echo -e "${RED}Error: iOS builds require macOS${NC}"
                return 1
            fi
            ;;
    esac
    
    # Build command
    local build_cmd="cargo build --target $rust_target $RELEASE $VERBOSE"
    echo -e "${BLUE}Running: $build_cmd${NC}"
    
    if eval "$build_cmd"; then
        echo -e "${GREEN}✓ Successfully built for $target${NC}"
        return 0
    else
        echo -e "${RED}✗ Build failed for $target${NC}"
        return 1
    fi
}

# Build for multiple targets
build_multiple() {
    local targets=("$@")
    local failed_targets=()
    local success_count=0
    
    for target in "${targets[@]}"; do
        if build_target "$target"; then
            ((success_count++))
        else
            failed_targets+=("$target")
        fi
        echo ""
    done
    
    echo -e "${BLUE}=== Build Summary ===${NC}"
    echo -e "${GREEN}Successful builds: $success_count/${#targets[@]}${NC}"
    
    if [ ${#failed_targets[@]} -gt 0 ]; then
        echo -e "${RED}Failed targets:${NC}"
        for target in "${failed_targets[@]}"; do
            echo -e "  ${RED}✗ $target${NC}"
        done
        return 1
    else
        echo -e "${GREEN}All builds completed successfully!${NC}"
        return 0
    fi
}

# Main build logic
case $TARGET in
    all-android)
        echo -e "${GREEN}Building for all Android targets...${NC}"
        build_multiple "android-arm64" "android-arm" "android-x86_64" "android-x86"
        ;;
    all-linux-arm)
        echo -e "${GREEN}Building for all Linux ARM targets...${NC}"
        build_multiple "linux-arm64" "linux-arm"
        ;;
    all-ios)
        echo -e "${GREEN}Building for all iOS targets...${NC}"
        build_multiple "ios-arm64" "ios-simulator"
        ;;
    *)
        build_target "$TARGET"
        ;;
esac