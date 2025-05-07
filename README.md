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
 JAVA_HOME=/snap/android-studio/189/jbr  CMAKE_MAKE_PROGRAM=/bin/make ANDROID_NDK_HOME=~/Android/Sdk/ndk/28.0.13004108/   NDK_HOME=~/Android/Sdk/ndk/28.0.13004108/ ANDROID_HOME=~/Android/Sdk/ bun run tauri android init
 JAVA_HOME=/snap/android-studio/189/jbr  CMAKE_MAKE_PROGRAM=/bin/make ANDROID_NDK_HOME=~/Android/Sdk/ndk/28.0.13004108/   NDK_HOME=~/Android/Sdk/ndk/28.0.13004108/ ANDROID_HOME=~/Android/Sdk/ bun run tauri android build --target armv7 --debug
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


