#!/bin/bash

cd ../yellow-client; TAURI=true bun run build; cp -r build ../yellow-client-native
