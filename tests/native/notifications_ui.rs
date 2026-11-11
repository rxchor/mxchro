//! Real application with controlled acknowledgements; no backend, hooks or disk.
use gpui_kit::{component::WindowExt, *};
use mxchro::{
    application::{
        commands::Command,
        dto::{MacroDTO, MacroEditDTO},
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

fn snapshot() -> Snapshot {
    Snapshot {
        hotkey_issues: Vec::new(),
        macros: vec![MacroDTO {
            trigger_key: "F6".into(),
            mode: Mode::Once,
            actions_count: 1,
            block_original_event: false,
            ignore_injected_keys: true,
            enabled: false,
            collection_id: None,
            toggle_enabled_hotkey: Some("B".into()),
        }],
        collections: vec![],
        keys: vec!["A".into(), "B".into(), "F6".into()],
    }
}

fn update(request_completed: bool, error: Option<CommandError>, empty: bool) -> Update {
    let mut snapshot = snapshot();
    if empty {
        snapshot.macros.clear();
    }
    Update {
        request_completed,
        snapshot,
        error,
        saved: false,
        editor: None,
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

fn count(handle: AnyWindowHandle, cx: &mut AsyncApp) -> usize {
    cx.update_window(handle, |_, window, cx| window.notifications(cx).len())
        .unwrap()
}

fn scroll_editor_to_bottom(handle: AnyWindowHandle, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            ScrollEvent {
                position: point(px(800.), px(400.)),
                modifiers: Modifiers::default(),
                delta: ScrollDelta::Pixels(point(px(0.), px(-2000.))),
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
        updates.send(update(false, None, false)).unwrap();
        cx.spawn(async move |cx| {
            settle(cx).await;
            assert_eq!(count(handle, cx), 0);
            click(handle, 1006., 230., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::SetMacroEnabled(..)))));
            updates.send(update(true, None, false)).unwrap();
            settle(cx).await;
            assert_eq!(count(handle, cx), 0, "a toggle produced a success toast");

            click(handle, 947., 230., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Query(Query::EditMacro(_)))));
            let mut editing = update(true, None, false);
            editing.editor = Some(MacroEditDTO {
                trigger_key: "F6".into(), mode: Mode::Once,
                actions: vec![Action::Wait { duration_ms: 1 }],
                block_original_event: false, ignore_injected_keys: true,
                enabled: false, collection_id: None, toggle_enabled_hotkey: None,
            });
            updates.send(editing).unwrap();
            settle(cx).await;
            scroll_editor_to_bottom(handle, cx);
            click(handle, 1015., 740., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::SaveMacro { .. }))), "save click missed");
            updates.send(update(true, Some(CommandError::InvalidTriggerKey), false)).unwrap();
            settle(cx).await;
            assert_eq!(count(handle, cx), 0, "trigger validation was sent to a toast");
            scroll_editor_to_bottom(handle, cx);
            click(handle, 1015., 740., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::SaveMacro { .. }))));
            updates.send(update(true, Some(CommandError::ToggleHotkeyInUse("A".into())), false)).unwrap();
            settle(cx).await;
            assert_eq!(count(handle, cx), 0, "duplicate hotkey validation was sent to a toast");
            scroll_editor_to_bottom(handle, cx);
            click(handle, 1015., 740., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::SaveMacro { .. }))));
            let mut saved = update(true, None, false);
            saved.saved = true;
            updates.send(saved).unwrap();
            settle(cx).await;
            assert_eq!(count(handle, cx), 1, "accepted save did not show success");
            click(handle, 1130., 64., cx);
            settle(cx).await;
            assert_eq!(count(handle, cx), 0);

            click(handle, 1063., 230., cx);
            settle(cx).await;
            dialog::accept(handle, cx).await;
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::DeleteMacro(_)))));
            updates.send(update(false, None, false)).unwrap();
            settle(cx).await;
            assert_eq!(count(handle, cx), 0, "a refresh produced premature success");
            click(handle, 1063., 230., cx);
            assert!(requests.try_recv().is_err(), "a refresh released pending controls");
            updates.send(update(true, Some(CommandError::OperationFailed), false)).unwrap();
            settle(cx).await;
            assert_eq!(count(handle, cx), 1);
            updates.send(update(false, Some(CommandError::OperationFailed), false)).unwrap();
            settle(cx).await;
            assert_eq!(count(handle, cx), 1, "identical errors were duplicated");
            cx.background_executor().timer(Duration::from_secs(6)).await;
            assert_eq!(count(handle, cx), 1, "error hid itself");
            click(handle, 1130., 64., cx); // GPUI-KIT's actual close button.
            settle(cx).await;
            assert_eq!(count(handle, cx), 0, "manual close failed");

            click(handle, 1063., 230., cx);
            settle(cx).await;
            dialog::accept(handle, cx).await;
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::DeleteMacro(_)))));
            updates.send(update(true, None, true)).unwrap();
            settle(cx).await;
            assert_eq!(count(handle, cx), 1, "acknowledged delete did not show success");
            // Keep the pointer outside the toast so its countdown is not paused.
            click(handle, 2., 2., cx);
            cx.background_executor().timer(Duration::from_secs(6)).await;
            assert_eq!(count(handle, cx), 0, "success did not autohide");

            click(handle, 100., 720., cx);
            settle(cx).await;
            assert!(requests.try_recv().is_err(), "opening the collection form submitted a command");
            dialog::key(handle, "enter", cx);
            settle(cx).await;
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::CreateCollection(name))) if name.is_empty()));
            updates.send(update(true, Some(CommandError::CollectionNameRequired), true)).unwrap();
            settle(cx).await;
            assert_eq!(count(handle, cx), 0, "field validation was sent to a toast");

            drop(updates);
            settle(cx).await;
            assert_eq!(count(handle, cx), 1);
            click(handle, 1130., 64., cx);
            settle(cx).await;
            assert_eq!(count(handle, cx), 0);
            settle(cx).await;
            assert_eq!(count(handle, cx), 0, "disconnection was republished every poll");
            println!("Native notifications passed: quiet toggles, request acknowledgements vs refreshes, persistent/deduplicated errors, close button, success autohide, inline name validation and one-shot disconnect.");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
