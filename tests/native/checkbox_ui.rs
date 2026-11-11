//! Whole-card checkbox hit testing and keyboard interaction. No backend or OS input.
use gpui_kit::{component::ActiveTheme, *};
use mxchro::view::{
    assets::AppAssets,
    i18n::{self, text},
    theme,
    ui::components::AppCheckBox,
};
use std::{cell::RefCell, rc::Rc, time::Duration};

struct Probe {
    checked: [bool; 3],
    disabled: bool,
    events: Vec<(usize, bool)>,
    bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bounds = self.bounds.clone();
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .bg(cx.theme().background)
            .on_children_prepainted(move |children, _, _| *bounds.borrow_mut() = children)
            .children(
                [
                    "editor.block_original",
                    "editor.ignore_injected",
                    "common.enabled",
                ]
                .into_iter()
                .enumerate()
                .map(|(index, label)| {
                    AppCheckBox::new(("option", index), text(label))
                        .checked(self.checked[index])
                        .disabled(self.disabled)
                        .on_change(cx.listener(move |probe, value: &bool, _, cx| {
                            probe.events.push((index, *value));
                            probe.checked[index] = *value;
                            cx.notify();
                        }))
                }),
            )
    }
}

fn draw(handle: AnyWindowHandle, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
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
    draw(handle, cx);
}

fn key(handle: AnyWindowHandle, name: &'static str, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        let keystroke = Keystroke::parse(name).unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
    })
    .unwrap();
    draw(handle, cx);
}

fn main() {
    i18n::init();
    gpui_kit::application().with_assets(AppAssets).run(|cx| {
        gpui_kit::init(cx);
        theme::init(cx);
        let bounds = Rc::new(RefCell::new(Vec::new()));
        let (handle, view) = gpui_kit::open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(20.), px(20.)), size: size(px(500.), px(360.)) })),
            ..Default::default()
        }, cx, |_, cx| cx.new(|_| Probe { checked: [false; 3], disabled: false, events: vec![], bounds: bounds.clone() })).unwrap();
        cx.spawn(async move |cx| {
            cx.background_executor().timer(Duration::from_millis(100)).await;
            draw(handle, cx);
            let cards = bounds.borrow().clone();
            assert_eq!(cards.len(), 3);
            for (index, card) in cards.iter().enumerate() {
                assert!(card.size.width > px(400.), "checkbox does not fill its card");
                for position in [
                    point(card.right() - px(6.), card.center().y), // empty right area
                    point(card.center().x, card.top() + px(4.)), // top padding
                    point(card.left() + px(20.), card.center().y), // indicator
                    point(card.left() + px(75.), card.center().y), // label
                    point(card.left() + px(4.), card.center().y), // left padding
                    point(card.right() - px(1.), card.center().y), // border
                ] {
                    let (before, previous) = cx.update(|cx| { let probe = view.read(cx); (probe.events.len(), probe.checked[index]) });
                    click(handle, position, cx);
                    cx.update(|cx| {
                        let probe = view.read(cx);
                        assert_eq!(probe.events.len(), before + 1, "card {index} did not emit exactly once at {position:?}");
                        assert_eq!(probe.events.last(), Some(&(index, !previous)));
                        assert_eq!(probe.checked[index], !previous);
                    });
                }
            }
            // Padding does not install an extra focus target; the native control
            // still owns focus and keyboard activation.
            cx.update_window(handle, |_, window, cx| window.focus_next(cx)).unwrap();
            draw(handle, cx);
            let before = cx.update(|cx| view.read(cx).events.len());
            key(handle, "space", cx);
            assert_eq!(cx.update(|cx| view.read(cx).events.len()), before + 1);
            assert_eq!(cx.update(|cx| view.read(cx).events.last().copied()), Some((0, true)));
            key(handle, "tab", cx);
            key(handle, "space", cx);
            assert_eq!(cx.update(|cx| view.read(cx).events.last().copied()), Some((1, true)));

            cx.update(|cx| view.update(cx, |probe, cx| { probe.disabled = true; cx.notify(); }));
            draw(handle, cx);
            let before = cx.update(|cx| view.read(cx).events.len());
            for card in &cards {
                for x in [card.left() + px(20.), card.left() + px(75.), card.right() - px(6.)] {
                    click(handle, point(x, card.center().y), cx);
                }
            }
            key(handle, "space", cx);
            assert_eq!(cx.update(|cx| view.read(cx).events.len()), before, "disabled card activated");
            println!("Native checkbox checks passed: full width, padding/border/indicator/label, exactly one event, keyboard/tab and disabled.");
            cx.update(|cx| cx.quit());
        }).detach();
    });
}
