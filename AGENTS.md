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

### Action Model (Press/Release Separation)

Actions use a discriminated union (`ActionConfig`) that separates key press and release as independent actions:

```rust
pub enum ActionConfig {
    Key { key: String, press: bool },  // press=true for keydown, press=false for keyup
    Wait { duration_ms: u64 },
}
```

This enables complex key combinations (e.g., Ctrl+C = Press Ctrl, Press C, Release C, Release Ctrl) and gives users full control over timing with explicit Wait actions between press/release events.

### Macro Editing

Macros can be edited via `update_macro` command. The system:
- Stores original `actions_config` in `MacroInstance` to allow editing
- Uses `get_macro` command to retrieve full macro data for the edit form
- Supports changing the trigger key (removes old, creates new)

### Data Persistence

Macros persist to an embedded PoloDB database (document-oriented, BSON format), configurable via `.env`:

```env
# Development
MACROS_FILE_PATH=./macros.polodb

# Production (default if .env not set)
%APPDATA%/mxchro/macros.polodb
```

Implementation:
- `core/persistence.rs`: Handles PoloDB operations (load_all/insert/delete/replace/set_enabled/wipe_data)
- `trigger_key` used as document `_id` (natural unique key)
- Auto-load on startup via `MacroManager::new()`
- Granular operations after CRUD (no full file rewrite)
- `set_enabled` uses partial update with `$set` operator
- `replace` uses delete + insert (trigger_key can change during edit)
- `wipe_data` command available for clearing data during development

### Naming Conventions

**Backend (Rust):**
- **Entity**: Domain models in `core/types.rs` (e.g., `Macro`, `Action`, `Mode`)
- **Document**: PoloDB wrapper in `persistence.rs` (e.g., `MacroDocument` with `_id` field)
- **View**: Projections for UI in `core/types.rs` (e.g., `MacroView`)

**Frontend (TypeScript):**
- **Model**: Domain types in `src/model/` (e.g., `Macro`, `Action`, `Mode`, `MacroView`)
- **Response**: API response types from backend in `src/services/` (e.g., `MacroResponse`, `MacroViewResponse`, `ActionResponse`)

**Conversion Pattern:**
- Backend uses `snake_case` for JSON serialization
- Frontend `Response` types match backend `snake_case`
- Frontend `Model` types use `camelCase`
- Conversion functions in `MacroService.ts`: `fromSnakeCase()` converts Response → Model

**Example Flow:**
```
Backend: MacroView (snake_case in JSON)
    ↓
Frontend: MacroViewResponse (snake_case interface)
    ↓ fromSnakeCase()
Frontend: MacroView (camelCase model for UI)
```

### UI Features

- **Icons in ActionsList**: Each action type has a distinct icon (ArrowDownToLine for Press, ArrowUpFromLine for Release, Timer for Wait)
- **Full-element drag & drop**: Entire action card is draggable, not just the grip icon
- **Batch action creation**: Adding a key creates both Press and Release actions together by default (configurable)

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

## Code Style Conventions

- **Extract repeated logic** into helper functions within components to avoid duplication
- **Prefer composition over duplication**: If you write similar code multiple times, extract it into a reusable function
- **Use descriptive function names** that explain intent (e.g., `actionIcon`, `actionLabel`, `actionText`)
- **Keep components focused**: Delegate rendering logic to helper functions, keep main component clean
- **Helper functions should accept parameters for flexibility**: For example, `className` parameters allow callers to customize styling
- **Use discriminated unions** (TypeScript) or enums (Rust) for type-safe state representation
- **Prefer explicit over implicit**: Use `flex-row` instead of relying on defaults, pass full `className` strings rather than combining partial ones
- **Consistent container sizing**: Empty states and content states must use the same height/width (e.g., both `h-[40vh]`) to avoid layout shifts
- **Simple CSS solutions first**: Before reaching for component libraries (e.g., ScrollArea), check if native CSS (`overflow-y-auto`) solves the problem

## File Modifications Guide

- **New macro features** → `core/macro_manager.rs` + `core/types.rs`
- **New key types** → `infrastructure/keyboard/mapper.rs`
- **Keyboard simulation changes** → `infrastructure/keyboard/simulator.rs`
- **New Tauri commands** → Add method to `MacroManager`, wrapper in `adapters/commands.rs`, register in `lib.rs`
- **Frontend** → React/TypeScript in `src/`, shadcn/ui components, Tauri commands via `@tauri-apps/api/core`
