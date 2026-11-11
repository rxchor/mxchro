//! Opt-in native CPU benchmark: synthetic GPUI events, no OS input,
//! backend, hooks or database. Measures draw work, not monitor FPS/GPU presentation.
use gpui_kit::*;
use mxchro::{
    domain::types::Action,
    view::ui::{
        pages::macro_editor::{MacroEditorPage, actions_list::ActionsListEvent},
        state::MacroDraft,
    },
};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

struct Probe {
    draft: MacroDraft,
    renders: Rc<Cell<usize>>,
    events: Vec<ActionsListEvent>,
}

fn click(handle: AnyWindowHandle, position: Point<Pixels>, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position,
                pressed_button: None,
                modifiers: Modifiers::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseDownEvent {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
}

fn replace_actions(view: &Entity<Probe>, count: usize, cx: &mut AsyncApp) {
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.draft.config.actions = (0..count)
                .map(|_| Action::Wait { duration_ms: 100 })
                .collect();
            view.draft.actions_list.update(cx, |list, cx| {
                list.replace_actions(view.draft.config.actions.clone(), cx)
            });
            cx.notify();
        })
    });
}

fn interaction_checks(handle: AnyWindowHandle, view: &Entity<Probe>, cx: &mut AsyncApp) {
    replace_actions(view, 500, cx);
    let scroll = cx.update(|cx| view.read(cx).draft.actions_scroll.clone());
    scroll.set_offset(point(px(0.), px(-58. * 200.)));
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    let bounds = scroll.bounds();
    click(
        handle,
        point(bounds.right() - px(36.), bounds.top() + px(25.)),
        cx,
    );
    cx.update(|cx| {
        let probe = view.read(cx);
        assert!(
            matches!(probe.events.last(), Some(ActionsListEvent::Remove(200))),
            "wrong virtual row index"
        );
        assert_eq!(probe.draft.config.actions.len(), 499);
    });
    let before = cx.update(|cx| view.read(cx).events.len());
    let list = cx.update(|cx| view.read(cx).draft.actions_list.clone());
    cx.update(|cx| list.update(cx, |list, cx| list.set_disabled(true, cx)));
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    click(
        handle,
        point(bounds.right() - px(36.), bounds.top() + px(25.)),
        cx,
    );
    assert_eq!(cx.update(|cx| view.read(cx).events.len()), before);
    cx.update(|cx| list.update(cx, |list, cx| list.set_disabled(false, cx)));

    replace_actions(view, 3, cx);
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    assert_eq!(scroll.offset().y, px(0.));
    assert_eq!(scroll.max_offset().y, px(0.));
    let bounds = scroll.bounds();
    // Boundary buttons are disabled; the first row's down button still emits Move.
    click(
        handle,
        point(bounds.right() - px(100.), bounds.top() + px(25.)),
        cx,
    );
    click(
        handle,
        point(bounds.right() - px(68.), bounds.top() + px(58. * 2. + 25.)),
        cx,
    );
    assert_eq!(cx.update(|cx| view.read(cx).events.len()), before);
    click(
        handle,
        point(bounds.right() - px(68.), bounds.top() + px(25.)),
        cx,
    );
    assert!(cx.update(|cx| matches!(
        view.read(cx).events.last(),
        Some(ActionsListEvent::Move { from: 0, to: 1 })
    )));

    replace_actions(view, 0, cx);
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    assert_eq!(scroll.offset().y, px(0.));
    assert_eq!(scroll.max_offset().y, px(0.));

    replace_actions(view, 100, cx);
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let bounds = scroll.bounds();
    let start = point(bounds.right() - px(8.), bounds.top() + px(20.));
    let end = start + point(px(0.), px(80.));
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position: start,
                pressed_button: None,
                modifiers: Modifiers::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseDownEvent {
                position: start,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseMoveEvent {
                position: end,
                pressed_button: Some(MouseButton::Left),
                modifiers: Modifiers::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                position: end,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    assert!(
        scroll.offset().y < px(0.),
        "scrollbar drag did not move the content"
    );
    println!(
        "Native interaction checks passed: virtual indices, remove/move, disabled, boundaries, shrink/empty, scrollbar drag."
    );
}

fn move_pointer(handle: AnyWindowHandle, position: Point<Pixels>, held: bool, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position,
                pressed_button: held.then_some(MouseButton::Left),
                modifiers: Modifiers::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.draw(cx).clear(cx);
    })
    .unwrap();
}

