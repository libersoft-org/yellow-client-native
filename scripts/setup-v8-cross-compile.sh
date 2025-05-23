#!/bin/bash
# Setup script for V8 cross-compilation environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Setting up V8 cross-compilation environment...${NC}"

# Check if we're on macOS (required for iOS builds)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${GREEN}✓ macOS detected - iOS builds supported${NC}"
    IOS_SUPPORTED=true
else
    echo -e "${YELLOW}⚠ iOS builds require macOS${NC}"
    IOS_SUPPORTED=false
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for required tools
echo -e "${GREEN}Checking required tools...${NC}"

if command_exists rustup; then
    echo -e "${GREEN}✓ Rust toolchain found${NC}"
else
    echo -e "${RED}✗ Rust toolchain not found. Please install Rust first.${NC}"
    exit 1
fi

if command_exists git; then
    echo -e "${GREEN}✓ Git found${NC}"
else
    echo -e "${RED}✗ Git not found. Please install Git first.${NC}"
    exit 1
fi

# Install required Rust targets
echo -e "${GREEN}Installing Rust targets for cross-compilation...${NC}"

# Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android

# ARM Linux targets
rustup target add aarch64-unknown-linux-gnu
rustup target add armv7-unknown-linux-gnueabihf

# iOS targets (macOS only)
if [ "$IOS_SUPPORTED" = true ]; then
    rustup target add aarch64-apple-ios
    rustup target add x86_64-apple-ios
    echo -e "${GREEN}✓ iOS targets installed${NC}"
fi

echo -e "${GREEN}✓ Rust targets installed${NC}"

# Check for Android NDK
echo -e "${GREEN}Checking Android NDK...${NC}"
if [ -n "$ANDROID_NDK_HOME" ] && [ -d "$ANDROID_NDK_HOME" ]; then
    echo -e "${GREEN}✓ Android NDK found at: $ANDROID_NDK_HOME${NC}"
    
    # Add NDK tools to PATH if not already there
    NDK_TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin"
    if [ -d "$NDK_TOOLCHAIN" ]; then
        export PATH="$NDK_TOOLCHAIN:$PATH"
        echo -e "${GREEN}✓ NDK toolchain added to PATH${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Android NDK not found. Set ANDROID_NDK_HOME for Android builds${NC}"
    echo -e "${YELLOW}  Download from: https://developer.android.com/ndk/downloads${NC}"
fi

# Check for cross-compilation tools
echo -e "${GREEN}Checking cross-compilation tools...${NC}"

# Check for ARM GCC cross-compiler
if command_exists aarch64-linux-gnu-gcc; then
    echo -e "${GREEN}✓ ARM64 GCC cross-compiler found${NC}"
else
    echo -e "${YELLOW}⚠ ARM64 GCC cross-compiler not found${NC}"
    echo -e "${YELLOW}  Install with: sudo apt-get install gcc-aarch64-linux-gnu (Ubuntu/Debian)${NC}"
fi

if command_exists arm-linux-gnueabihf-gcc; then
    echo -e "${GREEN}✓ ARM GCC cross-compiler found${NC}"
else
    echo -e "${YELLOW}⚠ ARM GCC cross-compiler not found${NC}"
    echo -e "${YELLOW}  Install with: sudo apt-get install gcc-arm-linux-gnueabihf (Ubuntu/Debian)${NC}"
fi

# Create V8 environment setup script
cat > v8-env.sh << 'EOF'
#!/bin/bash
# V8 environment variables for cross-compilation

# Android configuration
if [ -n "$ANDROID_NDK_HOME" ]; then
    export CC_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/aarch64-linux-android21-clang"
    export CXX_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/aarch64-linux-android21-clang++"
    export AR_aarch64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/llvm-ar"
    
    export CC_armv7_linux_androideabi="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/armv7a-linux-androideabi21-clang"
    export CXX_armv7_linux_androideabi="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/armv7a-linux-androideabi21-clang++"
    export AR_armv7_linux_androideabi="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/llvm-ar"
    
    export CC_x86_64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/x86_64-linux-android21-clang"
    export CXX_x86_64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/x86_64-linux-android21-clang++"
    export AR_x86_64_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/llvm-ar"
    
    export CC_i686_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/i686-linux-android21-clang"
    export CXX_i686_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/i686-linux-android21-clang++"
    export AR_i686_linux_android="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -s | tr '[:upper:]' '[:lower:]')-x86_64/bin/llvm-ar"
fi

# ARM Linux configuration
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar

export CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc
export CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
export AR_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-ar

# V8 configuration
export V8_FROM_SOURCE=1
export V8_ENABLE_POINTER_COMPRESSION=false

echo "V8 cross-compilation environment configured"
EOF

chmod +x v8-env.sh

echo -e "${GREEN}✓ Created v8-env.sh environment setup script${NC}"
echo -e "${GREEN}✓ V8 cross-compilation environment setup complete!${NC}"
echo ""
echo -e "${YELLOW}Usage:${NC}"
echo -e "  1. Source the environment: ${GREEN}source v8-env.sh${NC}"
echo -e "  2. Build for target: ${GREEN}cargo build --target aarch64-linux-android${NC}"
echo ""
echo -e "${YELLOW}Supported targets:${NC}"
echo -e "  • aarch64-linux-android (Android ARM64)"
echo -e "  • armv7-linux-androideabi (Android ARM)"
echo -e "  • x86_64-linux-android (Android x86_64)"
echo -e "  • i686-linux-android (Android x86)"
echo -e "  • aarch64-unknown-linux-gnu (Linux ARM64)"
echo -e "  • armv7-unknown-linux-gnueabihf (Linux ARM)"
if [ "$IOS_SUPPORTED" = true ]; then
echo -e "  • aarch64-apple-ios (iOS ARM64)"
echo -e "  • x86_64-apple-ios (iOS Simulator)"
fi