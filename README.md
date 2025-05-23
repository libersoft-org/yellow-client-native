# Yellow Native Client
[![Tauri v2 Release Process](https://github.com/libersoft-org/yellow-client-native/actions/workflows/cloud-publish.yaml/badge.svg)](https://github.com/libersoft-org/yellow-client-native/actions/workflows/cloud-publish.yaml)

Using Tauri 2, Svelte 5, SvelteKit, TypeScript, Vite and Bun.

## Cross-Platform V8/Rustyscript Build Support

This project includes enhanced build support for V8 cross-compilation to enable rustyscript functionality across multiple platforms including Android, iOS, and ARM Linux targets.

### Quick Setup

1. **Initial setup for cross-compilation:**
   ```bash
   ./scripts/setup-v8-cross-compile.sh
   ```

2. **Configure environment:**
   ```bash
   source v8-env.sh
   ```

3. **Build for specific platform:**
   ```bash
   ./scripts/build-cross-platform.sh -t android-arm64 -r
   ```

### Supported Platforms

- **Android**: ARM64, ARM, x86_64, x86
- **iOS**: ARM64, Simulator (x86_64)  
- **Linux ARM**: ARM64, ARM

### Cross-Compilation Requirements

#### Android
- Android NDK (set `ANDROID_NDK_HOME`)
- Rust Android targets installed

#### iOS (macOS only)
- Xcode with iOS SDK
- Rust iOS targets installed

#### ARM Linux
- GCC cross-compilers (`gcc-aarch64-linux-gnu`, `gcc-arm-linux-gnueabihf`)

### Build Configuration

The build system automatically detects target platforms and configures V8 compilation flags:

- **Android**: Enables component build, logging, architecture-specific settings
- **iOS**: Enables jitless mode (required), monolithic build, disables unsupported features
- **ARM**: Configures float ABI and architecture-specific optimizations

### Environment Variables

Key environment variables for V8 cross-compilation:
- `V8_FROM_SOURCE=1`: Build V8 from source
- `V8_ENABLE_POINTER_COMPRESSION=false`: Disable pointer compression
- `ANDROID_NDK_HOME`: Path to Android NDK
- `V8_ARM_FLOAT_ABI`: ARM float ABI setting

### Build Scripts

- `scripts/setup-v8-cross-compile.sh`: Initial environment setup
- `scripts/build-cross-platform.sh`: Cross-platform build tool
- `copy-cxx-lib.sh`: Android C++ library setup
- `v8-env.sh`: Environment configuration (generated)

See [V8 Cross-Compilation Docs](https://v8.dev/docs/cross-compile-arm) and [V8 iOS Docs](https://v8.dev/docs/cross-compile-ios) for detailed information.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Development

### dependencies

https://tauri.app/start/prerequisites/

#### Linux
```
(outdated, ignore)
sudo apt install libcrypto++-dev libssl-dev libasound2-dev
```


### Development
#### Android
```
 sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

 curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

 rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

 JAVA_HOME=/snap/android-studio/current/jbr/ CMAKE_MAKE_PROGRAM=/bin/make ANDROID_NDK_HOME=~/Android/Sdk/ndk/* NDK_HOME=~/Android/Sdk/ndk/* ANDROID_HOME=~/Android/Sdk/ bun run tauri android init
 JAVA_HOME=/snap/android-studio/current/jbr/ CMAKE_MAKE_PROGRAM=/bin/make ANDROID_NDK_HOME=~/Android/Sdk/ndk/* NDK_HOME=~/Android/Sdk/ndk/* ANDROID_HOME=~/Android/Sdk/ bun run tauri android build --debug

```
start android dev:
...` tauri android dev`

start desktop dev:
* `bun tauri dev`

### Build

**Debug:**

```sh
bun tauri build --debug
```

**Release:**

```sh
bun tauri build
```


