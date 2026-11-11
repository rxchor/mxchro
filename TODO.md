# TODO — Mxchro

Migration roadmap from the original Mxchro (Tauri) to Rust + GPUI-KIT.
The original TODO is a reference, not a checklist to copy literally: its UI must
be reimplemented, while part of the backend has already been reused.

## How to read this list

- `[x]`: implemented in native Mxchro; not necessarily fully validated graphically
  or with real input.
- `[ ]`: implementation or the stated validation is pending.
- Distinguish Mxchro parity, technical improvements and future ideas.
- Review the original code before implementing a feature. This list is a starting
  point, not an exhaustive parity audit or a finalized design.
- Preserve the existing backend where appropriate instead of duplicating it.
- Topic guides: [docs/README.md](docs/README.md). This file tracks task status;
  it does not replace technical rules or testing methods.

## 1. Available native foundation

- [x] Native startup/window with GPUI-KIT, without Tauri, React or WebView2.
- [x] Monochrome, Catppuccin Mocha and Rosé Pine dark themes centralized in
  `src/view/theme.rs`, selectable in Settings and applied through `Theme::update`.
- [x] Theme-following or fixed logo (three palettes), reusing `assets/icon.svg`.
  Base SVG and EXE icon are Monochrome; redundant Original option removed.
- [x] Small/large Windows window icons follow logo choice through an injected port,
  in-memory rasterization and owned native resources. `settings_ui` checks pixels,
  fixed/follow behavior, invalid SVG without replacement and handle restoration/
  release. Final taskbar/pinned-cache appearance still needs manual Windows review.
- [x] Preferences in a standalone APPDATA TOML file, separate from PoloDB, with a
  write worker, errors and retry. Live changes, all four logo choices, retry,
  reopening and retained editor draft tested in `settings_ui`; unit tests cover
  defaults, invalid files without reset, replacement and data-preserving failures.
  User visual review remains pending.
- [x] Choose the preferences file with `MXCHRO_SETTINGS_PATH` (environment or `.env`),
  absolute/relative paths and empty-value fallback, without migrating the old file
  or changing macro storage paths.
- [x] Unify settings/database defaults under `%APPDATA%/mxchro-native/` (fallback
  `./mxchro-native/`) with a shared infrastructure helper. Preserve overrides and
  inherited layouts without moving/deleting old data.
- [x] Declare the default directory in cargo-packager/NSIS `appdata-paths` for
  optional settings/database removal, unchecked by default.
- [ ] Validate the real NSIS installer/uninstaller, including data-preserving
  upgrades, retaining/removing data on uninstall and custom paths.
- [x] Windows icon generated automatically from `assets/icon.svg` by `build.rs`,
  in seven sizes and embedded in the executable. ICO sizes and EXE extraction
  checked; taskbar/pinned appearance still needs manual review.
- [x] Separate app, pages, reusable components and editable state.
- [x] Documented modular monolith with Clean Architecture boundaries: `domain`,
  `application`, `infrastructure`, `view`, with assembly only in `bootstrap.rs`.
  Repositories injected through ports; PoloDB outside domain/application.
- [x] Application-owned DTOs used by the UI: `MacroDTO`/`CollectionDTO` summaries,
  separate `MacroEditDTO` query and `SaveMacroDTO` input; list snapshots do not
  carry entire action sequences.
- [x] Separate lightweight input/execution and UI coordination handlers; engine
  tests use in-memory storage and AST dependency checks. No schema, data-path,
  transaction or reset-policy changes.
- [x] Shared `AppCheckBox`, `AppInputField`, `AppSelect` and `AppNotice` controls.
- [x] Rust command/update channels with the worker off the UI thread.
- [x] Use cases in `application/commands/`, separate queries and typed errors;
  `CommandBus` adapter dedicated to channels, worker and native assembly.
- [x] Tests for dispatch, CRUD, validation, queries, worker ordering, errors,
  save outcomes and external refreshes, with fake input and exclusive temporary DBs.
- [x] Centralize tests in `tests/`: application integration, shared fixtures and
  area-specific unit tests, without exposing private logic for tests.
