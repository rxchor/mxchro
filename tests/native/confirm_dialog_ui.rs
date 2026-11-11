//! Native destructive-confirmation checks. Only fake channels and GPUI events:
//! no backend, database, hooks or injected OS input.
use gpui_kit::{component::WindowExt, *};
use mxchro::{
    application::{
        commands::Command,
        dto::{CollectionDTO, MacroDTO, MacroEditDTO, SaveMacroDTO},
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

fn snapshot() -> Snapshot {
    Snapshot {
        hotkey_issues: vec![],
        macros: vec![MacroDTO {
            trigger_key: "F6".into(),
            mode: Mode::Once,
            actions_count: 2,
            block_original_event: false,
            ignore_injected_keys: true,
            enabled: false,
            collection_id: None,
            toggle_enabled_hotkey: None,
        }],
        collections: vec![CollectionDTO {
            id: "work".into(),
            name: "Work".into(),
            enabled: false,
            macros_count: 0,
            order: 0,
            toggle_enabled_hotkey: None,
        }],
        keys: vec!["F6".into(), "A".into()],
    }
}
fn update(completed: bool) -> Update {
    Update {
        snapshot: snapshot(),
        request_completed: completed,
        editor: None,
        error: None,
        saved: false,
    }
}
fn editor() -> MacroEditDTO {
    MacroEditDTO {
        trigger_key: "F6".into(),
        mode: Mode::Once,
        actions: vec![
            Action::Wait { duration_ms: 100 },
            Action::Key {
                key: "A".into(),
                press: true,
            },
        ],
        block_original_event: false,
        ignore_injected_keys: true,
        enabled: false,
        collection_id: None,
        toggle_enabled_hotkey: None,
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
        .timer(Duration::from_millis(600))
        .await;
}
fn clear_notifications(handle: AnyWindowHandle, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| window.clear_notifications(cx))
        .unwrap();
}
fn save(
    handle: AnyWindowHandle,
    requests: &mpsc::Receiver<Request>,
    cx: &mut AsyncApp,
) -> SaveMacroDTO {
    scroll(handle, true, cx);
    click(handle, 1015., 740., cx);
    match requests.try_recv().expect("save click missed") {
        Request::Command(Command::SaveMacro { config, .. }) => config,
        _ => panic!("unexpected request instead of save"),
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
        updates.send(update(false)).unwrap();
        cx.spawn(async move |cx| {
            settle(cx).await;
            // A single Delete opens the dialog, never sends a destructive request.
            click(handle, 1063., 230., cx);
            settle(cx).await;
            assert!(dialog::active(handle, cx));
            assert!(requests.try_recv().is_err());
            dialog::key(handle, "escape", cx);
            settle(cx).await;
            assert!(!dialog::active(handle, cx));
            assert!(requests.try_recv().is_err());

            click(handle, 1063., 230., cx);
            settle(cx).await;
            dialog::key(handle, "enter", cx); // Safe default is cancellation.
            settle(cx).await;
            assert!(!dialog::active(handle, cx));
            assert!(requests.try_recv().is_err(), "Enter confirmed without focusing Delete");

            click(handle, 1063., 230., cx);
            settle(cx).await;
            click(handle, 20., 400., cx); // Overlay dismissal must also cancel.
            settle(cx).await;
            assert!(!dialog::active(handle, cx));
            assert!(requests.try_recv().is_err());

            click(handle, 1063., 230., cx);
            settle(cx).await;
            dialog::key(handle, "tab", cx);
            updates.send(update(false)).unwrap(); // Rebuild must not steal focus.
            settle(cx).await;
            dialog::key(handle, "enter", cx);
            settle(cx).await;
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::DeleteMacro(key))) if key == "F6"));
            dialog::key(handle, "enter", cx);
            click(handle, 1063., 230., cx);
            assert!(requests.try_recv().is_err(), "confirmation sent duplicate deletions");
            updates.send(update(false)).unwrap();
            settle(cx).await;
            click(handle, 1063., 230., cx);
            assert!(!dialog::active(handle, cx), "pending delete controls were unlocked by a refresh");
            updates.send(update(true)).unwrap();
            settle(cx).await;
            clear_notifications(handle, cx);
            settle(cx).await;

            click(handle, 1063., 230., cx);
            settle(cx).await;
            let mut removed = update(false);
            removed.snapshot.macros.clear();
            updates.send(removed).unwrap();
            settle(cx).await;
            assert!(!dialog::active(handle, cx), "obsolete macro confirmation stayed open");
            assert!(requests.try_recv().is_err());
            updates.send(update(false)).unwrap();
            settle(cx).await;

            click(handle, 80., 210., cx); // Work collection.
            click(handle, 666., 118., cx);
            settle(cx).await;
            assert!(dialog::active(handle, cx));
            assert!(requests.try_recv().is_err());
            dialog::accept(handle, cx).await;
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::DeleteCollection(id))) if id == "work"));
            assert!(requests.try_recv().is_err());
            updates.send(update(true)).unwrap();
            settle(cx).await;
            clear_notifications(handle, cx);
            settle(cx).await;

            click(handle, 80., 72., cx);
            click(handle, 947., 230., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Query(Query::EditMacro(_)))));
            let mut editing = update(true);
            editing.editor = Some(editor());
            updates.send(editing).unwrap();
            settle(cx).await;
            click(handle, 732., 542., cx);
            settle(cx).await;
            assert!(!dialog::active(handle, cx), "action removal must be immediate, without confirmation");
            assert!(requests.try_recv().is_err(), "removing a draft action wrote to backend");
            let config = save(handle, &requests, cx);
            assert_eq!(config.actions.len(), 1);
            assert!(matches!(&config.actions[0], Action::Key { key, press: true } if key == "A"));
            // Sending a save still locks action editing until its acknowledgement.
            scroll(handle, false, cx);
            click(handle, 732., 542., cx);
            settle(cx).await;
            assert!(!dialog::active(handle, cx));
            assert!(requests.try_recv().is_err());
            updates.send(update(true)).unwrap();
            settle(cx).await;
            let unchanged = save(handle, &requests, cx);
            assert_eq!(unchanged.actions.len(), 1, "pending removal changed the draft");
            updates.send(update(true)).unwrap();
            settle(cx).await;
            scroll(handle, false, cx);
            click(handle, 732., 542., cx);
            settle(cx).await;
            assert!(!dialog::active(handle, cx));
            let config = save(handle, &requests, cx);
            assert!(config.actions.is_empty());
            println!("Native confirmations passed: macro/collection modals, safe Enter/Cancel/Escape/overlay, stable focus, single deletes, pending/stale guards and immediate draft-only action removal without a dialog. No backend, hooks or real data.");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
