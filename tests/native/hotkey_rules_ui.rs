//! Native grouped-editor and warning checks. No backend, hooks, disk or OS input.
use gpui_kit::*;
use mxchro::{
    application::dto::{
        CollectionDTO, HotkeyIssueDTO, HotkeyIssueKind, HotkeyOwnerDTO, MacroEditDTO,
    },
    domain::types::{Action, Mode},
    view::{
        assets::AppAssets,
        i18n, theme,
        ui::{
            components::hotkey_warnings::HotkeyWarnings,
            pages::macro_editor::macro_form::{MacroForm, MacroFormEvent},
            state::macro_draft::MacroDraft,
        },
    },
};
use std::{rc::Rc, time::Duration};

struct Probe {
    draft: MacroDraft,
    collections: Vec<CollectionDTO>,
    disabled: bool,
    clears: usize,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .child(MacroForm {
                errors: self.draft.errors.clone(),
                config: self.draft.config.clone(),
                trigger: self.draft.trigger.clone(),
                hotkey: self.draft.hotkey.clone(),
                mode: self.draft.mode.clone(),
                collection: self.draft.collection.clone(),
                disabled: self.disabled,
                capture_enabled: false,
                on_action: Rc::new(cx.listener(|this, event, window, cx| {
                    if let MacroFormEvent::ClearHotkey = event {
                        this.clears += 1;
                        this.draft
                            .hotkey
                            .update(cx, |select, cx| select.set_selected_index(None, window, cx));
                        cx.notify();
                    }
                })),
            })
            .child(HotkeyWarnings {
                collections: self.collections.clone(),
                issues: vec![
                    HotkeyIssueDTO {
                        owner: HotkeyOwnerDTO::Macro("F6".into()),
                        hotkey: "unknown".into(),
                        kind: HotkeyIssueKind::GroupedMacro,
                    },
                    HotkeyIssueDTO {
                        owner: HotkeyOwnerDTO::Collection("work".into()),
                        hotkey: "A".into(),
                        kind: HotkeyIssueKind::Duplicate,
                    },
                    HotkeyIssueDTO {
                        owner: HotkeyOwnerDTO::Macro("B".into()),
                        hotkey: "old-key".into(),
                        kind: HotkeyIssueKind::InvalidKey,
                    },
                ],
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

fn main() {
    i18n::init();
    gpui_kit::application().with_assets(AppAssets).run(|cx| {
        gpui_kit::init(cx);
        theme::init(cx);
        let (handle, view) = gpui_kit::open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(20.), px(20.)), size: size(px(1100.), px(850.)) })),
            ..Default::default()
        }, cx, |window, cx| cx.new(|cx| {
            let collections = vec![CollectionDTO { id: "work".into(), name: "Work".into(), enabled: false, macros_count: 1, order: 0, toggle_enabled_hotkey: Some("A".into()) }];
            let draft = MacroDraft::new(Some(MacroEditDTO {
                trigger_key: "F6".into(), mode: Mode::Once, actions: vec![Action::Wait { duration_ms: 1 }],
                block_original_event: false, ignore_injected_keys: true, enabled: false,
                collection_id: Some("work".into()), toggle_enabled_hotkey: Some("unknown".into()),
            }), None, &["F6".into(), "A".into()], &collections, window, cx);
            Probe { draft, collections, disabled: true, clears: 0 }
        })).unwrap();
        cx.spawn(async move |cx| {
            cx.background_executor().timer(Duration::from_millis(300)).await;
            cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx)).unwrap();
            let entity = cx.update(|cx| {
                let probe = view.read(cx);
                assert_eq!(probe.draft.read_config(cx).unwrap().toggle_enabled_hotkey.as_deref(), Some("unknown"));
                probe.draft.hotkey.entity_id()
            });
            // Clear button below the grouped-hotkey help in the right column.
            click(handle, 650., 290., cx);
            assert_eq!(cx.update(|cx| view.read(cx).clears), 0);
            cx.update(|cx| view.update(cx, |probe, cx| { probe.disabled = false; cx.notify(); }));
            cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx)).unwrap();
            click(handle, 650., 290., cx);
            cx.update(|cx| {
                let probe = view.read(cx);
                assert_eq!(probe.clears, 1, "clear button was not invoked");
                assert_eq!(probe.draft.hotkey.entity_id(), entity);
                let config = probe.draft.read_config(cx).unwrap();
                assert!(config.toggle_enabled_hotkey.is_none());
                assert_eq!(config.collection_id.as_deref(), Some("work"));
            });
            println!("Native hotkey rules passed: warning render, unknown legacy selection retained, grouped clear button, disabled, retained select entity and collection assignment.");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