- [x] Reuse managers, PoloDB repositories, mapper, hooks and Windows simulator.
- [x] `InputBackend` contract separates domain/infrastructure and allows fake input.
- [x] Native application data separate from the original Tauri Mxchro data.
- [x] Rename package/crate/executable and internal references to `mxchro`, with
  `MxchroApp` as root entity, without moving the repository or changing data paths.
- [x] Short README, essential rules in local `AGENTS.md` and topic guides in `docs/`,
  with task-specific agent reading and test instructions beside fixtures.

## 2. UI feature parity

### Macros and actions

- [x] List, create, edit, enable/disable and delete macros with confirmation.
- [x] Consistent sidebar during creation/editing: no selected filter; choosing
  All macros/No collection/a collection discards the draft like Cancel and opens
  that list without writing or another confirmation. Settings retains the draft.
  Pending/loading/confirmation block navigation; navigating cancels capture.
  Checked with `sidebar_navigation_ui` and `input_capture_ui`, without hooks/DB.
- [x] Group `All macros` into named collection boxes with counts, in catalog order,
  no empty groups and `No collection` last. Orphaned references remain visible
  without modifying data; individual filters retain their behavior. Grouping,
  ordering, duplicate-name and empty-state tests plus native toggle/confirmation/
  notification/capture/settings fixtures use no real data. User visual review pending.
- [x] Show macro mode with GPUI-KIT `Tag`, separate action count and no repeated
  collection name in card descriptions.
- [x] Show assigned enable/disable hotkeys beside mode/actions and in collection
  headers, including the individual collection filter. Informational `Enable/disable`
  tags mark incompatible/grouped assignments inactive; no engine, persistence or
  chord-support changes.
- [x] Show `Enabled`/`Disabled` through GPUI-KIT outline `success`/`danger` tags,
  following the active palette without changing activation logic.
- [x] Centralize macro/collection deletion in `AppConfirmDialog`, based on GPUI-KIT
  Dialog, replacing double-click and Delete/Confirm labels. Cancel gets initial
  focus; Escape/X/overlay cancel. One deferred decision, typed targets, modal locking
  and stale-target checks. Action removal remains immediate, without a dialog,
  draft-only until Save. Checked in `confirm_dialog_ui` without real data.
- [x] Expose Once/Hold/Toggle, original-input blocking and ignoring injected input.
- [x] Independent press/release/wait actions and adding press/release pairs together.
- [x] Reorder actions with buttons and remove actions.
- [x] Replace Remove text with a square X icon button, preserving tooltip,
  accessible name and disabled locking.
- [x] Global enable/disable hotkeys only for ungrouped macros, with optional
  selector and explicit clearing of legacy grouped assignments.
- [x] Reject duplicate hotkeys across independent macros and collections. Preserve
  incompatible old configuration: all eligible duplicates, grouped assignments
  and unknown keys stay inactive with owner/key/reason warnings. Collision,
  concurrency, deletion/unassignment, resolution and write-failure tests use
  in-memory ports. Inherited general CRUD persistence errors remain unresolved.
  Native checks: `hotkey_rules_ui`, `collection_hotkey_ui` and inline duplicate
  rejection in `notifications_ui`, without hooks/DB. User review and physical input pending.
- [x] Searchable trigger/action selectors, preserving mapper names with readable labels.
- [x] Capture beside trigger/action/hotkey selectors without replacing or recreating
  their entities. Uses existing keyboard/mouse hooks and shared VK mapper; explicit
  draft save and collection autosave. Unknown VK reports Unsupported key without
  changing selection. Escape/focus/timeout/navigation/new editor/close clean up.
  Ignore injected input and repeats of previously held keys; intercept before
  engine processing. No wheel/movement/chords. Added side-specific Shift/Alt and
  extended right Alt/Ctrl simulation flags. Unit tests without hooks and
  `input_capture_ui` with a fake port cover selection, actions, cancellation,
  timeout/focus/close, grouped/disabled controls and one autosave.
- [x] Replace Capture text with a recording icon, retaining field-specific tooltip
  and accessible name. Listening/errors use InApp notifications, without new
  panels or inline capture errors. Remove listening on completion/cancellation;
  dismissal checks the session token to avoid canceling a later session.
  Tested in `input_capture_ui`.
- [ ] Validate capture with physical keys, X buttons, modifiers/AltGr, real keyboard
  layouts and different privilege levels.