fn begin_drag(
    handle: AnyWindowHandle,
    start: Point<Pixels>,
    target: Point<Pixels>,
    cx: &mut AsyncApp,
) {
    move_pointer(handle, start, false, cx);
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseDownEvent {
                position: start,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    move_pointer(handle, start + point(px(10.), px(0.)), true, cx);
    move_pointer(handle, target, true, cx);
}

fn end_drag(handle: AnyWindowHandle, target: Point<Pixels>, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseUpEvent {
                position: target,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
        window.draw(cx).clear(cx);
    })
    .unwrap();
}

async fn drag_checks(handle: AnyWindowHandle, view: &Entity<Probe>, cx: &mut AsyncApp) {
    replace_actions(view, 5, cx);
    cx.update(|cx| {
        view.update(cx, |probe, cx| {
            probe.draft.config.actions = (1..=5)
                .map(|duration_ms| Action::Wait { duration_ms })
                .collect();
            probe.draft.actions_list.update(cx, |list, cx| {
                list.replace_actions(probe.draft.config.actions.clone(), cx)
            });
        })
    });
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let scroll = cx.update(|cx| view.read(cx).draft.actions_scroll.clone());
    let bounds = scroll.bounds();
    let source = point(bounds.left() + px(22.), bounds.top() + px(25.));
    let target = point(bounds.left() + px(150.), bounds.top() + px(58. * 2. + 40.));
    begin_drag(handle, source, target, cx);
    assert!(
        cx.update(|cx| cx.has_active_drag()),
        "handle did not start a native GPUI drag"
    );
    // Let the visual gap settle and rapidly reverse its destination. None of
    // these frames may reorder the owner's draft or emit an edit before drop.
    let before_motion = cx.update(|cx| view.read(cx).events.len());
    cx.background_executor()
        .timer(Duration::from_millis(180))
        .await;
    let first = point(target.x, bounds.top() + px(10.));
    move_pointer(handle, first, true, cx);
    cx.background_executor()
        .timer(Duration::from_millis(40))
        .await;
    move_pointer(handle, target, true, cx);
    cx.background_executor()
        .timer(Duration::from_millis(180))
        .await;
    cx.update(|cx| {
        let probe = view.read(cx);
        assert_eq!(probe.events.len(), before_motion);
        let durations: Vec<_> = probe
            .draft
            .config
            .actions
            .iter()
            .map(|action| match action {
                Action::Wait { duration_ms } => *duration_ms,
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(
            durations,
            [1, 2, 3, 4, 5],
            "visual motion edited the draft before drop"
        );
    });
    end_drag(handle, target, cx);
    assert!(cx.update(|cx| matches!(
        view.read(cx).events.last(),
        Some(ActionsListEvent::Move { from: 0, to: 2 })
    )));
    let values = |cx: &mut AsyncApp| {
        cx.update(|cx| {
            view.read(cx)
                .draft
                .config
                .actions
                .iter()
                .map(|a| match a {
                    Action::Wait { duration_ms } => *duration_ms,
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>()
        })
    };
    assert_eq!(values(cx), [2, 3, 1, 4, 5]);
    let source_last = point(source.x, bounds.top() + px(58. * 4. + 25.));
    let target_first = point(target.x, bounds.top() + px(10.));
    begin_drag(handle, source_last, target_first, cx);
    end_drag(handle, target_first, cx);
    assert_eq!(values(cx), [5, 2, 3, 1, 4]);

    let before = cx.update(|cx| view.read(cx).events.len());
    begin_drag(handle, source, target_first, cx); // same insertion boundary
    end_drag(handle, target_first, cx);
    let outside = point(bounds.right() + px(50.), bounds.top() + px(100.));
    begin_drag(handle, source, outside, cx);
    end_drag(handle, outside, cx);
    begin_drag(handle, source, target, cx);
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            KeyDownEvent {
                keystroke: Keystroke::parse("escape").unwrap(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    assert!(
        !cx.update(|cx| cx.has_active_drag()),
        "Escape did not cancel the drag"
    );
    end_drag(handle, target, cx);
    assert_eq!(cx.update(|cx| view.read(cx).events.len()), before);

    let list = cx.update(|cx| view.read(cx).draft.actions_list.clone());
    cx.update(|cx| list.update(cx, |list, cx| list.set_disabled(true, cx)));
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    begin_drag(handle, source, target, cx);
    assert!(
        !cx.update(|cx| cx.has_active_drag()),
        "disabled handle started dragging"
    );
    end_drag(handle, target, cx);
    cx.update(|cx| list.update(cx, |list, cx| list.set_disabled(false, cx)));
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    begin_drag(handle, source, target, cx);
    replace_actions(view, 5, cx); // invalidate the source revision while dragging
    end_drag(handle, target, cx);
    assert_eq!(cx.update(|cx| view.read(cx).events.len()), before);
    begin_drag(handle, source, target, cx);
    cx.update(|cx| list.update(cx, |list, cx| list.set_disabled(true, cx)));
    end_drag(handle, target, cx);
    assert_eq!(cx.update(|cx| view.read(cx).events.len()), before);
    cx.update(|cx| list.update(cx, |list, cx| list.set_disabled(false, cx)));
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();

    // Another editor must not accept this editor's payload.
    let (other_handle, other) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(40.), px(40.)),
                    size: size(px(1160.), px(1000.)),
                })),
                ..Default::default()
            },
            cx,
            |window, cx| {
                cx.new(|cx| {
                    let draft =
                        MacroDraft::new(None, None, &["A".into(), "F6".into()], &[], window, cx);
                    cx.subscribe(&draft.actions_list, |probe: &mut Probe, _, event, _| {
                        probe.events.push(event.clone());
                    })
                    .detach();
                    Probe {
                        draft,
                        renders: Rc::new(Cell::new(0)),
                        events: Vec::new(),
                    }
                })
            },
        )
        .unwrap()
    });
    replace_actions(&other, 5, cx);
    cx.update_window(other_handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let other_bounds = cx.update(|cx| other.read(cx).draft.actions_scroll.bounds());
    begin_drag(handle, source, target, cx);
    let other_target = other_bounds.center();
    move_pointer(other_handle, other_target, true, cx);
    end_drag(other_handle, other_target, cx);
    assert!(
        cx.update(|cx| other.read(cx).events.is_empty()),
        "foreign list accepted the drag"
    );
    assert_eq!(cx.update(|cx| view.read(cx).events.len()), before);
    cx.update_window(other_handle, |_, window, _| window.remove_window())
        .unwrap();
    // Let native activation/mouse notifications from closing the second window
    // settle before dispatching another synthetic stationary-pointer gesture.
    cx.background_executor()
        .timer(Duration::from_millis(150))
        .await;

    replace_actions(view, 500, cx);
    scroll.set_offset(point(px(0.), px(-58. * 200.)));
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let bounds = scroll.bounds();
    let source = point(bounds.left() + px(22.), bounds.top() + px(25.));
    let bottom = point(bounds.left() + px(150.), bounds.bottom() - px(2.));
    let offset = scroll.offset().y;
    begin_drag(handle, source, bottom, cx);
    assert!(
        cx.update(|cx| cx.has_active_drag()),
        "virtual row handle did not start dragging"
    );
    cx.background_executor()
        .timer(Duration::from_millis(300))
        .await;
    assert!(
        scroll.offset().y < offset - px(58.),
        "stationary pointer did not auto-scroll past a virtual row: {:?} -> {:?}",
        offset,
        scroll.offset().y
    );
    end_drag(handle, bottom, cx);
    assert!(cx.update(|cx| matches!(view.read(cx).events.last(), Some(ActionsListEvent::Move { from: 200, to }) if *to > 204)), "drop lost the global source index after virtualization");
    cx.background_executor()
        .timer(Duration::from_millis(40))
        .await;
    let stopped = scroll.offset().y;
    cx.background_executor()
        .timer(Duration::from_millis(80))
        .await;
    assert_eq!(
        scroll.offset().y,
        stopped,
        "auto-scroll continued after dropping"
    );
    let first_full_row = (-f32::from(scroll.offset().y) / 58.).ceil();
    let source = point(
        bounds.left() + px(22.),
        bounds.top() + px(first_full_row * 58. + f32::from(scroll.offset().y) + 25.),
    );
    let top = point(bounds.left() + px(150.), bounds.top() + px(2.));
    begin_drag(handle, source, top, cx);
    assert!(
        cx.update(|cx| cx.has_active_drag()),
        "top-edge source did not start dragging"
    );
    cx.background_executor()
        .timer(Duration::from_millis(180))
        .await;
    assert!(
        scroll.offset().y > stopped,
        "top edge did not scroll upward"
    );
    end_drag(handle, outside, cx);
    println!(
        "Native drag checks passed: animated gap/reversal without draft edits, insert not swap, first/last, no-op, outside, Escape, disabled, stale/foreign payloads, virtual indices and both auto-scroll edges."
    );
}

