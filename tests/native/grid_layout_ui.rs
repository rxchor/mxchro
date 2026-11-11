//! Isolated A/B experiment, never imported by production. No engine, hooks or storage.
//! Default: interactive until closed. --check: synthetic interactions + CPU samples.
use gpui_kit::{
    assets::IconName,
    base::{Disableable, Selectable},
    component::{
        ActiveTheme, Icon, Sizable,
        button::Button,
        grid_layout::{
            GridArea, GridItem, GridLayoutEvent, GridLayoutState, grid_drag_handle, grid_layout,
        },
        scroll::{Scrollbar, ScrollbarMode},
    },
    *,
};
use mxchro::{
    domain::types::Action,
    view::{
        assets::AppAssets,
        i18n::{self, action_count, text},
        theme,
        ui::{
            components::AppCheckBox,
            pages::macro_editor::actions_list::{ActionsList, ActionsListEvent},
        },
    },
};
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

rust_i18n::i18n!("locales", fallback = "en");

const ROW_HEIGHT: f32 = 50.;
const ROW_STRIDE: f32 = 58.;
const PANEL_HEIGHT: f32 = 360.;

#[derive(Clone)]
struct Step {
    id: SharedString,
    action: Action,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Panels {
    Both,
    Current,
    Grid,
}

struct Experiment {
    steps: Vec<Step>,
    current: Entity<ActionsList>,
    grid: Entity<GridLayoutState>,
    current_scroll: ScrollHandle,
    grid_scroll: ScrollHandle,
    disabled: bool,
    panels: Panels,
    status: &'static str,
    insertion_matches: bool,
}

fn samples(count: usize) -> Vec<Step> {
    (0..count)
        .map(|index| {
            let action = match index % 4 {
                0 | 2 => Action::Key {
                    key: char::from(b'A' + ((index / 4) % 26) as u8).to_string(),
                    press: index % 4 == 0,
                },
                _ => Action::Wait {
                    duration_ms: (index as u64 + 1) * 10,
                },
            };
            Step {
                id: format!("step-{index}").into(),
                action,
            }
        })
        .collect()
}

fn areas(steps: &[Step]) -> Vec<GridArea> {
    steps
        .iter()
        .enumerate()
        .map(|(row, step)| GridArea::new(step.id.clone()).row(row).max_span(1, 1))
        .collect()
}

fn action_title(action: &Action) -> String {
    match action {
        Action::Key { key, press } => format!(
            "{}  {key}",
            text(if *press {
                "action.press"
            } else {
                "action.release"
            })
        ),
        Action::Wait { duration_ms } => format!(
            "{}  {}",
            text("action.wait"),
            rust_i18n::t!("action.duration", duration = duration_ms)
        ),
    }
}

impl Experiment {
    fn new(cx: &mut Context<Self>) -> Self {
        let steps = samples(25);
        let current_scroll = ScrollHandle::new();
        let current = cx.new(|cx| {
            ActionsList::new(
                steps.iter().map(|step| step.action.clone()).collect(),
                current_scroll.clone(),
                cx,
            )
        });
        let grid = cx.new(|cx| {
            GridLayoutState::new(cx)
                .cols(1)
                .resizable(false)
                .default_areas(areas(&steps))
        });
        cx.subscribe(&current, |this, _, event, cx| {
            if this.disabled {
                return;
            }
            match *event {
                ActionsListEvent::Move { from, to }
                    if from < this.steps.len() && to < this.steps.len() =>
                {
                    let step = this.steps.remove(from);
                    this.steps.insert(to, step);
                    this.status = "experiment.grid.insertion";
                    this.insertion_matches = true;
                }
                ActionsListEvent::Remove(index) if index < this.steps.len() => {
                    this.steps.remove(index);
                }
                _ => return,
            }
            this.synchronize(cx);
        })
        .detach();
        cx.subscribe(&grid, |this, _, event, cx| {
            if let GridLayoutEvent::Change(layout) = event {
                this.commit_grid(layout, cx);
            }
        })
        .detach();
        Self {
            steps,
            current,
            grid,
            current_scroll,
            grid_scroll: ScrollHandle::new(),
            disabled: false,
            panels: Panels::Both,
            status: "experiment.grid.unchanged",
            insertion_matches: true,
        }
    }

