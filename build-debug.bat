@echo on

git stash
git pull
git submodule update --init
bun i --frozen-lockfile
bun --bun run tauri build --debug
