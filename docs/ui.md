# UI implementation rules

[Index](README.md) · [Architecture](architecture.md) · [Usage](usage.md) · [Appearance](appearance.md)

Read before changing `src/view/ui/`, visual feedback or UI/backend coordination.
This guide defines implementation rules; [usage](usage.md) describes interaction.

## Composition and state

- Consume GPUI through `gpui_kit`, without direct internal-crate dependencies
  when the facade already provides the required API.
- Prefer controlled `RenderOnce` components; reserve entities for state or
  lifecycle requirements. Pages/components do not know `MxchroApp`, managers or
  the backend: pass data and `EventHandler<E>` with typed events.
- `view/ui/mod.rs` declares modules and exports `MxchroApp`. Root composition is
  in `mxchro_app/mod.rs`; backend/home/editor/collection handlers are separated
  by responsibility. Do not assemble dependencies or move handler logic into rendering.
- `MxchroApp` owns state, applies events and calls `cx.notify()`. `MacroDraft`
  retains editor state; `MacroFilter` owns filtering.
- Shared controls belong in `components/`; page-specific pieces alongside their
  page, such as `pages/macro_editor/`. Extract repeated logic with descriptive
  names, rather than fragmenting renders without a real responsibility.
- Reuse `AppCheckBox`, `AppInputField`, `AppSelect`, `AppNotice` and
  `AppConfirmDialog`. Keep empty/content dimensions consistent.
