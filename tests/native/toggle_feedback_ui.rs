//! Real MxchroApp with controlled channels, no worker, DB, hooks or OS input.
use gpui_kit::*;
use mxchro::{
    application::{
        commands::Command,
        dto::{CollectionDTO, MacroDTO, MacroEditDTO},
        error::CommandError,
        queries::{Query, Snapshot},
        request::Request,
    },
    domain::types::{Action, Mode},
    view::backend::{CommandBus, Update},
    view::ui::MxchroApp,
};
use std::{sync::mpsc, time::Duration};
#[path = "support/settings_bus.rs"]
mod settings_bus;
#[path = "support/window_capture.rs"]
mod window_capture;

fn snapshot(enabled: bool) -> Snapshot {
    Snapshot {
        hotkey_issues: Vec::new(),
        macros: ["F6", "F7"]
            .into_iter()
            .map(|key| MacroDTO {
                trigger_key: key.into(),
                mode: Mode::Once,
                actions_count: 1,
                block_original_event: false,
                ignore_injected_keys: true,
                enabled: enabled && key == "F6",
                collection_id: Some("work".into()),
                // A preserved grouped legacy assignment must render as inactive.
                toggle_enabled_hotkey: (key == "F6").then(|| "B".into()),
            })
            .collect(),
        collections: vec![CollectionDTO {
            id: "work".into(),
            name: "Work".into(),
            enabled,
            macros_count: 2,
            order: 0,
            toggle_enabled_hotkey: Some("A".into()),
        }],
        keys: vec!["A".into(), "F6".into(), "F7".into()],
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
    draw(handle, cx);
}

fn draw(handle: AnyWindowHandle, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
}

/// Compare native client pixels, not a render count or an FPS estimate.
/// Move off the button so ordinary hover/pressed changes do not affect the result.
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
    cx.background_executor()
        .timer(Duration::from_millis(70))
        .await;
    window_capture::pixels()
}

fn main() {
    mxchro::view::i18n::init();
    gpui_kit::application().with_assets(mxchro::view::assets::AppAssets).run(|cx| {
        gpui_kit::init(cx);
        mxchro::view::theme::init(cx);
        let (commands, requests) = mpsc::channel();
        let (updates, receiver) = mpsc::channel();
        let (handle, _view) = gpui_kit::open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(20.), px(20.)), size: size(px(1160.), px(780.)) })),
            ..Default::default()
        }, cx, |window, cx| cx.new(|cx| MxchroApp::new(CommandBus { requests: commands, updates: receiver }, settings_bus::disconnected(), None, window, cx))).unwrap();
        updates.send(Update { request_completed: false, snapshot: snapshot(false), error: None, saved: false, editor: None }).unwrap();
        cx.spawn(async move |cx| {
            cx.background_executor().timer(Duration::from_millis(150)).await;
            let idle = scene(handle, cx).await;
            click(handle, 1006., 230., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::SetMacroEnabled(key, true))) if key == "F6"), "macro toggle click missed");
            assert!(idle == scene(handle, cx).await, "a pending macro toggle changed idle pixels");
            // Slow reply: the stable-looking controls must still be disabled.
            click(handle, 1006., 230., cx);
            click(handle, 1006., 359., cx);
            click(handle, 1080., 48., cx);
            assert!(requests.try_recv().is_err(), "pending controls submitted another command");
            cx.background_executor().timer(Duration::from_millis(150)).await;
            assert!(idle == scene(handle, cx).await, "pending presentation changed during a slow reply");
            updates.send(Update { request_completed: true, snapshot: snapshot(true), error: None, saved: false, editor: None }).unwrap();
            cx.background_executor().timer(Duration::from_millis(120)).await;
            click(handle, 1006., 230., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::SetMacroEnabled(key, false))) if key == "F6"), "reply did not release the controls or update the toggle value");
            updates.send(Update { request_completed: true, snapshot: snapshot(false), error: None, saved: false, editor: None }).unwrap();
            cx.background_executor().timer(Duration::from_millis(120)).await;

            // Select Work, then exercise the collection toolbar and its select.
            click(handle, 100., 224., cx);
            let idle = scene(handle, cx).await;
            click(handle, 350., 118., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::SetCollectionEnabled(id, true))) if id == "work"), "collection toggle click missed");
            // GPUI's disabled select retains muted text/clear-icon styling, even
            // with its surface stabilized. Only these two tiny glyph regions may
            // differ; buttons, input surface, list and surrounding UI must match.
            assert!(idle.same_outside(&scene(handle, cx).await, &[(282, 190, 298, 205), (609, 190, 626, 205)]), "a pending collection toggle changed unrelated pixels");
            click(handle, 350., 118., cx);
            assert!(requests.try_recv().is_err());
            updates.send(Update { request_completed: true, snapshot: snapshot(false), error: Some(CommandError::OperationFailed), saved: false, editor: None }).unwrap();
            cx.background_executor().timer(Duration::from_millis(120)).await;
            assert!(idle != scene(handle, cx).await, "the error was not presented");
            // Toasts overlay the page; unlike the previous banner they do not move it.
            click(handle, 350., 118., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::SetCollectionEnabled(id, true))) if id == "work"), "rejected command did not restore the controls and authoritative value");
            updates.send(Update { request_completed: true, snapshot: snapshot(false), error: None, saved: false, editor: None }).unwrap();
            cx.update_window(handle, |_, window, cx| {
                gpui_kit::component::WindowExt::clear_notifications(window, cx);
            }).unwrap();
            cx.background_executor().timer(Duration::from_millis(250)).await;
            cx.background_executor().timer(Duration::from_millis(120)).await;
            let recovered = scene(handle, cx).await;
            assert!(idle == recovered, "the UI did not recover after the rejected command");
            // Cards send an ID query, not an entity or the full action sequence.
            click(handle, 964., 326., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Query(Query::EditMacro(key))) if key == "F6"));
            click(handle, 964., 326., cx);
            assert!(requests.try_recv().is_err(), "edit query was submitted twice while pending");
            updates.send(Update { request_completed: true, snapshot: snapshot(false), error: None, saved: false, editor: Some(MacroEditDTO {
                trigger_key: "F6".into(), mode: Mode::Once, actions: vec![Action::Wait { duration_ms: 1 }],
                block_original_event: false, ignore_injected_keys: true, enabled: false,
                collection_id: Some("work".into()), toggle_enabled_hotkey: None,
            }) }).unwrap();
            cx.background_executor().timer(Duration::from_millis(120)).await;
            assert!(recovered != scene(handle, cx).await, "detail response did not open the editor");
            updates.send(Update { request_completed: true, snapshot: snapshot(false), error: None, saved: true, editor: None }).unwrap();
            cx.background_executor().timer(Duration::from_millis(120)).await;
            assert!(recovered == scene(handle, cx).await, "accepted save did not return to the DTO list");
            drop(updates);
            cx.background_executor().timer(Duration::from_millis(120)).await;
            assert!(recovered != scene(handle, cx).await, "backend disconnection must not keep the quiet pending styling");
            click(handle, 350., 182., cx);
            assert!(requests.try_recv().is_err(), "a disconnected backend must leave controls disabled");
            println!("Native checks passed: stable toggle feedback, duplicate blocking, edit-by-ID query and detail DTO, accepted-save return, rejection/recovery and disconnection. No real input or hooks.");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
