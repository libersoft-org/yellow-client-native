# Yellow native client

Using Tauri 2, Svelte 5, SvelteKit, TypeScript, Vite and Bun.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## development

### start frontend dev server
* `cd ../yellow-client/ && bun run dev`
start android dev:
* `ANDROID_HOME=/home/koom/Android/Sdk/ NDK_HOME=~/Android/Sdk/ndk/29.0.13113456/ bun run tauri android dev`
start desktop dev:
* `bun run tauri dev`
