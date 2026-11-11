# Windows packaging

[Index](README.md) · [Build environment](development.md) · [Data](data.md)

## Generate the NSIS installer

Packaging uses **cargo-packager with NSIS**, with the same Rust/MSVC and C++/LLVM
environment as application builds. From the repository root:

```powershell
# Install the tool once, if it is not available
cargo install cargo-packager --locked

# Build in release mode and generate the .exe installer
cargo packager --release --formats nsis --binaries-dir target/release --out-dir target/packages
```

`Cargo.toml` configures `before-packaging-command = "cargo build --release"`,
so a separate build is unnecessary. The executable is `target/release/mxchro.exe`;
the command above places packages in `target/packages/`.
`cargo build --release` alone does not create an installer.

Before distribution, test installation, startup, upgrades and uninstallation in
an isolated Windows environment, both retaining and explicitly removing user data.
Full validation is still pending; producing a package does not replace it.
Do not use the user's real database.

## Uninstallation and data

`Cargo.toml` declares only `$APPDATA/mxchro-native` in `appdata-paths`.
The uninstaller offers to remove this directory through a checkbox that is
unchecked by default. If selected, it removes the preferences and database inside.

Do not register external custom paths, the old working-directory database or
the local fallback without APPDATA. Do not add shared directories or delete data
during upgrades. Tests cover the configuration; real installation, upgrade and
uninstallation validation remains pending.

## Executable icon

The editable source is `assets/icon.svg`. `build.rs` generates an `.ico` with
16, 24, 32, 48, 64, 128 and 256 px sizes and embeds it as Windows resource ID 1.
Conversion uses Rust crates, without external conversion tools.

To change the icon, replace the SVG and run `cargo build` or
`cargo build --release`; Cargo regenerates it automatically. The intermediate file
is `target/<profile>/build/mxchro-*/out/mxchro.ico`, not a copy under `assets/`.
Keep SVG text converted to paths: conversion does not load fonts.

The base SVG and EXE resource are Monochrome. Runtime logo variants and window
icons are described under [appearance](appearance.md). Windows may cache a pinned
shortcut's icon: if rebuilding/restarting does not update it, unpin and repin the
executable. The application must not modify other windows or force cache cleanup.
