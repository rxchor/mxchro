//! Real sidebar/editor navigation with fake channels. No hooks, disk or OS input.
use gpui_kit::{component::WindowExt, *};
use mxchro::{
    application::{
        commands::Command,
        dto::{CollectionDTO, MacroDTO, MacroEditDTO, SaveMacroDTO},
        error::CommandError,
        queries::{Query, Snapshot},
        request::Request,
    },
    domain::types::{Action, Mode},
    view::{
        assets::AppAssets,
        backend::{CommandBus, Update},
        i18n, theme,
        ui::MxchroApp,
    },
};
use std::{sync::mpsc, time::Duration};
#[path = "support/dialog.rs"]
mod dialog;
#[path = "support/settings_bus.rs"]
mod settings_bus;
#[path = "support/window_capture.rs"]
mod window_capture;

fn editing() -> MacroEditDTO {
    MacroEditDTO {
        trigger_key: "F6".into(),
        mode: Mode::Once,
        actions: vec![Action::Wait { duration_ms: 123 }],
        enabled: false,
        block_original_event: false,
        ignore_injected_keys: true,
        collection_id: None,
        toggle_enabled_hotkey: None,
    }
}

fn update(completed: bool, editor: Option<MacroEditDTO>, error: Option<CommandError>) -> Update {
    Update {
        request_completed: completed,
        editor,
        error,
        saved: false,
        snapshot: Snapshot {
            keys: vec!["A".into(), "F6".into()],
            hotkey_issues: vec![],
            macros: vec![MacroDTO {
                trigger_key: "F6".into(),
                mode: Mode::Once,
                actions_count: 1,
                enabled: false,
                block_original_event: false,
                ignore_injected_keys: true,
                collection_id: None,
                toggle_enabled_hotkey: None,
            }],
            collections: vec![CollectionDTO {
                id: "work".into(),
                name: "Work".into(),
                order: 0,
                enabled: false,
                macros_count: 0,
                toggle_enabled_hotkey: None,
            }],
        },
    }
}

fn click(handle: AnyWindowHandle, x: f32, y: f32, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        let position = point(px(x), px(y));
        window.dispatch_event(
            MouseMoveEvent {
                position,
                pressed_button: None,
                modifiers: Modifiers::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.draw(cx).clear(cx);
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
        window.draw(cx).clear(cx);
    })
    .unwrap();
}

fn scroll(handle: AnyWindowHandle, bottom: bool, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            ScrollEvent {
                position: point(px(800.), px(400.)),
                modifiers: Modifiers::default(),
                delta: ScrollDelta::Pixels(point(px(0.), px(if bottom { -2000. } else { 2000. }))),
                touch_phase: TouchPhase::Moved,
                source: ScrollSource::Pointer,
            }
            .to_platform_input(),
            cx,
        );
        window.draw(cx).clear(cx);
    })
    .unwrap();
}

async fn settle(cx: &mut AsyncApp) {
    cx.background_executor()
        .timer(Duration::from_millis(250))
        .await;
}

async fn scene(handle: AnyWindowHandle, cx: &mut AsyncApp) -> window_capture::Capture {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position: point(px(2.), px(2.)),
                pressed_button: None,
                modifiers: Modifiers::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.draw(cx).clear(cx);
    })
    .unwrap();
    settle(cx).await;
    window_capture::pixels()
}

fn save(
    handle: AnyWindowHandle,
    requests: &mpsc::Receiver<Request>,
    cx: &mut AsyncApp,
) -> (Option<String>, SaveMacroDTO) {
    scroll(handle, true, cx);
    click(handle, 1015., 740., cx);
    match requests.try_recv() {
        Ok(Request::Command(Command::SaveMacro { old_key, config })) => (old_key, config),
        _ => panic!("editor Save was not available"),
    }
}

