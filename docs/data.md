# Data and persistence

[Index](README.md) · [Preferences](appearance.md) · [Installer](packaging.md)

Read before changing persistence, paths, schema, reset or migrations.

## Locations and overrides

The database and preferences share a default directory but remain independent:

```text
%APPDATA%/mxchro-native/
  settings.toml
  macros.polodb/     # PoloDB storage
```

Without APPDATA, or when it is empty: `./mxchro-native/`. The helper is
`src/infrastructure/data_path.rs`; preferences are never stored in PoloDB.
`.env` is loaded optionally; see [`.env.example`](../.env.example).

This does not move, import or delete existing data. The old `./macros.polodb`
database is not loaded by default; `MACROS_FILE_PATH=.` preserves that location
when launched from the same working directory. Do not automatically open, import,
modify or reuse the original Mxchro database.

`MACROS_FILE_PATH` retains its inherited layout: normalize the supplied path's
extension to `.polodb` and append `/macros.polodb` inside it. For example,
`./custom.db` resolves to `./custom.polodb/macros.polodb`.
Do not reinterpret or "correct" explicit paths by inference.

`MXCHRO_SETTINGS_PATH` independently overrides the TOML file: preserve the exact
path and extension rather than applying the database layout. Examples and relative
path behavior are described under [appearance](appearance.md).

## Reset warning

**Do not point at original storage without a backup.**
`SCHEMA_VERSION` is in `src/infrastructure/persistence/mod.rs`. A missing, invalid
or different version causes data deletion; some metadata read failures can also
trigger it. `RESET_DB=true` or `1` forces a reset.

- Do not increment the schema or use reset as a routine fix.
- Do not change schema, paths, reset policy or persistence semantics during a
  structural refactor.
- Tests must use an exclusive temporary database, never the user's database.
- Agree on an explicit strategy and protect data before a destructive migration.
  Importing from Mxchro is still pending.

## Storage contracts

- Only `infrastructure/persistence` knows PoloDB, BSON, schema and transactions.
  Its documents are private, not DTOs or domain entities.
- `MacroDocument` uses `trigger_key` as `_id`; `CollectionDocument` uses the collection ID.
- Managers load data on startup and receive injected repositories. Use granular
  operations rather than rewriting all macros after every change.
- Macro `replace` uses deletion followed by insertion: do not assume atomicity or
  disk persistence merely because in-memory state changed.
- Enabling/disabling a collection persists the collection and associated macros
  in one transaction. Keep that as a single port operation; do not replace it
  with separate use-case calls.
- Deleting a collection preserves its macros without a collection, in memory and
  on disk. Keep hotkey cleanup/reprojection as described under [engine](engine.md).
- Some inherited CRUD operations only log persistence failures without fully
  propagating them. Do not present notifications or a refactor as a validated fix.

TOML preferences use a separate repository and worker: [appearance](appearance.md).
Optional NSIS data removal is limited to the default directory:
[packaging](packaging.md#uninstallation-and-data).
