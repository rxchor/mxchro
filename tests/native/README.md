# Opt-in native checks

```powershell
cargo run --example collection_hotkey_ui
cargo run --example collection_name_ui
cargo run --example sidebar_navigation_ui
cargo run --example hotkey_rules_ui
cargo run --example input_capture_ui
cargo run --example confirm_dialog_ui
cargo run --example toggle_feedback_ui
cargo run --example checkbox_ui
cargo run --example notifications_ui
cargo run --example settings_ui
cargo run --example grid_layout_ui -- --check
```

Each check opens a separate window and exits automatically. No real backend,
database, hooks or OS input injection: clicks are dispatched only to its GPUI
window. `settings_ui` is the storage exception: it uses an exclusive temporary
preferences file, never APPDATA or a macro database.

The collection hotkey fixture checks the real toolbar/selector: rendering,
`disabled`, clearing by click, dropdown selection, authoritative restoration after
a simulated failure, snapshot replacement/clearing and retained entities.
Programmatic synchronization must not emit `Confirm` or trigger another save.
Click coordinates depend on the current layout; review them if the toolbar changes.

`All macros` uses collection boxes; toggle/confirmation/notification/capture/
settings fixture coordinates include that container. `toggle_feedback_ui` checks
interaction inside a collection box and navigation to the individual filter;
other fixtures also cover `No collection`. Grouping tests live in
`tests/unit/view/ui/home_page.rs`: ordering, duplicate names, empty groups,
ungrouped macros, orphaned references, collection hotkey projection and typed
conflict owners. `toggle_hotkey.rs` covers absent assignments, labels and
interpolation. `toggle_feedback_ui` renders a collection hotkey and an inactive
legacy grouped individual hotkey; `notifications_ui` renders an independent
individual hotkey while preserving interaction checks.

These fixtures do not test the worker or real hooks. `tests/application.rs`
covers commands, persistence, validation, indexes and toggle requests using fake
input. Physical hotkeys with/without focus and a full restart flow remain pending.
Engine/application tests cover collisions with fake input.

## Sidebar navigation

`sidebar_navigation_ui` uses MxchroApp and fake channels. It compares filter pixels
in create/edit against Settings (no selection), exercises all three sidebar
destinations and verifies through DTOs that leaving discards the draft without
saving and New macro inherits the chosen list's collection. It checks retained
drafts through Settings, save/query/unsolicited-refresh locks and sidebar clicks
under a confirmation overlay. `input_capture_ui` covers navigation during editor
capture: cancel the session, discard the editor and use the chosen collection's
hotkey. No hooks, OS input or real storage.

## Collection name forms

`collection_name_ui` uses real MxchroApp/Popover with fake channels. Checks opening
without commands, retained input focus/entity across refreshes, Enter/single
submit, pending not released by unsolicited snapshots, inline validation/retry,
Escape/outside cancellation, prefilled rename, accepted-response closing and
invalidation when the collection disappears. No hooks/database.
`notifications_ui` also checks name errors inside the popover.

## Hotkey rules and legacy configuration

`hotkey_rules_ui` renders the real grouped form and all three persistent warning
types. It checks retaining an unknown legacy key, `Clear individual hotkey`
respecting `disabled` and explicit clearing preserving the selector entity and
collection. No macro saving/execution. `collection_hotkey_ui` also covers unknown
hotkey restoration and clearing by click. `notifications_ui` checks duplicate
hotkey rejection as inline validation rather than a general notification.
These checks do not validate physical input or the user's visual review.

## Keyboard and mouse capture

`input_capture_ui` mounts MxchroApp with a fake `KeyCaptureBackend`. It clicks real
recording icons for trigger, action, macro hotkey and collection hotkey; checks
submitted DTOs, press/release creation, preserved values on unknown VK/cancellation
and one collection autosave. Covers locked controls, grouped hotkeys, editor
replacement, 10-second timeout and closing. Focus-loss testing opens/closes a
second fixture-owned window. No hooks or OS input. Explicit quit also verifies
cleanup after the last window closes. `MXCHRO_CAPTURE_DIR` optionally saves captures
of its own window through the shared helper; the directory must already exist.
Checks listening notification/removal, unavailable/busy/unknown-VK/timeout errors,
dismissal without reappearance, and retaining/manually clearing the dropdown
after a capture failure.

Real VK mapping and down/repeats/up suppression are tested separately with fake
unit-test events. Physical input and privilege-level validation remain pending.

## Deletion confirmation

`confirm_dialog_ui` uses MxchroApp, fake channels and real GPUI-KIT Dialog. Checks
single-click opening, Escape/initial Enter/overlay cancellation, Tab+Enter confirm,
retained focus after rebuilding, single commands and pending under unsolicited
snapshots. Covers macros/collections and immediate action removal without a dialog,
draft-only until Save, respecting pending. Removing the target closes the modal
without stale deletion. No hooks or persistence.

`notifications_ui` also uses the dialog before simulating deletion results:
opening/canceling must not announce success, and refreshes must not release
pending. `support/dialog.rs` shares keyboard events for the real modal.

