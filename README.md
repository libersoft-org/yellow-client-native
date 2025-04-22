# Yellow Native Client
[![Tauri v2 Release Process](https://github.com/libersoft-org/yellow-client-native/actions/workflows/cloud-publish.yaml/badge.svg)](https://github.com/libersoft-org/yellow-client-native/actions/workflows/cloud-publish.yaml)

Using Tauri 2, Svelte 5, SvelteKit, TypeScript, Vite and Bun.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Development

### dependencies

#### Linux
```
sudo apt install libcrypto++-dev libssl-dev
```


### Development
start android dev:
* `ANDROID_HOME=/home/koom/Android/Sdk/ NDK_HOME=~/Android/Sdk/ndk/29.0.13113456/ bun run tauri android dev`
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
