# Yellow Native Client
[![Tauri v2 Release Process](https://github.com/libersoft-org/yellow-client-native/actions/workflows/cloud-publish.yaml/badge.svg)](https://github.com/libersoft-org/yellow-client-native/actions/workflows/cloud-publish.yaml)

Using Tauri 2, Svelte 5, SvelteKit, TypeScript, Vite and Bun.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Development

### dependencies

https://tauri.app/start/prerequisites/

#### Linux
```
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