fn main() {
    i18n::init();
    gpui_kit::application().with_assets(AppAssets).run(|cx| {
        gpui_kit::init(cx);
        theme::init(cx);
        let (commands, requests) = mpsc::channel();
        let (updates, receiver) = mpsc::channel();
        let (handle, _) = gpui_kit::open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(20.), px(20.)), size: size(px(1160.), px(780.)) })),
            ..Default::default()
        }, cx, |window, cx| cx.new(|cx| MxchroApp::new(CommandBus { requests: commands, updates: receiver }, settings_bus::disconnected(), None, window, cx))).unwrap();
        updates.send(update(false, None, None)).unwrap();
        cx.spawn(async move |cx| {
            settle(cx).await;
            let home = scene(handle, cx).await;
            click(handle, 1080., 48., cx); // New macro, not an injected edit detail.
            settle(cx).await;
            let creating = scene(handle, cx).await;
            let excluded = &[(250, 0, 1160, 780), (0, 700, 250, 780)];
            assert!(!home.same_outside(&creating, excluded), "All remained selected while creating");
            click(handle, 208., 750., cx);
            let settings = scene(handle, cx).await;
            assert!(creating.same_outside(&settings, excluded), "a filter was selected in the editor");
            click(handle, 1100., 40., cx); // Back from Settings preserves draft.
            settle(cx).await;
            assert!(creating.same_outside(&scene(handle, cx).await, excluded));
            assert!(requests.try_recv().is_err(), "opening/navigating wrote to backend");

            for (x, y, expected_collection) in [(80., 72., None), (80., 116., None), (80., 210., Some("work"))] {
                // Also cover editing an existing macro, not only a blank draft.
                updates.send(update(false, Some(editing()), None)).unwrap();
                settle(cx).await;
                scroll(handle, false, cx);
                assert!(creating.same_outside(&scene(handle, cx).await, excluded), "editing retained a sidebar filter selection");
                click(handle, x, y, cx);
                settle(cx).await;
                assert!(requests.try_recv().is_err(), "leaving the draft submitted a command");
                scroll(handle, false, cx);
                click(handle, 1080., 48., cx); // Must now be Home, with New macro available.
                settle(cx).await;
                let (old_key, config) = save(handle, &requests, cx);
                assert!(old_key.is_none(), "sidebar did not discard the edited macro");
                assert!(config.trigger_key.is_empty() && config.actions.is_empty(), "old draft leaked into New macro");
                assert_eq!(config.collection_id.as_deref(), expected_collection, "sidebar opened the wrong filter");
                for (nav_x, nav_y) in [(80., 72.), (80., 116.), (80., 210.), (208., 750.)] {
                    click(handle, nav_x, nav_y, cx);
                }
                updates.send(update(false, None, None)).unwrap();
                settle(cx).await;
                click(handle, 1015., 740., cx);
                assert!(requests.try_recv().is_err(), "pending refresh allowed another save");
                updates.send(update(true, None, Some(CommandError::InvalidTriggerKey))).unwrap();
                settle(cx).await;
                let (old_key, config) = save(handle, &requests, cx);
                assert!(old_key.is_none());
                assert_eq!(config.collection_id.as_deref(), expected_collection, "pending navigation replaced the draft");
                updates.send(update(true, None, Some(CommandError::InvalidTriggerKey))).unwrap();
                settle(cx).await;
            }

            // A pending edit-detail query cannot navigate away before its reply.
            click(handle, 80., 72., cx);
            settle(cx).await;
            scroll(handle, false, cx);
            click(handle, 947., 230., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Query(Query::EditMacro(key))) if key == "F6"));
            click(handle, 80., 210., cx);
            updates.send(update(true, Some(editing()), None)).unwrap();
            settle(cx).await;
            let (old_key, config) = save(handle, &requests, cx);
            assert_eq!(old_key.as_deref(), Some("F6"));
            assert_eq!(config.trigger_key, "F6");
            updates.send(update(true, None, Some(CommandError::OperationFailed))).unwrap();
            settle(cx).await;
            cx.update_window(handle, |_, window, cx| window.clear_notifications(cx)).unwrap();
            click(handle, 80., 72., cx);
            settle(cx).await;
            scroll(handle, false, cx);
            click(handle, 1063., 230., cx);
            settle(cx).await;
            assert!(dialog::active(handle, cx));
            click(handle, 80., 210., cx); // Overlay cancels; never navigates underneath.
            settle(cx).await;
            assert!(!dialog::active(handle, cx));
            assert!(requests.try_recv().is_err());
            click(handle, 947., 230., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Query(Query::EditMacro(key))) if key == "F6"), "confirmation allowed sidebar navigation underneath");
            println!("Sidebar navigation passed: no selected filters in create/edit, Settings retains draft, all three filters discard without saving, correct new-macro collection, pending save/query/refresh guards and confirmation overlay. No hooks or real data.");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
