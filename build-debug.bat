@echo on

git stash
git pull
git submodule update --init
cd yellow-client
git checkout main
git pull
cd ..
bun i --frozen-lockfile
bun --bun run tauri build --debug
