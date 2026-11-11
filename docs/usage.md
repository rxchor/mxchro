# Using Mxchro

[Index](README.md) · [Appearance](appearance.md) · [Data](data.md)

## Home and collections

The sidebar filters all macros, macros without a collection or one specific
collection. In `All macros`, cards are grouped into named boxes with counts,
in collection order, omitting empty groups and placing `No collection` last.
Each macro shows its mode (`Once`, `Hold`, `Toggle`) in a Tag and an action count,
without repeating the collection name. `Enabled`/`Disabled` uses a theme-colored tag.
An assigned enable/disable key appears as `Enable/disable: F8` in the macro's
second row and beside its collection name (or count in the collection filter).
This is not the key that executes the sequence. Preserved incompatible assignments
show `(inactive)`; macros without hotkeys do not add a tag.

`Create collection` opens a Popover form beside the sidebar button. To rename,
select a collection and click `Rename collection`: it opens with the current name,
without a permanent shared sidebar field. The input receives focus; Enter submits,
and Cancel/Escape/outside click discard without saving. Submission locks the form.
Only an accepted response closes it on success; unsolicited refreshes do not.
Rejection preserves text and shows name errors beside the field. Navigation or
losing the collection invalidates the rename target.

Collections enable/disable their macros together. The selected collection's hotkey
supports search, clearing and autosave. Deleting a collection preserves its macros
as `No collection`.

## Creating and editing macros

The sidebar stays visible without selected filters while editing. Clicking
`All macros`, `No collection` or a collection exits the form and opens that list,
discarding the draft like Cancel, without saving or another confirmation.
Saving/confirmation blocks navigation. Settings retains the draft when returning;
navigating during capture cancels it.

Trigger, action key and optional hotkey use searchable selectors, including mouse
buttons. Labels preserve exact mapper names (`A`, `F6`, `left_ctrl`, `mouse_left`,
etc.). All selectors allow clearing. Trigger and mode are required; select an
action key before adding it.

- Once executes a sequence; Hold repeats while the trigger is held; Toggle
  toggles execution on each press.
- Triggers and hotkeys support **one key**, not chords such as Ctrl+F6.
- To produce Ctrl+C: press Ctrl, press C, release C, release Ctrl, with explicit
  waits where needed. Press and release remain independent steps.
- Options allow blocking original input and ignoring injected input. The engine
  also retains self-interference protections.
- The mode selector has no search. Collections are searchable by name; clearing
  leaves the macro without a collection. Renaming/deleting a collection updates
  the editor catalog without recreating selectors.
- `Key action` adds a press, release or both. `Wait` adds a positive millisecond
  delay. Both cards add steps to the same sequence.
- The editor pairs fields and uses panels that stack at narrower widths.
  The sequence scrolls internally and retains its height when empty.
- Actions have icons and PRESS/RELEASE/WAIT labels, with key/duration separate.
  There is no numbering: execution order runs from top to bottom.

## Ordering and deletion

Drag the dotted handle and drop onto the dashed destination box. Moving the
pointer near either edge scrolls the list; Escape or dropping outside cancels.
The ↑/↓ buttons remain available. Only the chosen step moves, without arbitrary
swapping or grouping press/release actions. The draft changes on drop and persists
with `Save macro`. Cards open the destination gap with a 150 ms transition.

The X (`Remove`) button removes a step immediately, without confirmation or disk
writes until Save. Deleting macros/collections opens a dialog: Cancel has initial
focus, Enter cancels by default, and Tab can focus the destructive button.
Escape, X and outside click cancel. No double-click or Delete/Confirm label change.
The dialog is invalidated if its target disappears or the editor opens.

## Capturing a key or button

Click the recording icon beside a selector, then press and release the key/button.
The dropdown remains available and selects the exact mapper name. Editor capture
only changes the draft; collection hotkey capture performs one autosave.

All five mouse buttons are supported: left, right, middle, X1 and X2. Wheel and
movement do not complete capture. Escape cancels (use the dropdown to assign `esc`);
focus loss, navigation/editor changes, closing and a 10-second timeout also cancel.
Listening uses an InApp notification removed at completion. Failures are persistent,
deduplicated notifications, not inline field errors.

An unknown VK shows `Unsupported key` with its code and preserves selection.
Injected input and repeats/releases of keys held before capture started are not
captured. The captured gesture does not activate macros or click the UI. Sequences
already running are not stopped. Physical hooks, keyboard layouts and privilege
levels still require validation.

## Incompatible hotkeys

A macro in a collection cannot have an individual hotkey. The editor disables
that selector and offers `Clear individual hotkey` for legacy values. Removing
the collection allows individual hotkeys again. Each hotkey must be unique across
collections and independent macros; duplicates are rejected without replacing owners.

Incompatible legacy data is preserved: grouped/unknown hotkeys and all eligible
duplicate assignments remain inactive. A warning identifies owner, key and reason.
Editing and clearing/changing the hotkey recalculates the index. An inactive grouped
hotkey does not prevent a collection from using that key. Manual enable/disable
buttons remain available even when a hotkey is inactive.

## Safety and limitations

New macros are saved **disabled**. Review actions before enabling them. Avoid
repetition without waits: Hold/Toggle can generate input very quickly. Hooks are
global while editing; disable relevant macros before typing their triggers in the app.

Long waits do not stop immediately, and inherited persistence/held-key release
limitations remain. See [validation](validation.md), [data](data.md) and
[TODO](../TODO.md); do not use valuable data without backups.
