//! Actual MxchroApp capture controls with a fake input port. No hooks, OS input,
//! macro execution, database or settings files.
use gpui_kit::component::WindowExt;
use gpui_kit::*;
use mxchro::{
    application::{
        commands::Command,
        dto::{CollectionDTO, MacroDTO, MacroEditDTO, SaveMacroDTO},
        ports::{CaptureError, CaptureOutcome, CaptureSession, KeyCaptureBackend},
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
use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};
#[path = "support/settings_bus.rs"]
mod settings_bus;
#[allow(dead_code)]
#[path = "support/window_capture.rs"]
mod window_capture;

#[derive(Default)]
struct FakeState {
    next: u32,
    active: Option<CaptureSession>,
    outcome: Option<CaptureOutcome>,
    failure: Option<CaptureError>,
}
#[derive(Default)]
struct FakeCapture(Mutex<FakeState>);
struct FocusProbe;
impl Render for FocusProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child("Focus probe")
    }
}
impl FakeCapture {
    fn active(&self) -> bool {
        self.0.lock().unwrap().active.is_some()
    }
    fn fail_next(&self, error: CaptureError) {
        self.0.lock().unwrap().failure = Some(error);
    }
    fn finish(&self, outcome: CaptureOutcome) {
        let mut state = self.0.lock().unwrap();
        assert!(state.active.is_some(), "capture button was not invoked");
        state.outcome = Some(outcome);
    }
}
impl KeyCaptureBackend for FakeCapture {
    fn start(&self) -> Result<CaptureSession, CaptureError> {
        let mut state = self.0.lock().unwrap();
        if let Some(error) = state.failure.take() {
            return Err(error);
        }
        if state.active.is_some() {
            return Err(CaptureError::Busy);
        }
        state.next += 1;
        let session = CaptureSession(state.next);
        state.active = Some(session);
        Ok(session)
    }
    fn cancel(&self, session: CaptureSession) {
        let mut state = self.0.lock().unwrap();
        if state.active == Some(session) {
            state.active = None;
            state.outcome = None;
        }
    }
    fn poll(&self, session: CaptureSession) -> Option<CaptureOutcome> {
        let mut state = self.0.lock().unwrap();
        (state.active == Some(session))
            .then(|| state.outcome.take())
            .flatten()
    }
}

