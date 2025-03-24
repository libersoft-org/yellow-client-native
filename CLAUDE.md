# Development Guidelines for Yellow Project

## Build, Lint & Test Commands
- Build: `bun run build` or `vite build`
- Run dev: `bun run tauri dev`
- TypeCheck: `bun run check` or `svelte-check --tsconfig ./tsconfig.json`
- Preview: `bun run preview`
- Watch mode: `bun run check:watch`

## Code Style
- Framework: Tauri 2.0, Svelte 5, SvelteKit, TypeScript
- File Naming: kebab-case for components, camelCase for utilities
- TypeScript: Strict mode for type checking
- Components: Svelte components with TypeScript
- Imports: Group imports by source/type (framework, internal, types)
- Rust: Follow Rust style conventions for Tauri backend code
- Error Handling: Use Result types in Rust code, try/catch patterns in TypeScript
- Logging: Use structured logging with the info/debug macros

## Architecture
- Frontend: Svelte 5 with SvelteKit
- Backend: Tauri 2 with Rust for native capabilities
- Windows: Use the Manager trait for window operations
- Tauri Commands: Use #[tauri::command] for exposing Rust functions to frontend

This file is meant for Claude and other AI assistants to understand project conventions.