- [x] Replace cycling mode button with a Once/Hold/Toggle selector.
- [x] Allow clearing every selected value; prevent saving with the mode cleared.
- [x] Replace cycling collection button with searchable/clearable selection (`No collection`).
- [x] Drag-and-drop reordering through a dedicated handle (`action_drag.rs`), retaining
  virtualization, insertion marker, preview, edge autoscroll, Escape and ↑/↓ keyboard
  alternatives. Hide the source card without losing its slot/marker; restore on
  drop/cancel. Destination uses a theme-colored dashed card without row-height
  changes. Neighbors open a gap through an interruptible 150 ms transition
  (`action_motion.rs`) without editing the draft until drop. Keep one extra card
  per edge. Native fixture checks arbitrary moves, cancellation, disabled/stale
  revisions, virtual indexes/scroll, quick retargeting, unchanged draft before drop
  and drawing with 25/100/500 actions; no backend/hooks/real input. User-perceived
  smoothness and release profiling remain pending.
- [ ] Consider full-card dragging without interfering with its buttons.
- [x] Isolated `grid-layout` comparison prototype: `cargo run --example grid_layout_ui`,
  shared fake data, 25/100/500 steps, one column, no resizing, keyboard selection
  and disabled mode. Production is unchanged. `--check` tests drag/Escape/outside
  drop and drawing CPU time.
- [ ] Decide whether to adopt `grid-layout` after visual/cost evaluation. It is not
  virtualized, commits outside drops and lacks required autoscroll. Verify insertion,
  ordering and all protections before replacing anything. Prototype cards are
  simplified: timings are neither identical-widget comparisons nor FPS measurements.
- [x] Internal action-list scrolling and compact layout.
- [x] Omit card/preview numbering; execution order is shown top to bottom.
- [x] General in-app GPUI-KIT notifications: persistent/dismissible/deduplicated
  errors, temporary success and quiet toggles. Validation beside fields/lists;
  distinguish replies from unsolicited snapshots to avoid early success or pending
  release. Save/delete, validation, real dismissal, autohide and disconnection
  tested in `notifications_ui` without real backend/data. Inherited backend error
  limitations remain.
- [x] Virtualized action rows in a retained entity without changing cards/commands;
  local before/after debug profiling in `tests/perf/README.md` measures CPU, not FPS.
- [x] Boxed icons with separate PRESS/RELEASE/WAIT labels and key/duration, following
  Mxchro; native rendering reviewed at 460 and 600 px panel widths.
- [x] Preserve panels and sequence height for empty/content states; show a draggable
  vertical scrollbar on overflow. Synthetic GPUI dragging checked; real mouse/
  monitor feel remains pending.
- [x] Separate `Key action` and `Wait` cards; native rendering reviewed at 300/440 px
  with enabled/disabled controls.
- [x] Align the two add-card heights with the sequence and distribute all three key
  buttons in one full row; native rendering reviewed at 300 px.
- [x] Group fields/options and arrange sequence/add panels side by side, stacking
  at narrow widths, using original Mxchro as reference.
- [ ] Review visual equivalence of actions, states, options and hotkeys with the original.
- [ ] Expose backend total wipe with explicit confirmation and data protection.

### Collections

- [x] Create, rename, delete and enable/disable collections from the UI.
- [x] Filter all macros, ungrouped macros or one collection.
- [x] Assign/change a collection from the macro editor.
- [x] Preserve macros without a collection when deleting their collection.
- [x] Configure, show and clear collection hotkeys from the UI with retained selector
  and worker autosave. Assignment/replacement/clearing/validation/persistence/index
  tests use fake input; native fixture covers selection/clearing, disabled controls
  and snapshot reconciliation. Backend collision checks pass; physical focus/no-focus
  hook validation remains pending.
- [ ] Expose persisted collection reordering.
- [x] Visually group collections in All macros.
- [ ] Restore a shortcut to move macros directly from their cards.
- [x] Create/rename with `CollectionNamePopover` (GPUI-KIT Popover), without a permanent
  sidebar field. Prefill current name on rename; retained input, initial focus,
  Enter/Cancel/Escape/outside click, inline validation, pending lock and close after
  accepted reply. Rejections retain draft; navigation/editor/capture/confirmation
  and lost targets invalidate it. `collection_name_ui` checks focus, refreshes,
  single submit, rejection/retry, cancellation, success and stale target without
  hooks/DB. Visual review pending.

