# Development

[Index](README.md) · [Architecture](architecture.md) · [Tests](../tests/README.md)

## Environment and scope

Mxchro is a native Windows application using Rust (edition 2024) and GPUI-KIT.
Package, crate and executable: `mxchro`; root entity: `MxchroApp`.
The original Tauri/React project (`../mxchro_tauri`) is a reference only unless
explicitly requested. Do not reintroduce Tauri, React, JavaScript, WebView2 or
serialization to connect the UI and application. Node, npm and pnpm are not required.

Requires Rust, Windows/MSVC build tools and the C++/LLVM environment needed by
the native RocksDB/PoloDB bindings. Work in this repository; do not rename
directories or move data by inference. Preserve explicit paths even if their
names refer to an older project layout.

Keep documentation, READMEs, explanatory source comments and examples in English.
Do not translate identifiers, mapper names, stored values or user content.
`AGENTS.md` is a local, Git-ignored instruction file; do not force-add it or make
public guides depend on it.

## Commands

From the repository root in PowerShell:

```powershell
cargo run
cargo build
cargo build --release
cargo check
cargo test
cargo clippy --all-targets
cargo fmt
cargo fmt --check
```

To include examples and respect the lockfile:

```powershell
cargo check --all-targets --locked -j 1
cargo test --locked -j 1
cargo clippy --all-targets --locked -j 1
```

`-j 1` reduces parallel compilation if `rustc` runs out of memory; it does not
require a project configuration change. Building does not generate an installer:
see [packaging](packaging.md).

## Logs and environment

Logging uses `env_logger`; `dotenvy` optionally loads `.env`. Path overrides are
documented under [data](data.md) and [preferences](appearance.md).

```powershell
$env:RUST_LOG = "mxchro=info"   # CRUD and lifecycle
cargo run
$env:RUST_LOG = "mxchro=debug"  # Activation and self-interference protection
cargo run
$env:RUST_LOG = "mxchro=trace"  # Keyboard and mouse events
cargo run
```

## Testing rules

- Keep all test code and fixtures in `tests/`, not alongside implementation.
- `tests/application.rs` tests the public API; shared fixtures live in
  `tests/support/` and area-specific unit tests in `tests/unit/`.
- To test private details, include the test file from its module using `#[cfg(test)]`
  and `#[path]`. Only declarations belong in `src/`, not test bodies or fixtures.
  Do not expose private logic solely for testing.
- Test the worker with injected channels and repositories, without calling
  `bootstrap::run`: it opens the real database and installs global hooks.
- Use a fake `InputBackend` and in-memory storage or an exclusive temporary
  database. Do not install hooks, inject OS input or touch user data.
- Engine changes must cover persistence, index cleanup, injected input protection
  and the hook not waiting for the CRUD lock.
- UI changes also require relevant startup/render and interaction checks.
  Helper tests are not equivalent to complete graphical testing.
- Native smoke tests must use empty temporary data, close only their own process
  and never activate real macros without explicit authorization.
- Keep architecture tests against reverse dependencies and DTO regressions;
  engine tests must support in-memory storage.

`tests/architecture.rs` analyzes Rust imports and paths with `syn`. It does not
isolate crates or verify arbitrary macro-generated code. Full methods and commands:
[tests/README.md](../tests/README.md), [native fixtures](../tests/native/README.md)
and [profiling](../tests/perf/README.md).

## Warnings and validation scope

Inherited Clippy and MSVC linker warnings remain (`LNK4098`, conflicting native
CRT libraries). Report them; do not hide them or make broad native changes without
investigation. Previous checks are recorded in [validation](validation.md), with
pending work in [TODO](../TODO.md).