## Full-card checkbox

`checkbox_ui` mounts the editor's three controlled `AppCheckBox` components.
It obtains actual bounds after layout and checks clicks in empty space, top/left
padding, border, indicator and label. Every click must emit exactly one change,
reflected by the owner. Also checks full width, Tab/Space focus/activation and
pointer/keyboard blocking with `disabled`. Only GPUI events in its own window;
no backend, database, hooks or OS input.

## Grid-layout experiment

```powershell
cargo run --example grid_layout_ui           # Interactive; close with X
cargo run --example grid_layout_ui -- --check # Event/CPU checks; exits automatically
```

An isolated experiment, not a production replacement. The left side uses real
`ActionsList`; the right uses one-column `grid_layout`, 50 px rows, 8 px gaps,
vertical compaction and no resizing. Both share a temporary sequence with stable
IDs, including press/release steps, and sync on accepted changes. Previews do not
modify the sequence before drop; sorting areas by row maps grid layout to sequence.
The fixture reports if this differs from single-step insertion. Removing an action
updates both views. The grid keeps native animation/outside-drop commit behavior;
no autoscroll is added.

Controls reset to 25/100/500 steps, show one/both views and lock editing. No `.env`,
APPDATA, database or macro backend is loaded. `--check` dispatches events only to
that window for dragging, Escape, outside drop and disabled controls, then draws
each alternative separately with 25/100/500 steps and five samples per size.
Reports median CPU time for `Window::draw(...).clear(...)`, **not FPS/GPU
presentation**. Simplified grid cards lack some `ActionsList` controls: values
are indicative, not identical-widget comparisons or full equivalence tests.
Visual feel and adoption remain undecided.

Local debug sample: five drawings per panel/size, CPU milliseconds, not FPS;
different card controls:

| Actions | Current list | Grid prototype |
| --- | ---: | ---: |
| 25 | 8.8 | 8.4 |
| 100 | 9.9 | 27.2 |
| 500 | 10.3 | 128.9 |

The grid traverses/builds all elements, including those outside the viewport.
These results argue against automatically replacing the virtual list. Outside-grid
commit was also observed; adapting it and checking autoscroll, keyboard behavior,
boundaries and stale revisions requires more work before integration.

## Enable/disable feedback

`toggle_feedback_ui` mounts real `MxchroApp` with controlled channels, without
native backend assembly. Compares its own window's pixels before/during a delayed
reply. In the macro view all pixels must match after moving off hover. Collection
views permit only the selector text/clear-icon's two small regions to differ:
GPUI-KIT retains disabled styling there, but its surface/rest of the window must match.

Checks duplicate clicks, unrelated controls being locked, updated values after
reply, rejection/recovery and disconnection. It does not measure FPS or eliminate
normal renders. Also verifies Edit requests detail by ID, blocks duplicate queries,
opens on `MacroEditDTO` and returns to summary DTOs after an accepted save.
Persistence is not simulated in this flow. Click points depend on current layout.

## Notifications and validation

`notifications_ui` mounts `MxchroApp` with controlled channels at 1160×780.
Checks quiet toggles, real editor save/reject/retry, deletion success only after
its response, and unsolicited refreshes neither releasing controls nor announcing
success. Verifies errors persist beyond the timeout, identical-error replacement,
real close button, success autohide (6 seconds with the pointer away), inline
trigger/name validation and disconnection not reappearing after dismissal.
Uses public GPUI-KIT notification-stack counts and its own window events;
no hooks, OS input, database or worker assembly.

`toggle_feedback_ui` continues checking by pixels that error notifications do not
hide failures and pending toggles do not dim controls. Notifications overlay the
page without shifting its coordinates.

## Settings, themes and logo

`settings_ui` mounts the application with fake macro channels and the real TOML
repository in an exclusive temporary directory. Exercises three palettes/four
logo choices, color/token synchronization, write failure without success feedback,
retry, replacement and worker file reopening. Preferences must not send engine
commands; Back must retain the macro draft/sequence. On exit it checks the last
choice from the file and removes only its own temporary directory. No hooks/input injection.

Also installs the real icon adapter exclusively on its window: queries `WM_GETICON`,
compares small/large `HICON` pixels, checks fixed logos are not regenerated on
theme-only changes, rejects invalid SVG and restores borrowed icons/releases owned
handles when removing the adapter. Replaced icons must also be released. It never
inspects other windows or changes EXE icons, shortcuts or taskbar settings.
Additional unit tests cover unknown/invalid values without reset/rewrite, defaults
without creating files and replacement failures preserving data.

Clicks/masks match the current 1160×780 layout; review them if it changes. A Windows
desktop with working native rendering is required. The capture helper uses Win32
only inside `tests/` and selects exclusively a visible fixture-process window,
never another application.

To save diagnostic BMP captures, supply an existing directory:

```powershell
$env:MXCHRO_CAPTURE_DIR = "$env:LOCALAPPDATA/Temp/opencode"
cargo run --example toggle_feedback_ui
Remove-Item Env:MXCHRO_CAPTURE_DIR
```
