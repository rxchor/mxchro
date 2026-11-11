//! Real popovers and MxchroApp with fake responses; no storage/hooks/OS input.
use gpui_kit::{component::WindowExt, *};
use mxchro::{
    application::{
        commands::Command, dto::CollectionDTO, error::CommandError, queries::Snapshot,
        request::Request,
    },
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

fn update(completed: bool, error: Option<CommandError>, name: &str) -> Update {
    Update {
        request_completed: completed,
        error,
        editor: None,
        saved: false,
        snapshot: Snapshot {
            collections: vec![
                CollectionDTO {
                    id: "work".into(),
                    name: name.into(),
                    enabled: false,
                    macros_count: 0,
                    order: 0,
                    toggle_enabled_hotkey: None,
                },
                CollectionDTO {
                    id: "other".into(),
                    name: "Other".into(),
                    enabled: false,
                    macros_count: 0,
                    order: 1,
                    toggle_enabled_hotkey: None,
                },
            ],
            ..Default::default()
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

async fn settle(cx: &mut AsyncApp) {
    cx.background_executor()
        .timer(Duration::from_millis(250))
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
        updates.send(update(false, None, "Work")).unwrap();
        cx.spawn(async move |cx| {
            settle(cx).await;
            click(handle, 100., 720., cx);
            settle(cx).await;
            assert!(requests.try_recv().is_err(), "opening create submitted a command");
            let focus = cx.update_window(handle, |_, window, cx| window.focused(cx)).unwrap();
            assert!(focus.is_some());
            updates.send(update(false, None, "Work")).unwrap();
            settle(cx).await;
            assert_eq!(focus, cx.update_window(handle, |_, window, cx| window.focused(cx)).unwrap(), "refresh replaced or refocused the input");
            dialog::key(handle, "enter", cx);
            settle(cx).await;
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::CreateCollection(name))) if name.is_empty()));
            dialog::key(handle, "enter", cx);
            updates.send(update(false, None, "Work")).unwrap();
            settle(cx).await;
            dialog::key(handle, "enter", cx);
            assert!(requests.try_recv().is_err(), "pending or spontaneous refresh allowed duplicate submit");
            updates.send(update(true, Some(CommandError::CollectionNameRequired), "Work")).unwrap();
            settle(cx).await;
            assert_eq!(cx.update_window(handle, |_, window, cx| window.notifications(cx).len()).unwrap(), 0, "name validation was not inline");
            dialog::key(handle, "escape", cx);
            settle(cx).await;
            dialog::key(handle, "enter", cx);
            assert!(requests.try_recv().is_err(), "Escape submitted the form");
            click(handle, 100., 720., cx);
            settle(cx).await;
            // The create popover is anchored above its sidebar button.
            click(handle, 180., 675., cx);
            settle(cx).await;
            dialog::key(handle, "enter", cx);
            assert!(requests.try_recv().is_err(), "Cancel did not dismiss the form");
            click(handle, 100., 720., cx);
            settle(cx).await;
            click(handle, 270., 675., cx);
            settle(cx).await;
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::CreateCollection(name))) if name.is_empty()), "Create button did not submit");
            updates.send(update(true, Some(CommandError::CollectionNameRequired), "Work")).unwrap();
            settle(cx).await;
            dialog::key(handle, "escape", cx);
            settle(cx).await;
            click(handle, 100., 720., cx);
            settle(cx).await;
            click(handle, 800., 400., cx);
            settle(cx).await;
            dialog::key(handle, "enter", cx);
            assert!(requests.try_recv().is_err(), "outside click submitted the form");

            click(handle, 100., 224., cx);
            settle(cx).await;
            click(handle, 480., 118., cx);
            settle(cx).await;
            assert!(requests.try_recv().is_err(), "opening rename submitted a command");
            updates.send(update(false, None, "Server name")).unwrap();
            settle(cx).await;
            dialog::key(handle, "enter", cx);
            settle(cx).await;
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::RenameCollection(id, name))) if id == "work" && name == "Work"), "rename did not retain the prefilled name across refresh");
            updates.send(update(true, Some(CommandError::EmptyCollectionName), "Server name")).unwrap();
            settle(cx).await;
            dialog::key(handle, "enter", cx);
            settle(cx).await;
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::RenameCollection(id, name))) if id == "work" && name == "Work"), "rejection lost the draft or target");
            updates.send(update(true, None, "Server name")).unwrap();
            settle(cx).await;
            cx.update_window(handle, |_, window, cx| window.clear_notifications(cx)).unwrap();
            dialog::key(handle, "enter", cx);
            assert!(requests.try_recv().is_err(), "accepted save left the form open");

            click(handle, 480., 118., cx);
            settle(cx).await;
            let mut removed = update(false, None, "Server name");
            removed.snapshot.collections.retain(|c| c.id != "work");
            updates.send(removed).unwrap();
            settle(cx).await;
            dialog::key(handle, "enter", cx);
            assert!(requests.try_recv().is_err(), "a stale target was renamed");
            println!("Collection name popovers passed: opening/focus, retained input on refresh, Enter, pending/duplicate guards, inline rejection/retry, Escape/outside cancellation, accepted-save close and stale target invalidation. No hooks or real data.");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
