# Architecture and code map

[Index](README.md) · [Engine](engine.md) · [UI](ui.md) · [Data](data.md)

## Modular monolith boundaries

One native application, process, executable and Cargo project. Do not introduce
microservices, multiple crates, empty layers or interfaces without a real
responsibility. The UI shares Rust types: it does not need JSON, TypeScript DTOs,
camelCase conversion, invoke services or Tauri events. Serde remains for persistence.

```text
view / infrastructure → application → domain
bootstrap assembles the implementations
```

- `domain`: entities, values and pure rules. No GPUI, Windows, PoloDB, disk,
  environment, channels or external-layer imports.
- `application`: use cases, queries, DTOs, contracts and engine coordination.
  Does not import `view`, `infrastructure`, `bootstrap`, GPUI or PoloDB.
- `application/ports`: input and repository contracts implemented by infrastructure
  or fakes. Inject dependencies; never open databases or install hooks from the
  domain, use cases or engine constructors.
- `infrastructure`: Windows input, simulation, capture, icons and storage.
  Only its repositories know PoloDB, BSON and transactions.
- `view`: presentation, editable state and application-facing adapters. No concrete
  infrastructure imports. Errors are translated only in this layer.
- `bootstrap.rs`: assembles persistence, managers, channels, hooks, workers and
  the window; `main.rs` delegates. Keep disk access and assembly off the GPUI thread.
- Separate CRUD, lightweight input processing and engine execution. Organize
  backend/home/editor/selector handlers without moving logic into rendering.
- Pages and components do not import `MxchroApp`, managers or the backend: they
  receive data and emit events through `EventHandler<E>`. `MxchroApp` owns global
  state and translates those events; do not concentrate page renders there again.
- Preserve schema, paths, reset policy, locks, command ordering, transactions and
  engine semantics when refactoring.

## Structure

```text
src/
  main.rs                         Delegated startup
  bootstrap.rs                    Native dependency assembly
  lib.rs                          Exports
  domain/
    types.rs                      Macro, Action, Mode, Collection
    validation.rs                 Pure rules and errors
  application/
    commands/                     Dispatch and macro/collection use cases
    dto/                          Summaries, editing details and save input
    ports/                        Input/repository contracts
    engine/
      macros.rs                   State and CRUD
      macros/input.rs             Lightweight input path
      macros/execution.rs         Persistent thread and execution
      collections.rs              Collection coordination
    error.rs                      CommandError, no presentation text
    queries.rs                    Snapshot and editing query
    request.rs                    Command/Query in one ordered queue
  infrastructure/
    persistence/                  PoloDB, documents, schema and transactions
    input/                        Windows mapper, hooks, capture and simulation
    settings.rs                   TOML preferences
    data_path.rs                  Default data location
    window_icon.rs                Window icon adapter
  view/
    backend/                      Channels, worker and error translation
    assets.rs                     GPUI-KIT resources and icons
    i18n.rs                       Language, text and helpers
    theme.rs                      Palettes and logo colors
    ui/
      mxchro_app/                 Composition and responsibility-specific handlers
      components/                 Reusable controls and typed events
      pages/                      Home, Settings and editor-specific pieces
      state/                      Drafts and filters
```

No compatibility aliases for old layers. The visual layer is called `view`,
not `presentation` or `frontend`.

## Types and responsibilities

| Name | Responsibility |
| --- | --- |
| Entity | `Macro`, `Action`, `Mode`, `Collection` in the domain |
| Document | Persisted BSON with `_id`, private to its repository |
| DTO | Rust projection in `application/dto/`, not a BSON document |
| Manager / Repository | Business coordination / storage access |
| Command / CommandOutcome | Use-case request and result |
| CommandError | Typed error, translated in `view/backend/error.rs` |
| Snapshot / Update | Application query / UI adapter publication |
| Draft / Filter | Editable state and UI selection |

Rust types use `PascalCase`; files, functions and fields use `snake_case`.
Projections are named `*DTO`, never `*View`. `Action` and `Mode` remain shared
domain values, without per-layer copies.

## Reading, editing and saving

`Snapshot` contains `MacroDTO` and `CollectionDTO` summaries. Cards and filters
do not receive full entities or sequences: `MacroDTO` carries `actions_count`.

Edit queues `Query::EditMacro(id)`; the worker returns `MacroEditDTO` and only
then is a draft created. The UI saves with `SaveMacroDTO` and preserves `old_key`
to allow changing the trigger. The application converts and validates the input.

`MxchroApp → CommandBus → worker → application/commands/mod.rs → use cases`
in `commands/macros.rs` and `commands/collections.rs`. Synchronous use cases
validate preconditions, coordinate managers and return typed errors, without GPUI,
infrastructure or i18n. Never invoke them from rendering or hooks.

`application/queries.rs` prepares snapshots without knowing channels or presentation.
`view/backend/worker.rs` executes ordered requests without assembling dependencies.
`bootstrap.rs` wires everything together. The edit query shares the command queue;
a missing macro produces an error, not a synchronous GPUI-to-engine query.

`CommandOutcome::MacroSaved` is published only after an accepted save. The UI
requires a completed response and `saved` to close the editor. Unsolicited
snapshots neither confirm success nor release `pending`: [concurrency](engine.md).
When an inherited manager returns `false`, the use case retains a generic failure;
moving layers does not fix hidden persistence errors.

## Where to make changes

| Change | Main location |
| --- | --- |
| Pure rules | `src/domain/` |
| Macros / collections | `src/application/engine/macros.rs`, `macros/`, `collections.rs` |
| Contracts | `src/application/ports/` |
| Keys / hooks / simulation | `src/infrastructure/input/mapper.rs`, `hook.rs`, `simulator.rs` |
| Use cases / queries / DTOs | `src/application/commands/`, `queries.rs`, `dto/` |
| Transport / visible errors | `src/view/backend/`, adapter `error.rs` |
| Persistence | `src/infrastructure/persistence/` |
| UI coordination | `src/view/ui/mxchro_app/` |
| Controls / pages / state | `src/view/ui/components/`, `pages/`, `state/` |
| Theme / language | `src/view/theme.rs`, `i18n.rs`, `locales/en.yml` |
| Startup / EXE icon | `src/bootstrap.rs`, `src/main.rs`, `assets/icon.svg`, `build.rs` |

Keep architecture tests against reverse dependencies and DTO regressions.
Test suite structure and boundaries are described under [development](development.md).