fn snapshot() -> Snapshot {
    Snapshot {
        hotkey_issues: vec![],
        macros: vec![MacroDTO {
            trigger_key: "F6".into(),
            mode: Mode::Once,
            actions_count: 1,
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
        keys: ["A", "B", "F6", "F7", "mouse_x1", "left_shift"]
            .map(str::to_owned)
            .into(),
    }
}
fn update(editor: Option<MacroEditDTO>, request_completed: bool) -> Update {
    Update {
        snapshot: snapshot(),
        request_completed,
        editor,
        saved: false,
        error: None,
    }
}
fn editing() -> MacroEditDTO {
    MacroEditDTO {
        trigger_key: "F6".into(),
        mode: Mode::Once,
        actions: vec![Action::Wait { duration_ms: 1 }],
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
fn notification_count(handle: AnyWindowHandle, cx: &mut AsyncApp) -> usize {
    cx.update_window(handle, |_, window, cx| window.notifications(cx).len())
        .unwrap()
}
fn clear_notifications(handle: AnyWindowHandle, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| window.clear_notifications(cx))
        .unwrap();
}
async fn settle(cx: &mut AsyncApp) {
    cx.background_executor()
        .timer(Duration::from_millis(600))
        .await;
}
fn save(
    handle: AnyWindowHandle,
    requests: &mpsc::Receiver<Request>,
    cx: &mut AsyncApp,
) -> SaveMacroDTO {
    scroll(handle, true, cx);
    if std::env::var_os("MXCHRO_CAPTURE_DIR").is_some() {
        window_capture::pixels();
    }
    click(handle, 1015., 740., cx);
    match requests
        .try_recv()
        .expect("save button did not send a request")
    {
        Request::Command(Command::SaveMacro { config, .. }) => config,
        _ => panic!("unexpected request"),
    }
}

fn main() {
    i18n::init();
    gpui_kit::application().with_assets(AppAssets).with_quit_mode(QuitMode::Explicit).run(|cx| {
        gpui_kit::init(cx);
        theme::init(cx);
        let (commands, requests) = mpsc::channel();
        let (updates, receiver) = mpsc::channel();
        let capture = Arc::new(FakeCapture::default());
        let backend = capture.clone();
        let (handle, _) = gpui_kit::open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(20.), px(20.)), size: size(px(1160.), px(780.)) })),
            ..Default::default()
        }, cx, |window, cx| cx.new(|cx| MxchroApp::new(CommandBus { requests: commands, updates: receiver }, settings_bus::disconnected(), None, window, cx).with_key_capture(backend))).unwrap();
        updates.send(update(None, false)).unwrap();
        cx.spawn(async move |cx| {
            settle(cx).await;
            click(handle, 947., 230., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Query(Query::EditMacro(_)))));
            updates.send(update(Some(editing()), true)).unwrap();
            settle(cx).await;
            if std::env::var_os("MXCHRO_CAPTURE_DIR").is_some() { window_capture::pixels(); }

            for error in [CaptureError::Unavailable, CaptureError::Busy] {
                capture.fail_next(error);
                click(handle, 676., 147., cx);
                settle(cx).await;
                assert!(!capture.active());
                assert_eq!(notification_count(handle, cx), 1, "capture startup error was not a notification");
                clear_notifications(handle, cx);
                settle(cx).await;
            }
            click(handle, 676., 147., cx);
            assert!(capture.active(), "trigger capture button missed");
            settle(cx).await;
            assert_eq!(notification_count(handle, cx), 1, "listening notification missing");
            // Active capture disables submission and other capture requests.
            scroll(handle, true, cx);
            click(handle, 1015., 740., cx);
            assert!(requests.try_recv().is_err(), "save was enabled during capture");
            capture.finish(CaptureOutcome::Key { vk: 118, key: Some("F7".into()) });
            settle(cx).await;
            assert!(!capture.active());
            assert_eq!(notification_count(handle, cx), 0, "listening notification survived completion");
            let config = save(handle, &requests, cx);
            assert_eq!(config.trigger_key, "F7");
            assert_eq!(config.actions.len(), 1);
            updates.send(update(None, true)).unwrap();
            settle(cx).await;
            scroll(handle, false, cx);

            click(handle, 1098., 265., cx);
            assert!(capture.active(), "hotkey capture button missed");
            capture.finish(CaptureOutcome::Key { vk: 5, key: Some("mouse_x1".into()) });
            settle(cx).await;
            let config = save(handle, &requests, cx);
            assert_eq!(config.toggle_enabled_hotkey.as_deref(), Some("mouse_x1"));
            updates.send(update(None, true)).unwrap();
            settle(cx).await;
            scroll(handle, false, cx);

            click(handle, 676., 147., cx);
            capture.finish(CaptureOutcome::Key { vk: 255, key: None });
            settle(cx).await;
            assert_eq!(notification_count(handle, cx), 1, "unsupported VK was not a notification");
            clear_notifications(handle, cx);
            settle(cx).await;
            let config = save(handle, &requests, cx);
            assert_eq!(config.trigger_key, "F7", "unsupported VK changed the previous selection");
            updates.send(update(None, true)).unwrap();
            settle(cx).await;
            scroll(handle, false, cx);

            click(handle, 676., 147., cx);
            capture.finish(CaptureOutcome::Cancelled);
            settle(cx).await;
            assert!(!capture.active());
            assert_eq!(notification_count(handle, cx), 0, "listening notification survived cancellation");
            let config = save(handle, &requests, cx);
            assert_eq!(config.trigger_key, "F7");
            updates.send(update(None, true)).unwrap();
            settle(cx).await;

            scroll(handle, false, cx);
            click(handle, 1103., 607., cx);
            assert!(capture.active(), "action capture button missed");
            capture.finish(CaptureOutcome::Key { vk: 160, key: Some("left_shift".into()) });
            settle(cx).await;
            click(handle, 872., 663., cx);
            let config = save(handle, &requests, cx);
            assert_eq!(config.actions.len(), 3);
            assert!(matches!(&config.actions[1], Action::Key { key, press: true } if key == "left_shift"));
            assert!(matches!(&config.actions[2], Action::Key { key, press: false } if key == "left_shift"));
            updates.send(update(None, true)).unwrap();
            settle(cx).await;

            scroll(handle, false, cx);
            click(handle, 676., 147., cx);
            assert!(capture.active());
            let (focus_handle, _) = cx.update(|cx| gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| FocusProbe)).unwrap());
            settle(cx).await;
            assert!(!capture.active(), "capture survived losing window focus");
            cx.update_window(focus_handle, |_, window, _| window.remove_window()).unwrap();
            cx.update_window(handle, |_, window, _| window.activate_window()).unwrap();
            settle(cx).await;
            assert!(cx.update_window(handle, |_, window, _| window.is_window_active()).unwrap(), "fixture window did not regain focus");
            scroll(handle, false, cx);
            if std::env::var_os("MXCHRO_CAPTURE_DIR").is_some() { window_capture::pixels(); }
            click(handle, 676., 147., cx);
            assert!(capture.active());
            cx.background_executor().timer(Duration::from_millis(10_300)).await;
            assert!(!capture.active(), "capture did not time out");
            assert_eq!(notification_count(handle, cx), 1, "timeout was not a notification");
            clear_notifications(handle, cx);
            settle(cx).await;
            let config = save(handle, &requests, cx);
            assert_eq!(config.trigger_key, "F7");
            updates.send(update(None, true)).unwrap();
            settle(cx).await;

            let mut grouped = editing();
            grouped.collection_id = Some("work".into());
            grouped.toggle_enabled_hotkey = Some("A".into());
            updates.send(update(Some(grouped), false)).unwrap();
            settle(cx).await;
            scroll(handle, false, cx);
            click(handle, 1098., 265., cx);
            assert!(!capture.active(), "grouped individual hotkey capture was enabled");
            click(handle, 676., 147., cx);
            assert!(capture.active());
            updates.send(update(Some(editing()), false)).unwrap();
            settle(cx).await;
            assert!(!capture.active(), "replacing an editor kept an obsolete capture");

            // Sidebar navigation cancels an active capture and discards the editor.
            scroll(handle, false, cx);
            click(handle, 676., 147., cx);
            assert!(capture.active());
            click(handle, 80., 210., cx);
            settle(cx).await;
            assert!(!capture.active(), "sidebar navigation kept the editor capture alive");
            if std::env::var_os("MXCHRO_CAPTURE_DIR").is_some() { window_capture::pixels(); }
            click(handle, 618., 197., cx);
            assert!(capture.active(), "collection capture button missed");
            capture.finish(CaptureOutcome::Key { vk: 65, key: Some("A".into()) });
            settle(cx).await;
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::SetCollectionHotkey(id, Some(key)))) if id == "work" && key == "A"));
            assert!(requests.try_recv().is_err(), "programmatic selection emitted duplicate saves");
            let mut accepted = update(None, true);
            accepted.snapshot.collections[0].toggle_enabled_hotkey = Some("A".into());
            updates.send(accepted).unwrap();
            settle(cx).await;
            clear_notifications(handle, cx);
            settle(cx).await;
            click(handle, 618., 197., cx);
            capture.finish(CaptureOutcome::Key { vk: 255, key: None });
            settle(cx).await;
            assert!(requests.try_recv().is_err(), "unknown collection VK was saved");
            assert_eq!(notification_count(handle, cx), 1);
            clear_notifications(handle, cx);
            settle(cx).await;
            // The original selector still has A and can be cleared manually.
            click(handle, 577., 197., cx);
            assert!(matches!(requests.try_recv(), Ok(Request::Command(Command::SetCollectionHotkey(id, None))) if id == "work"));
            updates.send(update(None, true)).unwrap();
            settle(cx).await;
            clear_notifications(handle, cx);
            settle(cx).await;
            // Closing the listening toast cancels its session without bringing
            // it back on the next poll or affecting a newer session.
            click(handle, 618., 197., cx);
            settle(cx).await;
            assert_eq!(notification_count(handle, cx), 1);
            clear_notifications(handle, cx);
            settle(cx).await;
            assert!(!capture.active());
            assert_eq!(notification_count(handle, cx), 0);
            click(handle, 618., 197., cx);
            assert!(capture.active());
            cx.update_window(handle, |_, window, _| window.remove_window()).unwrap();
            settle(cx).await;
            assert!(!capture.active(), "capture survived closing the window");
            println!("Native capture checks passed: icon controls, listening/error notifications and dismissal, keyboard/mouse selection, unsupported/cancelled input, timeout, focus loss, stale editor, close, grouped/pending controls and one collection autosave. No hooks or real input.");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
