# Mxchro

A native **Windows** keyboard and mouse automation application, written in
**Rust + GPUI-KIT** and distributed as a single executable.

Create Once/Hold/Toggle macros with key press, release and wait actions; organize
them into collections; configure hotkeys and capture keys from the UI.
Includes a virtualized drag-and-drop action editor, dark themes and local storage.
The interface is in English.

## Quick start

Requires Rust, Windows/MSVC build tools and the C++/LLVM environment needed by
the native RocksDB/PoloDB bindings. From the repository root:

```powershell
cargo run
```

Package/crate: `mxchro`; release executable: `target/release/mxchro.exe`.
Build without launching: `cargo build --release`.
To generate the NSIS installer, see [packaging](docs/packaging.md).

## Data and safety

The database and preferences default to `%APPDATA%/mxchro-native/`, or
`./mxchro-native/` without APPDATA. Data from the original project is not imported.
**The inherited schema reset can delete data**: read
[paths, overrides and precautions](docs/data.md) before using an existing database.

New macros are saved disabled. Review their actions before enabling them;
long waits still do not guarantee an immediate stop.

## Documentation

| Topic | Guide |
| --- | --- |
| Editor, capture and collections | [Usage](docs/usage.md) |
| Building, logs and safe testing | [Development](docs/development.md) |
| Installer and executable icon | [Packaging](docs/packaging.md) |
| Layers, data flow and code map | [Architecture](docs/architecture.md) |
| Components, state and action list | [UI](docs/ui.md) |
| Hooks, hotkeys and concurrency | [Engine and input](docs/engine.md) |
| Paths, persistence and reset | [Data](docs/data.md) |
| Themes, logos, settings and language | [Appearance](docs/appearance.md) |
| Limitations and previous checks | [Validation](docs/validation.md) |

[Documentation index](docs/README.md) · [Tests](tests/README.md) ·
[Status and planned work](TODO.md)

Implemented does not mean physically validated or ready for distribution.
