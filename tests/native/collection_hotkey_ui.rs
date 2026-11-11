//! Opt-in native render/interaction checks. No backend, database, hooks or OS input.
use gpui_kit::component::select::SelectEvent;
use gpui_kit::*;
use mxchro::{
    application::dto::CollectionDTO,
    view::ui::{
        components::collection_toolbar::CollectionToolbar,
        state::collection_hotkey::CollectionHotkeyDraft,
        state::collection_name::CollectionNameDraft,
    },
};
use std::{rc::Rc, time::Duration};

struct Probe {
    collection: CollectionDTO,
    draft: CollectionHotkeyDraft,
    name_form: CollectionNameDraft,
    disabled: bool,
    confirmations: Vec<Option<String>>,
    _subscription: Subscription,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().p_6().child(CollectionToolbar {
            name_form: self.name_form.clone(),
            collection: self.collection.clone(),
            hotkey: Some(self.draft.select.clone()),
            disabled: self.disabled,
            stable_pending: false,
            capture_enabled: false,
            on_action: Rc::new(|_, _, _| {}),
        })
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
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
}

fn main() {
    mxchro::view::i18n::init();
    gpui_kit::application().with_assets(mxchro::view::assets::AppAssets).run(|cx| {
        gpui_kit::init(cx);
        mxchro::view::theme::init(cx);
        let (handle, view) = gpui_kit::open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(20.), px(20.)), size: size(px(900.), px(600.)) })),
            ..Default::default()
        }, cx, |window, cx| cx.new(|cx| {
            let collection = CollectionDTO { id: "fixture".into(), name: "Work".into(), enabled: false, macros_count: 0, order: 0, toggle_enabled_hotkey: Some("A".into()) };
            let draft = CollectionHotkeyDraft::new(&collection, &["A".into(), "B".into(), "F6".into()], window, cx);
            let subscription = cx.subscribe(&draft.select, |probe: &mut Probe, _, event, cx| {
                let SelectEvent::Confirm(value) = event;
                probe.confirmations.push(value.clone());
                cx.notify();
            });
            Probe { collection, draft, name_form: CollectionNameDraft::new(window, cx), disabled: true, confirmations: vec![], _subscription: subscription }
        })).unwrap();
        cx.spawn(async move |cx| {
            cx.background_executor().timer(Duration::from_millis(150)).await;
            cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx)).unwrap();
            // The select follows a 32 px button row, 16 px gap, label and 8 px gap.
            click(handle, 366., 116., cx);
            assert!(cx.update(|cx| view.read(cx).confirmations.is_empty()));
            cx.update(|cx| view.update(cx, |probe, cx| { probe.disabled = false; cx.notify(); }));
            cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx)).unwrap();
            click(handle, 366., 116., cx);
            assert_eq!(cx.update(|cx| view.read(cx).confirmations.clone()), vec![None]);
            // Simulate a failed save: reconcile with the authoritative A snapshot.
            cx.update_window(handle, |_, window, cx| view.update(cx, |probe, cx| {
                let entity = probe.draft.select.entity_id();
                probe.draft.sync(&probe.collection, &["A".into(), "B".into(), "F6".into()], window, cx);
                assert_eq!(probe.draft.select.entity_id(), entity);
                assert_eq!(probe.draft.select.read(cx).selected_value().map(String::as_str), Some("A"));
                probe.collection.toggle_enabled_hotkey = Some("B".into());
                probe.draft.sync(&probe.collection, &["A".into(), "B".into(), "F6".into()], window, cx);
                assert_eq!(probe.draft.select.read(cx).selected_value().map(String::as_str), Some("B"));
                probe.collection.toggle_enabled_hotkey = None;
                probe.draft.sync(&probe.collection, &["A".into(), "B".into(), "F6".into()], window, cx);
                assert_eq!(probe.draft.select.read(cx).selected_value(), None);
            })).unwrap();
            // Programmatic synchronization must not emit another save request.
            assert_eq!(cx.update(|cx| view.read(cx).confirmations.clone()), vec![None]);
            cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx)).unwrap();
            click(handle, 100., 116., cx);
            click(handle, 100., 195., cx);
            assert_eq!(cx.update(|cx| view.read(cx).confirmations.clone()), vec![None, Some("A".into())]);
            cx.update_window(handle, |_, window, cx| view.update(cx, |probe, cx| {
                probe.collection.toggle_enabled_hotkey = Some("legacy-unknown".into());
                probe.draft.sync(&probe.collection, &["A".into(), "B".into(), "F6".into()], window, cx);
                assert_eq!(probe.draft.select.read(cx).selected_value().map(String::as_str), Some("legacy-unknown"));
            })).unwrap();
            cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx)).unwrap();
            click(handle, 366., 116., cx);
            assert_eq!(cx.update(|cx| view.read(cx).confirmations.clone()), vec![None, Some("A".into()), None]);
            println!("Collection hotkey native checks passed: render, disabled, clear, dropdown selection, authoritative restore/replace/clear, retained entity, no confirm loop.");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
