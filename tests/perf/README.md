# Native scroll profiling

Run explicitly on Windows; these checks are not part of `cargo test`:

```powershell
cargo run --example scroll_profile
cargo run --example scroll_profile -- --preview  # Keep a preview open for 8 s afterward
cargo run --example scroll_profile -- --narrow --preview # 900 px window
```

The fixture opens an editor window with 25, 100 and 500 fake actions. It creates
no backend, opens no databases, installs no hooks and injects no OS input: synthetic
events are dispatched only to its own GPUI window. It exits automatically.
Do not close other Mxchro processes to run it.

## What is measured

- Debug mode, same window/controls, 5 warm-up updates and 40 samples per size,
  alternating ±12 px scrolling.
- CPU time for `Window::draw(...).clear(...)`, including element construction,
  layout and drawing preparation. **Excludes GPU presentation and monitor FPS.**
- Render count of the owning entity, to avoid equating a separate entity with
  the parent no longer rendering.
- No changes to scrollbar limits (120 FPS by default in the crate), animations,
  scroll physics, dependencies or Windows configuration.

## Local comparison (2026-11-13)

Sorted middle sample, in CPU milliseconds. Approximate values from this machine,
not guarantees for other computers or equivalents to real FPS:

| Actions | Before: all rows | Entity with virtualized rows |
| --- | ---: | ---: |
| 25 | 21.3 ms | 10.7 ms |
| 100 | 76.5 ms | 10.9 ms |
| 500 | 387.8 ms | 10.5 ms |

The owner rendered 45 times in both cases. GPUI also invalidates ancestors, and
this editor's views are not cached. The verified improvement bounds card work to
visible rows; it does **not eliminate all editor renders**. Real mouse-wheel/drag
feel on the user's monitor/configuration and release profiling remain pending.

The previous implementation is not duplicated in production. Baseline values
were measured with the same loop before the implementation change.

## Native functional checks

After profiling, the fixture checks through GPUI events:

- Removing virtual row 200 using the correct global index.
- Emitting/applying Move and Remove to the owner's sequence.
- Disabled controls and first/last movement boundaries.
- Scroll clamping after shortening/clearing the list.
- Scrollbar dragging with the simulated pointer inside the window.
- Action drag-and-drop: insertion instead of swapping, first/last row, same target,
  outside drop, Escape, disabled, stale revision if the sequence changes during
  dragging, and rejection from another editor. Stationary-pointer autoscroll at
  both edges, retaining the global source even when row 200 becomes invisible;
  stopping scroll when the gesture ends.
- Animated gap opening with quick retargeting, preserving draft order and emitting
  no edit events before drop.

Moves use `MacroDraft::move_action`, the same helper as production ownership.
Unit tests cover arbitrary moves and independent press/release steps. There is
no autosave: changed order belongs to the draft until `Save macro`.

After adding the drag handle, local debug profiling remained bounded by visible
rows: approximately 11.5–11.9 ms drawing CPU time for 25/100/500 actions.
This is not a measurement of FPS or perceived monitor smoothness.

These checks use the current editor geometry (50 px cards with 8 px gaps).
Review fixture click coordinates when the layout changes.

## Gap animation (2026-11-17, GPUI-KIT 0.10.0)

`action_motion.rs` unit tests use explicit times: movement in both directions,
interpolation, continuity when retargeting, exact final frame, repeated targets
without restarting and global virtual indexes. The native fixture lets the
transition finish and quickly changes destinations before dropping, then runs
`motion_profile` with 25, 100 and 500 actions, 20 measured drawings per size and
alternating destinations while the gesture remains active.

Local debug sample, drawing CPU time (not FPS or GPU presentation):

| Actions | p50 | p95 |
| --- | ---: | ---: |
| 25 | 18.5 ms | 19.9 ms |
| 100 | 15.8 ms | 19.3 ms |
| 500 | 15.3 ms | 17.3 ms |

Rendering remains limited to visible rows and one extra card per edge.
These values do not guarantee visual smoothness or replace checks on the user's
monitor; release profiling remains pending.