    fn synchronize(&mut self, cx: &mut Context<Self>) {
        self.current.update(cx, |list, cx| {
            list.replace_actions(
                self.steps.iter().map(|step| step.action.clone()).collect(),
                cx,
            )
        });
        self.grid
            .update(cx, |grid, cx| grid.set_areas(areas(&self.steps), cx));
        cx.notify();
    }

    fn reset(&mut self, count: usize, cx: &mut Context<Self>) {
        self.steps = samples(count);
        self.current_scroll.set_offset(Point::default());
        self.grid_scroll.set_offset(Point::default());
        self.status = "experiment.grid.unchanged";
        self.insertion_matches = true;
        self.synchronize(cx);
    }

    fn commit_grid(&mut self, layout: &[GridArea], cx: &mut Context<Self>) {
        let by_id: HashMap<_, _> = self
            .steps
            .iter()
            .map(|step| (step.id.clone(), step.clone()))
            .collect();
        let mut layout = layout.to_vec();
        layout.sort_by_key(|area| (area.row, area.col));
        let next: Option<Vec<_>> = layout
            .iter()
            .map(|area| by_id.get(&area.id).cloned())
            .collect();
        let unique: HashSet<_> = layout.iter().map(|area| &area.id).collect();
        let Some(next) = next.filter(|next| {
            next.len() == self.steps.len()
                && unique.len() == next.len()
                && layout
                    .iter()
                    .all(|area| area.col == 0 && area.col_span == 1 && area.row_span == 1)
        }) else {
            self.status = "experiment.grid.invalid";
            self.insertion_matches = false;
            self.synchronize(cx);
            return;
        };
        let selected = self.grid.read(cx).selected_item().cloned();
        let mut expected = self.ids();
        if let Some((from, to)) = selected.and_then(|id| {
            Some((
                expected.iter().position(|candidate| candidate == &id)?,
                next.iter().position(|step| step.id == id)?,
            ))
        }) {
            let id = expected.remove(from);
            expected.insert(to, id);
        }
        self.insertion_matches = next.iter().map(|step| &step.id).eq(expected.iter());
        self.status = if self.insertion_matches {
            "experiment.grid.insertion"
        } else {
            "experiment.grid.different"
        };
        self.steps = next;
        self.synchronize(cx);
    }

    fn ids(&self) -> Vec<SharedString> {
        self.steps.iter().map(|step| step.id.clone()).collect()
    }

    fn set_disabled(&mut self, disabled: bool, window: &mut Window, cx: &mut Context<Self>) {
        if cx.has_active_drag() {
            cx.stop_active_drag(window);
        }
        self.disabled = disabled;
        self.current
            .update(cx, |list, cx| list.set_disabled(disabled, cx));
        self.grid.update(cx, |grid, cx| {
            grid.set_areas(areas(&self.steps), cx);
            grid.set_draggable(!disabled, cx);
        });
        cx.notify();
    }

    fn grid_panel(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut candidate = grid_layout("candidate-grid", &self.grid, cx)
            .row_height(rems(ROW_HEIGHT / f32::from(window.rem_size())))
            .gap(px(ROW_STRIDE - ROW_HEIGHT));
        for step in &self.steps {
            let id = step.id.clone();
            let grip = div()
                .size(px(24.))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .cursor(if self.disabled {
                    CursorStyle::Arrow
                } else {
                    CursorStyle::OpenHand
                })
                .child(Icon::new(IconName::GripVertical).size(px(16.)));
            // The convenience handle registers on_drag even when state.draggable
            // is false. Do not mount that handler while editing is disabled.
            let handle = if self.disabled {
                grip.into_any_element()
            } else {
                grid_drag_handle(&self.grid, step.id.clone())
                    .flex_shrink_0()
                    .child(grip)
                    .into_any_element()
            };
            candidate = candidate.item(
                GridItem::new(step.id.clone())
                    .drag_handle_only(true)
                    .bg(cx.theme().sidebar)
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .accessibility_label(action_title(&step.action))
                    .child(
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .gap_2()
                            .p_2()
                            .child(handle)
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .text_sm()
                                    .child(action_title(&step.action)),
                            )
                            .child(
                                Button::new("remove")
                                    .small()
                                    .label(text("common.remove"))
                                    .disabled(self.disabled)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if !this.disabled {
                                            this.steps.retain(|step| step.id != id);
                                            this.synchronize(cx);
                                        }
                                    })),
                            ),
                    ),
            );
        }
        div()
            .h(px(PANEL_HEIGHT))
            .w_full()
            .min_w_0()
            .p_5()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .relative()
            .child(
                div()
                    .id("grid-scroll")
                    .size_full()
                    .pr_4()
                    .overflow_y_scroll()
                    .track_scroll(&self.grid_scroll)
                    .child(candidate),
            )
            .child(Scrollbar::vertical(&self.grid_scroll).mode(ScrollbarMode::Always))
    }
}