## 3. Checks required to complete migration

- [x] Unit tests for engine, persistence, mapper, validation, theme and UI helpers.
- [x] Check startup/shutdown with an empty temporary database.
- [ ] Test page/component interactions, including focus and keyboard navigation.
  - Editor checkboxes: the whole card toggles, including padding/border; indicator,
    label, one callback, Tab/Space and disabled checked with
    `cargo run --example checkbox_ui`, without backend/hooks/real input.
  - Manual toggle: avoid global dimming without removing `disabled`. Native pixel
    comparisons for macros/collections cover slow replies, duplicate blocking,
    rejection/recovery and disconnection in `cargo run --example toggle_feedback_ui`
    without hooks/DB/real input. User experience in the real environment still needs review.
  - Selectors: native rendering/state changes checked without backend/hooks/DB;
    initial collection, editing/clearing and rename/delete/empty-catalog changes
    retain selector entities.
  - Editor layout reviewed at 1160/900 px with a long sequence; complete scroll/focus,
    search, selection, clearing and disabled interaction coverage remains pending.
    Historical note: GPUI-KIT 0.9.1 `test-support` encountered a `taffy` conflict;
    reproduce before assuming it affects the current version. Native dependencies
    were not changed to bypass it. See [validation](docs/validation.md).
  - Virtual list: global indexes, Move/Remove, disabled, boundaries, shortening/
    clearing and dragging checked with GPUI events without backend/hooks/DB.
    Compare smoothness/FPS on real monitor/mouse and in release mode.
- [ ] Manually validate Once/Hold/Toggle and mouse buttons with authorized real input.
- [ ] Validate original-input blocking, injected input and hotkeys with/without focus.
- [ ] Check applications running at different privilege levels.
- [ ] Check through the UI that edits, trigger changes and collections survive restart.
- [ ] Review the original screen by screen and close remaining parity gaps.
- [ ] Verify release builds and execution outside the development environment.

The WebView2 focus bug is not carried forward as a pending JavaScript workaround:
native Mxchro does not use WebView2. Hook behavior needs separate tests.
Press/release **output** combinations are already possible; they do not imply
triggering a macro with a chord such as Ctrl+F6.

## 4. Robustness and data transition

These tasks do not imply they were solved in original Mxchro.

- [ ] Show hook installation failures and other operational errors in the UI.
- [ ] Fully propagate persistence errors and prevent memory/disk inconsistencies.
- [ ] Improve long-wait cancellation, stopping sequences and releasing held keys.
- [ ] Validate collisions between triggers and macro/collection enable hotkeys.
- [ ] Design explicit, safe import from a copy of original Mxchro data.
- [ ] Review destructive schema-change/error reset before using valuable data.
- [ ] Investigate MSVC `LNK4098` and native warnings on window close.
- [ ] Resolve inherited Clippy warnings without accidental behavior changes.

## 5. Ideas inherited from the original TODO — not parity blockers

These were proposals, not complete features that must be ported. Scope and
priority will be decided after the native foundation.

- [ ] Real-time macro recording.
- [ ] Activate macros based on the focused application/window.
- [ ] Profile import/export, separate from one-time data migration.
- [ ] Global hotkey to temporarily pause/resume all macros.
- [ ] Notifications when enabling/disabling macros and collections.
- [ ] Execution history and usage statistics.

Repeat timing can already be expressed with `Wait` actions. An additional speed
control would be a new product decision, not a technical migration requirement.

## 6. Additional Mxchro proposals

- [x] i18n foundation with `rust-i18n` and `locales/en.yml`: English-only initially,
  also in GPUI-KIT, without a selector or automatic language detection.
- [x] Extract visible labels/errors with interpolation and singular/plural counts,
  without changing IDs, mapper names or persisted values.
- [x] Key/catalog/fallback/interpolation/count tests; native Home/editor rendering
  checked with fake data, without backend/hooks/DB.
- [ ] Manually verify all English UI states: errors, confirmations, crate menus
  and long text.
