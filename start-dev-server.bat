@echo on

git pull
bun i --frozen-lockfile
bun --bun run tauri dev --config server-config.json