async fn motion_profile(handle: AnyWindowHandle, view: &Entity<Probe>, cx: &mut AsyncApp) {
    for count in [25, 100, 500] {
        replace_actions(view, count, cx);
        let scroll = cx.update(|cx| view.read(cx).draft.actions_scroll.clone());
        scroll.set_offset(point(px(0.), px(0.)));
        cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
            .unwrap();
        let bounds = scroll.bounds();
        let source = point(bounds.left() + px(22.), bounds.top() + px(25.));
        let targets = [
            point(bounds.left() + px(150.), bounds.top() + px(58. + 40.)),
            point(bounds.left() + px(150.), bounds.top() + px(58. * 3. + 40.)),
        ];
        begin_drag(handle, source, targets[0], cx);
        assert!(cx.update(|cx| cx.has_active_drag()));
        let mut samples = Vec::new();
        for index in 0..20 {
            move_pointer(handle, targets[index % 2], true, cx);
            cx.background_executor()
                .timer(Duration::from_millis(16))
                .await;
            samples.push(
                cx.update_window(handle, |_, window, cx| {
                    let started = Instant::now();
                    window.draw(cx).clear(cx);
                    started.elapsed().as_secs_f64() * 1000.
                })
                .unwrap(),
            );
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "animated_rows={count} samples={} draw_cpu_p50_ms={:.3} draw_cpu_p95_ms={:.3}",
            samples.len(),
            samples[samples.len() / 2],
            samples[18]
        );
        let outside = point(bounds.right() + px(50.), bounds.top() + px(100.));
        end_drag(handle, outside, cx);
        cx.background_executor()
            .timer(Duration::from_millis(40))
            .await;
    }
}
impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        div()
            .size_full()
            .flex()
            .child(
                div()
                    .w(px(250.))
                    .flex_shrink_0()
                    .p_4()
                    .child("Scroll profiling fixture"),
            )
            .child(
                div()
                    .id("profile-main")
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_y_scroll()
                    .p_6()
                    .child(MacroEditorPage {
                        capture_enabled: false,
                        interaction_locked: false,
                        draft: self.draft.clone(),
                        pending: false,
                        on_action: Rc::new(|_, _, _| {}),
                    }),
            )
    }
}
fn main() {
    let preview = std::env::args().any(|arg| arg == "--preview");
    let width = if std::env::args().any(|arg| arg == "--narrow") {
        900.
    } else {
        1160.
    };
    mxchro::view::i18n::init();
    gpui_kit::application().with_assets(mxchro::view::assets::AppAssets).run(move |cx| {
        gpui_kit::init(cx);
        mxchro::view::theme::init(cx);
        let renders = Rc::new(Cell::new(0));
        let (handle, view) = gpui_kit::open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(10.), px(10.)), size: size(px(width), px(1000.)) })),
            ..Default::default()
        }, cx, |window, cx| cx.new(|cx| {
            let draft = MacroDraft::new(None, None, &["A".into(), "F6".into()], &[], window, cx);
            cx.subscribe(&draft.actions_list, |probe: &mut Probe, _, event, cx| {
                probe.events.push(event.clone());
                match event {
                    ActionsListEvent::Move { from, to } => { probe.draft.move_action(*from, *to); },
                    ActionsListEvent::Remove(index) => { probe.draft.config.actions.remove(*index); },
                }
                probe.draft.actions_list.update(cx, |list, cx| list.replace_actions(probe.draft.config.actions.clone(), cx));
                cx.notify();
            }).detach();
            Probe { draft, renders: renders.clone(), events: Vec::new() }
        })).unwrap();
        cx.spawn(async move |cx| {
            cx.background_executor().timer(Duration::from_millis(100)).await;
            for count in [25, 100, 500] {
                replace_actions(&view, count, cx);
                cx.update(|cx| view.update(cx, |view, cx| {
                    view.draft.actions_scroll.set_offset(point(px(0.), px(0.)));
                    cx.notify();
                }));
                cx.update_window(handle, |_, window, cx| { window.refresh(); window.draw(cx).clear(cx); }).unwrap();
                let scroll = cx.update(|cx| view.read(cx).draft.actions_scroll.clone());
                let mut samples = Vec::new();
                let before = renders.get();
                for index in 0..45 {
                    cx.update_window(handle, |_, window, cx| {
                        window.dispatch_event(ScrollEvent {
                            position: scroll.bounds().center(), modifiers: Modifiers::default(),
                            delta: ScrollDelta::Pixels(point(px(0.), px(if index % 2 == 0 { -12. } else { 12. }))),
                            touch_phase: TouchPhase::Moved,
                            source: ScrollSource::Pointer,
                        }.to_platform_input(), cx);
                    }).unwrap();
                    let elapsed = cx.update_window(handle, |_, window, cx| {
                        let started = Instant::now();
                        window.draw(cx).clear(cx);
                        started.elapsed().as_secs_f64() * 1000.
                    }).unwrap();
                    if index >= 5 { samples.push(elapsed); }
                }
                samples.sort_by(f64::total_cmp);
                let p95_index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
                println!("rows={count} samples={} draw_cpu_p50_ms={:.3} draw_cpu_p95_ms={:.3} owner_renders={} final_y={:.1}",
                    samples.len(), samples[samples.len() / 2], samples[p95_index], renders.get() - before, f32::from(scroll.offset().y));
            }
            interaction_checks(handle, &view, cx);
            drag_checks(handle, &view, cx).await;
            motion_profile(handle, &view, cx).await;
            if preview {
                cx.update(|cx| view.update(cx, |probe, cx| {
                    probe.draft.config.actions = vec![
                        Action::Key { key: "left_ctrl".into(), press: true },
                        Action::Key { key: "left_ctrl".into(), press: false },
                        Action::Wait { duration_ms: 100 },
                    ];
                    probe.draft.actions_list.update(cx, |list, cx| list.replace_actions(probe.draft.config.actions.clone(), cx));
                    cx.notify();
                }));
                cx.background_executor().timer(Duration::from_secs(8)).await;
            }
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