impl Render for Experiment {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut toolbar = div()
            .id("sample-controls")
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2();
        for count in [25, 100, 500] {
            toolbar = toolbar.child(
                Button::new(("count", count))
                    .label(action_count(count))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_active_drag(window);
                        this.reset(count, cx);
                    })),
            );
        }
        for (index, (mode, label)) in [
            (Panels::Both, "experiment.grid.both"),
            (Panels::Current, "experiment.grid.current_only"),
            (Panels::Grid, "experiment.grid.grid_only"),
        ]
        .into_iter()
        .enumerate()
        {
            toolbar = toolbar.child(
                Button::new(("panels", index))
                    .label(text(label))
                    .selected(self.panels == mode)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_active_drag(window);
                        this.panels = mode;
                        cx.notify();
                    })),
            );
        }
        toolbar = toolbar.child(
            div().w(px(170.)).child(
                AppCheckBox::new("disabled", text("experiment.grid.disabled"))
                    .checked(self.disabled)
                    .on_change(cx.listener(|this, value: &bool, window, cx| {
                        this.set_disabled(*value, window, cx)
                    })),
            ),
        );
        let mut panels = div().flex().gap_6().min_w_0();
        if self.panels != Panels::Grid {
            panels = panels.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(div().text_lg().child(text("experiment.grid.current")))
                    .child(
                        div()
                            .h(px(40.))
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(text("experiment.grid.current_help")),
                    )
                    .child(self.current.clone()),
            );
        }
        if self.panels != Panels::Current {
            panels = panels.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(div().text_lg().child(text("experiment.grid.candidate")))
                    .child(
                        div()
                            .h(px(40.))
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(text("experiment.grid.candidate_help")),
                    )
                    .child(self.grid_panel(window, cx)),
            );
        }
        div()
            .id("experiment")
            .size_full()
            .overflow_y_scroll()
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(div().text_2xl().child(text("experiment.grid.title")))
            .child(text("experiment.grid.help"))
            .child(toolbar)
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(text("experiment.grid.reset")),
            )
            .child(panels)
            .child(
                div()
                    .text_sm()
                    .text_color(if self.insertion_matches {
                        cx.theme().muted_foreground
                    } else {
                        cx.theme().warning
                    })
                    .child(text(self.status)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().warning)
                    .child(text("experiment.grid.caveat")),
            )
    }
}

fn draw(handle: AnyWindowHandle, cx: &mut AsyncApp) -> f64 {
    cx.update_window(handle, |_, window, cx| {
        let start = Instant::now();
        window.draw(cx).clear(cx);
        start.elapsed().as_secs_f64() * 1000.
    })
    .unwrap()
}

fn pointer(handle: AnyWindowHandle, position: Point<Pixels>, pressed: bool, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position,
                pressed_button: pressed.then_some(MouseButton::Left),
                modifiers: Modifiers::default(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    draw(handle, cx);
}

fn mouse(handle: AnyWindowHandle, position: Point<Pixels>, down: bool, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        if down {
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
        } else {
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
        }
    })
    .unwrap();
    draw(handle, cx);
}

fn begin_drag(
    handle: AnyWindowHandle,
    source: Point<Pixels>,
    target: Point<Pixels>,
    cx: &mut AsyncApp,
) {
    pointer(handle, source, false, cx);
    mouse(handle, source, true, cx);
    pointer(handle, source + point(px(10.), px(0.)), true, cx);
    pointer(handle, target, true, cx);
    assert!(
        cx.update(|cx| cx.has_active_drag()),
        "native grid handle did not start dragging"
    );
}

