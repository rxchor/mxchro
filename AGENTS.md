# AGENTS.md

## Project Overview

Windows keyboard macro automation app built with Tauri 2.0. Clean architecture separates backend logic (Rust) from UI (React/TypeScript).

## Architecture

Backend follows clean architecture with strict layer boundaries:

```
src-tauri/src/
├── core/              # Business logic, platform-agnostic
├── infrastructure/    # Windows-specific (hooks, keyboard simulation)
├── adapters/          # Tauri ↔ core bridge
└── lib.rs            # Entry point, wiring
```

**Critical rule**: Core layer must never depend on infrastructure or adapters. Infrastructure implements platform-specific logic. Adapters only connect framework to core.

## Development Commands

```bash
pnpm install               # Install dependencies
pnpm tauri dev             # Development (frontend + backend)
pnpm tauri build           # Build release
cd src-tauri && cargo test # Run Rust tests
cd src-tauri && cargo check
cd src-tauri && cargo clippy
cd src-tauri && cargo fmt
```

**Package manager**: pnpm (not npm). Never use npm.

## Debug Logging

Backend uses `env_logger`. Scripts in `package.json` set `RUST_LOG` automatically:

```bash
pnpm tauri:dev:info   # Important events (macro CRUD, thread lifecycle)
pnpm tauri:dev:debug  # Key events, macro triggers, self-triggering prevention
pnpm tauri:dev:trace  # Everything (every key event, lock acquisitions)
```

## Key Design Decisions

### Threading Model

All three macro modes (Once/Hold/Toggle) use condvar with a single thread per macro. **Never spawn threads per keypress.**

- `Mode::Once`: Uses `is_executing` flag as debounce against spam
- `Mode::Hold`: Thread loops while `active=true`, stops when `active=false`
- `Mode::Toggle`: Flips `active` state on each keypress

### Self-Triggering Prevention

Macros must not interfere with themselves via injected events:

- **Hold**: Ignore injected KeyUp (prevents macro's own KeyUp from deactivating Hold)
- **Toggle**: Ignore injected KeyDown (prevents macro's own KeyDown from toggling off)
- **Once**: Protected by `is_executing` flag

### Event Blocking

`block_original_event` only applies to **non-injected** events. If it blocked injected events too, the macro's own actions would never reach the application.

## Known Issues

### WebView2 Hook Limitation

Windows low-level keyboard hooks (`WH_KEYBOARD_LL`) don't capture events when the Tauri app window has focus. WebView2 runs in a subprocess with its own message loop, intercepting keyboard input before the hook can process it.

**Planned fix**: Frontend JS keyboard listener that forwards events to backend via Tauri command when app window has focus.

## Common Pitfalls

1. **Spawning threads per keypress** → Always use condvar pattern
2. **Blocking injected events** → `should_block` must check `!is_injected`
3. **Multiple reads in hook** → Process all event logic in single `macros.read()` acquisition (use `process_key_event`)
4. **Thread cleanup** → Always call `stop_repeat_thread()` before removing macro
5. **WebView2 focus** → Hook won't capture events when app window has focus (known limitation)

## File Modifications Guide

- **New macro features** → `core/macro_manager.rs` + `core/types.rs`
- **New key types** → `infrastructure/keyboard/mapper.rs`
- **Keyboard simulation changes** → `infrastructure/keyboard/simulator.rs`
- **New Tauri commands** → Add method to `MacroManager`, wrapper in `adapters/commands.rs`, register in `lib.rs`
- **Frontend** → React/TypeScript in `src/`, shadcn/ui components, Tauri commands via `@tauri-apps/api/core`
