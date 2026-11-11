# Validation and limitations

[Index](README.md) · [TODO](../TODO.md) · [Tests](../tests/README.md)

This document preserves previous validation context. **It is not a report of a
new run** or certification that every checkout passes these checks. Detailed
methods live alongside the fixtures. Pending work is tracked in `TODO.md`.

## Known limitations

- Physical keyboard/mouse capture, modifiers, layouts and different privilege
  levels require authorized validation. Fakes do not test real hooks.
- Hooks are global; test focus/no-focus and injected-input blocking separately.
  The original WebView2 issue is not inherited as a JavaScript workaround, but
  changing frameworks does not automatically validate hooks either.
- Hook installation failures are logged without a general UI notification;
  Capture reports unavailability when attempted without both hooks.
- Some inherited CRUD operations hide persistence failures in logs. UI success
  means application acceptance, not a new guarantee of disk persistence.
- Long waits lack immediate cancellation, and releasing held keys needs improvement.
- Reset on missing/invalid/different schema and some metadata failures is
  destructive; never test with valuable data: [data](data.md).
- Real NSIS installation/upgrade/uninstallation testing remains pending,
  including retaining/removing data: [packaging](packaging.md).
- Collection reordering, explicit Mxchro import, total-wipe UI and trigger/hotkey
  collisions: check pending tasks rather than assuming they are implemented.
- Collection grouping is implemented; user visual review remains pending.
  Do not confuse missing validation with missing functionality.
- Inherited Clippy and MSVC `LNK4098` warnings (native CRT libraries) need
  investigation. Report them without hiding them or applying uninvestigated broad fixes.

## Recorded automated and native checks

- `cargo test`: validation, use cases, queries, worker, engine, persistence,
  themes, i18n and helpers. Fake input and temporary databases, no native bus startup.
- Build/check and formatting were checked in earlier tasks; this is not a
  guarantee of a current run. Clippy retains inherited warnings.
- Startup/shutdown with an empty temporary database and exit code 0, without
  activating real macros or opening original data. Command refactoring was tested
  with `MxchroApp` and fake channels; use cases/worker were tested separately.
- `collection_name_ui`: real popovers, opening without commands, retained focus/input,
  buttons/Enter, pending lock, inline errors/retry, cancellation, acceptance and stale targets.
- `collection_hotkey_ui`, `hotkey_rules_ui`: selection/clearing, legacy values,
  grouped restrictions, warnings, authoritative restoration and retained entities.
- `input_capture_ui`: real controls with a fake port, keyboard/mouse selection,
  notifications/dismissal, unknown VK/cancellation, timeout/focus/editor/close,
  grouped/pending controls and one autosave; no OS input.
- `confirm_dialog_ui`: macros/collections, safe Enter, Escape/overlay/focus,
  single command, stale targets and immediate draft-only action removal.
- `notifications_ui`: success/errors, dismissal/autohide, deduplication, validation,
  responses versus refreshes and disconnection.
- `toggle_feedback_ui`: pixel captures, blocking without global dimming,
  rejection/recovery and disconnection. `checkbox_ui`: entire card, one callback,
  Tab/Space and disabled controls.
- `settings_ui`: themes/tokens, four logos, small/large Windows icons,
  ownership/restoration, temporary preferences, write failure/retry and retained draft.

## Earlier rendering and performance reviews

- Selectors: render and mode/collection restoration/change/clearing in an isolated
  harness; arrow SVG from assets; catalog changes (rename/delete/empty) retaining
  entities. Coverage does not mean every search interaction is validated.
- Editor layout reviewed at 1160 and 900 px with a long sequence: paired fields,
  side-by-side/stacked panels. PRESS/RELEASE/WAIT indicators at 460/600 px, long
  keys and disabled controls. Key/Wait forms at 300/440 px aligned with the sequence.
- English/i18n: Home/editor rendered with test data; not every error,
  confirmation, crate menu or long label.
- Virtual list: global indexes, Move/Remove, clearing/shortening, disabled,
  drag/Escape/outside drop, internal scroll/revision and motion at 25/100/500 steps.
  Perceived monitor/mouse smoothness and release profiling remain pending.
- Previous debug profile of 100 actions: approximately 77 → 11 ms drawing CPU
  time; **not presentation FPS**. [Method and limitations](../tests/perf/README.md).
- Historical note: enabling `test-support` in **GPUI-KIT 0.9.1** encountered a
  `taffy` 0.15.1 versus 0.15.0 optional-dependency conflict. Native dependencies were
  not changed to bypass it. Do not extrapolate this diagnosis to the current
  version without reproducing it.

Do not run fixtures with hooks or real data to "complete" this list without
authorization. Safety instructions and commands: [development](development.md).