async fn checks(handle: AnyWindowHandle, view: &Entity<Experiment>, cx: &mut AsyncApp) {
    cx.background_executor()
        .timer(Duration::from_millis(100))
        .await;
    cx.update(|cx| view.update(cx, |this, cx| this.reset(8, cx)));
    draw(handle, cx);
    let bounds = cx.update(|cx| view.read(cx).grid_scroll.bounds());
    let source = point(bounds.left() + px(20.), bounds.top() + px(25.));
    let target = source + point(px(100.), px(ROW_STRIDE * 3.));
    let initial = cx.update(|cx| view.read(cx).ids());
    begin_drag(handle, source, target, cx);
    assert_eq!(
        cx.update(|cx| view.read(cx).ids()),
        initial,
        "preview changed committed data"
    );
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
    mouse(handle, target, false, cx);
    assert_eq!(
        cx.update(|cx| view.read(cx).ids()),
        initial,
        "Escape did not restore the sequence"
    );
    cx.background_executor()
        .timer(Duration::from_millis(400))
        .await;
    draw(handle, cx);
    begin_drag(handle, source, target, cx);
    mouse(handle, target, false, cx);
    let after = cx.update(|cx| view.read(cx).ids());
    assert_ne!(after, initial);
    assert_eq!(after.iter().collect::<HashSet<_>>().len(), initial.len());
    println!(
        "Grid native drop: insertion_matches={} order={after:?}",
        cx.update(|cx| view.read(cx).insertion_matches)
    );
    cx.update(|cx| view.update(cx, |this, cx| this.reset(8, cx)));
    cx.background_executor()
        .timer(Duration::from_millis(400))
        .await;
    draw(handle, cx);
    begin_drag(handle, source, target, cx);
    let outside = point(bounds.right() + px(20.), target.y);
    pointer(handle, outside, true, cx);
    mouse(handle, outside, false, cx);
    println!(
        "Grid outside drop commits={} (current list would cancel)",
        cx.update(|cx| view.read(cx).ids()) != initial
    );
    assert_ne!(
        cx.update(|cx| view.read(cx).ids()),
        initial,
        "outside drop did not commit the grid preview"
    );
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| this.set_disabled(true, window, cx))
    })
    .unwrap();
    pointer(handle, source, false, cx);
    mouse(handle, source, true, cx);
    pointer(handle, target, true, cx);
    assert!(
        !cx.update(|cx| cx.has_active_drag()),
        "disabled grid started a drag"
    );
    mouse(handle, target, false, cx);
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |this, cx| this.set_disabled(false, window, cx))
    })
    .unwrap();
    for count in [25, 100, 500] {
        for mode in [Panels::Current, Panels::Grid] {
            cx.update(|cx| {
                view.update(cx, |this, cx| {
                    this.reset(count, cx);
                    this.panels = mode;
                    cx.notify();
                })
            });
            draw(handle, cx);
            cx.background_executor()
                .timer(Duration::from_millis(400))
                .await;
            let mut times: Vec<_> = (0..5)
                .map(|_| {
                    cx.update(|cx| view.update(cx, |_, cx| cx.notify()));
                    draw(handle, cx)
                })
                .collect();
            times.sort_by(f64::total_cmp);
            println!(
                "experiment rows={count} panel={mode:?} draw_cpu_p50_ms={:.3} (different card controls; not FPS)",
                times[2]
            );
        }
    }
    println!(
        "Grid experiment checks passed: native drag, deferred commit, Escape, unique identities and disabled. Production is unchanged."
    );
}

fn main() {
    let check = std::env::args().any(|argument| argument == "--check");
    i18n::init();
    gpui_kit::application()
        .with_assets(AppAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            theme::init(cx);
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let (handle, view) = gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: point(px(20.), px(20.)),
                        size: size(px(1280.), px(800.)),
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some(text("experiment.grid.title").into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                cx,
                |_, cx| cx.new(Experiment::new),
            )
            .unwrap();
            if check {
                cx.spawn(async move |cx| {
                    checks(handle, &view, cx).await;
                    cx.update(|cx| cx.quit());
                })
                .detach();
            }
        });
}