- Use GPUI containers and scrolling, not literal CSS translations.
- Respect `pending`/`disabled`: no editing during saves.
- For visible text, follow [language and catalog](appearance.md#language-and-catalog):
  English only, labels/errors in `locales/en.yml`, no translated persisted values.

## Retained controls

- Preserve `Entity<InputState>` across renders; never recreate fields while typing.
- Preserve `Entity<AppSelectState>` in the draft. Search text is not the selected
  value. Use readable labels without changing exact mapper names.
- `AppSelect` shows a dropdown arrow. Register `assets::AppAssets`, including
  default GPUI-KIT assets and action/drag-handle icons. Mode has no search.
- All selectors allow clearing. A missing mode makes `MacroDraft::read_config`
  return an error; never silently restore the previous mode.
- Collection IDs are values, names are labels, and `None` means `No collection`.
  Sync the catalog without recreating entities or losing search on unrelated renders.
- Keep typed callbacks (`MacroOption`, `MacroFormEvent`), not numeric option indexes.
- Use the real GPUI-KIT checkbox: label, focus, keyboard and controlled state,
  not a button with a symbol. `AppCheckBox` applies width/padding/border to the
  checkbox itself, so the entire card toggles exactly once. Do not add a wrapper
  callback. Fixture: `checkbox_ui`.
- `AppInputField` combines label/input/error; `AppNotice` keeps loading and empty
  states in the page. Do not replace them with general notifications.

## Home and collections

The sidebar stays visible during creation/editing, with no selected filter.
The root passes `selected: None` while a draft or Settings is open; do not imply
that the editor's initial collection is a selected navigation item. Clicking
All macros/No collection/a collection closes the draft and subscriptions like
Cancel, then opens that list without saving or another confirmation.
Settings retains the draft and can return to it. Navigation has its own lock
(`navigation_disabled`): pending/loading/confirmation block controls and handlers;
active capture is canceled by navigation instead. Disabled buttons retain their
colors during toggle pending, including selection. `sidebar_navigation_ui` checks
interaction and filter selection through pixels.

`HomePage` groups **only All macros** into collection boxes, in catalog order,
with names/counts, omitting empty groups and placing `No collection` last.
Group by ID, not name. Orphaned references remain visible without changing their
assignment. Individual filters stay flat. Reuse `MacroCard`, events and locks;
do not add backend controls to rendering.

`MacroCard` shows its mode in a GPUI-KIT `Tag` and a separate action count. Do not
repeat the collection name, already visible in its box/filter. The mode tag is
informational, not a control or an enabled-state indicator.
State uses a separate outline tag: `Enabled` with `success`, `Disabled` with
`danger`, using semantic theme colors without local hex values. It presents the
existing `enabled`, not a calculated mixed state.

The second row includes an assigned individual enable/disable hotkey, labeled
`Enable/disable` to distinguish it from the trigger. Collection boxes show their
own hotkey beside the name; individual collection filters show it by the count.
No assignment means no tag or placeholder. `toggle_hotkey_tag` reuses outline Tag;
assignments listed in `Snapshot.hotkey_issues` and grouped individual assignments
show `(inactive)` with warning styling, without inventing or modifying bindings.
Rows may wrap at narrower widths; keep action buttons usable.

Create/rename uses controlled `CollectionNamePopover`, built on GPUI-KIT Popover
and `AppInputField` with retained `CollectionNameDraft`. Do not restore a permanent
sidebar field. `MxchroApp` prefills the current name, validates the typed target
and sends one command. Focus the input on opening; Enter submits, while
Cancel/Escape/outside click cancel without commands. Pending locks the form.
Only an accepted response closes it on success: refreshes neither release pending
nor reset text/focus. Rejections preserve text and inline errors. Navigation,
opening editor/capture/confirmation or losing the target invalidates the form.
If the field is no longer visible, use general error notifications.
Fixture: `collection_name_ui`, without a real backend.

The selected collection's hotkey uses `AppSelect`, autosave and state retained
in `MxchroApp`. Capture the ID in the subscription, respect pending and reconcile
snapshots without restarting searches on unrelated renders. Programmatic selection
must not emit Confirm. Collection reordering still has no UI controls.

## Deletion confirmation

Only macros/collections use `AppConfirmDialog`, based on Dialog and Root's overlay.
Do not restore double-click confirmation or Delete/Confirm labels. Use one typed,
deferred decision; the component remains coordinator/backend-independent.
Builders may run repeatedly: create guards/focus before opening and focus Cancel once.

Enter confirms only when the destructive button has focus; Escape/X/overlay cancel.
`mxchro_app/confirmation.rs` owns token/target, locks the UI, revalidates on confirm
and invalidates when the target disappears or the editor opens. Action removal
is immediate, without confirmation: draft only until Save. Do not add confirmations
to every edit/selector clear or expose wipe/reset without authorization.
Fixture: `confirm_dialog_ui`.

## Feedback and pending requests

General feedback uses GPUI-KIT `Notification`, `InApp` delivery only, from handlers
or polling, never rendering. `mxchro_app/feedback.rs` centralizes:

- Persistent errors until dismissal, deduplicated by replacement.
- Success notifications for macro save/delete and collection CRUD/hotkey changes
  with autohide (the crate's 5-second timeout, paused on interaction).
- No success notification for toggles, opening the editor or hotkey refreshes.
- Validation beside fields/lists, retained in the draft and cleared on edit/revalidation.
- Only `Update::request_completed` releases pending and consumes request feedback;
  save also requires `saved`. Disconnection warns once and keeps controls locked.

Success means acceptance, not a new guarantee about inherited persistence failures.
Hook installation failures still appear in logs. Fixture: `notifications_ui`.

Toggles retain `disabled`, but `toggle_pending` preserves button colors through
`pending_style.rs` to avoid brief global dimming. Do not replace locking with
active buttons that ignore clicks or hide errors. No optimistic domain update or
artificial delay. Selectors retain their surface; disabled text/clear-icon styling
remains native. `toggle_feedback_ui` compares pixels and locking, not only events.

## Capture from selectors

`AppSelect::on_capture` adds a `CircleDot` icon, tooltip and field-specific accessible
name without replacing the dropdown. `MxchroApp` owns session/target and applies
`set_selected_value` to the same entity. Use an injected `KeyCaptureBackend`;
pages/components neither know Windows APIs nor install hooks.

Editor capture does not save; collection capture sends one explicit command
without relying on programmatic Confirm. Disable controls during capture and
respect grouped hotkeys and pending. Listening is persistent per token and removed
on completion/cancellation; errors use persistent/deduplicated general feedback.
No listening panels or inline capture errors; normal validation stays by fields.
When closing the notification, defer its callback and verify the token to avoid
coordinator reentry or canceling a later session. Fixture: `input_capture_ui`;
[hook protections](engine.md#capture-before-engine-processing).

## Editor and action list

- Use `macro_editor/layout.rs`: flex-wrapped fields, side-by-side or stacked
  sequence/add panels. Preserve internal scrolling and avoid full-width numeric fields.
- Keep separate `Key action` and `Wait` cards, not one combined box. Their total
  height including the gap matches the sequence's `ACTION_PANEL_HEIGHT`.
- Both/Press/Release share one full row with complete tooltips/accessible names.
  Actions show a decorative boxed icon and catalog labels PRESS/RELEASE/WAIT
  separate from key/duration, not state controls.
- `ActionsList` is retained in `MacroDraft`, with `uniform_list`,
  `UniformListScrollHandle` and GPUI-KIT scrollbar. Render only visible cards,
  with 50 px rows and 8 px gaps. Never recreate entities/subscriptions per render.
- The list emits `ActionsListEvent`; the root subscribes when opening the editor,
  updates `config.actions` and synchronizes the projection outside rendering.
  Sync disabled when sending commands/receiving updates. The widget does not edit domain state.
- `action_drag.rs`: controlled handle and temporary preview, no per-row entities.
  Validate ID/revision to reject foreign/stale drags. Escape/outside drop cancel.
  Autoscroll affects only the internal list, with one GPUI task per gesture.
- `Move { from, to }` uses final indexes and insertion, preserving intervening
  steps; never `swap` for arbitrary moves.
- `action_motion.rs`: temporary visual order and a 150 ms transition, retargeted
  from the current animated position. Do not edit the draft before drop. Hide the
  source card; neighbors open the gap. Fixed rows, virtualization and at most one
  extra card per edge, no per-row entities/tasks. Clear on drop, cancellation or invalidation.
- The decorative destination has a dashed border, subtle theme fill and
  `ACTION_ROW_HEIGHT`, behind cards; do not change layout/indexes or capture drop events.

An entity boundary alone does not prevent ancestor renders. Justify improvements
with profiling and distinguish CPU from FPS; do not change scrollbar limits or
add smoothing without measurements. A previous 100-action debug profile improved
from approximately 77 to 11 ms drawing CPU time, not monitor FPS. Reproduce with
`cargo run --example scroll_profile`; [method and limits](../tests/perf/README.md).

`cargo run --example grid_layout_ui` is an isolated experiment with fake data,
25/100/500 steps and one column, without resizing, database/hooks or execution.
`--check` tests events/CPU and exits automatically. It does not replace `ActionsList`:
the grid is not virtualized, commits outside drops and lacks required autoscroll.
Do not adopt it without checking long lists, cost, insertion, cancellation and
gesture protections. Simplified prototype cards do not provide identical-widget
comparisons or real presentation FPS measurements.
