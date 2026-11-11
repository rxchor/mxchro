# Engine, concurrency and Windows input

[Index](README.md) · [Architecture](architecture.md) · [Data](data.md)

Read before changing the engine, hotkeys, hooks, mapper, capture or simulator.

## Flows and locks

```text
UI event → MxchroApp → CommandBus → worker → use cases → managers/repositories
  → Update { snapshot, error, saved } → UI

Windows input → capture (if active) → process_key_event
  → toggle_request → bounded queue → persistent worker → apply_toggle
  → dirty → queries → UI
```

- One ordered worker processes UI commands. GPUI checks updates every 50 ms;
  the worker checks hotkey changes during its 100 ms waits.
- Only `Update::request_completed` releases `pending` and consumes request
  feedback. An unsolicited snapshot does not confirm success; saving also requires `saved`.
- No disk access, joins, thread waits, prolonged locks or sequence simulation
  from rendering or the GPUI thread.
- Hooks must return quickly: no I/O, joins, blocking sends or CRUD.
  Queue toggles with `try_send`, never execute them inside the callback.
- `process_key_event` reads `input_states` and `toggle_hotkeys` and signals condvars.
  Never acquire the full `macros` map there: CRUD can hold it while joining threads.
- When both locks are needed, always acquire `collections` before `macros`.
  Do not introduce inversions in listing, deletion or toggles.
- Inject input through `Arc<dyn InputBackend>`: Windows in production, a fake in
  tests. Managers use ports; pure rules belong to the domain.

## Threads, modes and stopping

All modes use one execution thread per enabled macro and a condvar.
**Never create threads per key press.**

- Once: `is_executing` prevents reentry while the sequence runs.
- Hold: repeats while `active=true`; physical release deactivates it.
- Toggle: each press changes `active`.
- Call `stop_repeat_thread()` before removing an instance.
- Refresh input/hotkey indexes when creating, editing, enabling, deleting or clearing.

Long waits are not yet immediately cancelable, and releasing held keys when
stopping sequences still needs improvement. Do not promise instant stopping.

## Blocking and self-interference

`block_original_event` blocks only **non-injected** events of enabled macros.
Blocking injected events would prevent actions from reaching the target application.
Preserve `ignore_injected_keys` and these protections even when it is disabled:

- Hold ignores injected KeyUp to avoid deactivating itself.
- Toggle ignores injected KeyDown to avoid toggling itself.
- Once retains `is_executing` protection.

## Actions and validation

The type is `Action` in `domain/types.rs`, not `ActionConfig`:

```rust
pub enum Action {
    Key { key: String, press: bool },
    Wait { duration_ms: u64 },
}
```

Press and release are independent actions. Ctrl+C means press Ctrl, press C,
release C, release Ctrl, with explicit waits as needed. The UI may add both
actions together, but must not merge them in the domain. `MacroInstance.actions_config`
preserves the original sequence for editing.

`trigger_key` is unique. Triggers and hotkeys are single keys, not chords such
as Ctrl+F6; preserve exact mapper names. Validate the trigger, actions, optional
hotkey, positive waits and collection before saving. New macros are saved disabled.
Keep deletion confirmation for macros and collections.

## Hotkeys and legacy data

- Only macros without a collection may have individual enable/disable hotkeys.
- Hotkeys must be unique across independent macros and collections. Reject
  duplicates with typed errors, without replacing owners or dispatching to several targets.
- Preserve legacy grouped, duplicate or unknown assignments; do not delete them
  on load. Project only valid, conflict-free bindings into the lightweight index;
  publish `Snapshot.hotkey_issues` for persistent warnings.
- Inactive grouped assignments do not count as duplicates.
- Recalculate after changes/deletions under `collections → macros` locks. Hold
  the index lock only to swap the map, never during I/O or joins.
- Deleting a collection preserves its macros' legacy individual hotkeys; activate
  them only if valid and conflict-free.
- The grouped editor disables individual hotkeys and provides explicit clearing.
  Selectors retain unknown values so they remain visible and clearable.

Collection controls and autosave are described under [UI](ui.md).

## Capture before engine processing

`infrastructure/input/capture.rs` intercepts keyboard/mouse input before trigger
processing. Atomic state and one token per session; no locks, channels or
allocations in the hook. Resolve names from polling, not from the hook.

- Capture only with the bound window in the foreground and both hooks available.
- Ignore injected events. Do not capture down/repeats or releases of keys already
  held when arming; preserve their KeyUp so an existing Hold macro can stop.
- Suppress captured down/repeats/up, including X1/X2, even if the UI cancels
  between down and up. Capture does not stop sequences already running.
- Escape, 10 seconds, focus loss, navigation, a new editor and closing cancel capture.
- Unknown VK preserves selection and reports an error. Do not invent VK names,
  arbitrary support or chords. Wheel/movement are not key actions.
- `left_shift`, `right_shift`, `left_alt`, `right_alt` are mapper names.
  The simulator preserves E0 for right Alt/Ctrl.

The UI uses an injected `KeyCaptureBackend`; pages/components never install hooks.
Notifications, tokens and selectors: [UI](ui.md#capture-from-selectors).
Unit tests without hooks/SendInput and `input_capture_ui` with a fake port are not
physical validation. Focus and different privilege levels remain pending; do not
port the original WebView2 JavaScript workaround.

Hook installation failures are logged but have no general UI notification;
Capture reports unavailability when attempted without both hooks.
