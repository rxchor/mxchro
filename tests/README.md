# Tests

All test code and fixtures are centralized here:

```text
tests/
├── application.rs          Integration: use cases, validation, queries and persistence
├── architecture.rs         AST import/path checks and DTO ownership/usage
├── support/mod.rs          Fake input and exclusive temporary database
├── support/memory.rs       In-memory repository ports for engine tests
├── perf/                   Opt-in native scroll profiling and checks
├── native/                 Opt-in native selector, navigation and feedback interaction
└── unit/
    ├── application/        Engine and indexes (without PoloDB)
    ├── infrastructure/     Mapper/input (without hooks or real input injection)
    └── view/               Backend, UI, assets, i18n and theme
```

- `application.rs` is a Cargo integration target consuming the public `mxchro`
  API like any other library client.
- Unit tests are included from their modules with `#[cfg(test)]` and
  `#[path = "…"] mod tests;`, preserving private access and area-based organization.
  Test output retains the logical module path, although files are stored here.
- `src/` contains only test module declarations, not test bodies or fixtures.
- `support/` is shared by integration/worker tests; each suite's `app` alias
  refers to the external library or the crate under test.
- `unit/infrastructure/persistence_path.rs` checks settings/database shared
  defaults, APPDATA fallback, inherited overrides and the optional NSIS removal
  directory, without opening databases or changing environment variables.

From the repository root:

```powershell
cargo test                       # All tests
cargo test --test application    # Integration tests only
cargo test --test architecture   # Dependency and DTO rules
cargo test --lib                 # Unit tests included from tests/unit/
cargo test hook_does_not_wait    # Filter by test name
cargo run --example scroll_profile # Opt-in native profiling; no backend/hooks/DB
cargo run --example collection_hotkey_ui # Collection selector; no backend/hooks/DB
cargo run --example collection_name_ui # Create/rename popovers, focus, validation, pending
cargo run --example sidebar_navigation_ui # Leave editor through filters, selection, locks
cargo run --example hotkey_rules_ui # Grouped hotkeys, explicit clearing, legacy warnings
cargo run --example input_capture_ui # Real capture controls with fake port, no hooks/DB
cargo run --example confirm_dialog_ui # Real modal, cancellation, focus, stale targets
cargo run --example toggle_feedback_ui # Pixels and locking with MxchroApp/fake channels
cargo run --example checkbox_ui # Entire card, one callback, keyboard, disabled
cargo run --example notifications_ui # Dismissal/autohide, validation, acknowledgements
cargo run --example settings_ui # Themes/logos, temporary TOML, retained editor
cargo run --example grid_layout_ui -- --check # Isolated grid experiment, no production change
```

Engine tests use a fake `InputBackend` and `support/memory.rs` repositories,
without a database. Integration/worker fixtures use an exclusive UUID temporary
path under `opencode`, removed afterward. They do not change environment variables,
open user databases, call `bootstrap::run` or install hooks.

Architecture tests analyze Rust sources with `syn`: imports and paths, not
compiler-enforced crate isolation or arbitrary macro expansion. Native checks
cover edit-detail queries and returning to summary DTOs after accepted saves.
UI helper tests are not equivalent to complete graphical testing.

`unit/infrastructure/capture.rs` covers atomic capture state with fake events;
`hook.rs` covers all five mouse-button messages, and `simulator.rs` checks modifier
flags without SendInput. No global hooks are installed.

Profiling opens its own window and dispatches synthetic GPUI events only.
Do not run it automatically as part of unit tests. Methods and measurement
limitations: [perf/README.md](perf/README.md).
