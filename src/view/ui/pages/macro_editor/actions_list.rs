use super::action_drag::{
    ActionDrag, ActionDragHandle, edge_scroll_step, insertion_slot, move_destination,
};
use super::action_motion::ActionMotion;
use super::layout::{
    ACTION_PANEL_HEIGHT, ACTION_ROW_HEIGHT, ACTION_ROW_SPACING, ACTION_ROW_STRIDE, panel,
};
use crate::domain::types::Action;
use crate::view::i18n::{key_label, text};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    assets::IconName,
    base::{Disableable, StyledExt},
    component::{
        ActiveTheme, Icon, Sizable,
        button::Button,
        scroll::{Scrollbar, ScrollbarMode},
    },
    *,
};
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub enum ActionsListEvent {
    /// Move to the final index, preserving the order of intervening actions.
    Move {
        from: usize,
        to: usize,
    },
    Remove(usize),
}

pub struct ActionsList {
    actions: Vec<Action>,
    disabled: bool,
    scroll_handle: UniformListScrollHandle,
    revision: u64,
    dragging: Option<ActionDrag>,
    motion: Option<ActionMotion>,
    drag_generation: u64,
    drop_slot: Option<usize>,
    focus: FocusHandle,
}

impl EventEmitter<ActionsListEvent> for ActionsList {}

impl ActionsList {
    pub fn new(actions: Vec<Action>, scroll: ScrollHandle, cx: &mut App) -> Self {
        let scroll_handle = UniformListScrollHandle::new();
        scroll_handle.0.borrow_mut().base_handle = scroll;
        Self {
            actions,
            disabled: false,
            scroll_handle,
            revision: 0,
            dragging: None,
            motion: None,
            drag_generation: 0,
            drop_slot: None,
            focus: cx.focus_handle(),
        }
    }

    /// The editor owns the canonical sequence. Only edits replace this view's data.
    /// Scroll notifications target this entity; GPUI can still redraw ancestors,
    /// so virtualization, not entity separation alone, bounds the per-frame work.
    pub fn replace_actions(&mut self, actions: Vec<Action>, cx: &mut Context<Self>) {
        self.actions = actions;
        self.revision = self.revision.wrapping_add(1);
        self.dragging = None;
        self.motion = None;
        self.drop_slot = None;
        cx.notify();
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        if self.disabled != disabled {
            self.disabled = disabled;
            self.revision = self.revision.wrapping_add(1);
            self.dragging = None;
            self.motion = None;
            self.drop_slot = None;
            cx.notify();
        }
    }

    fn accepts_drag(&self, drag: &ActionDrag, cx: &Context<Self>) -> bool {
        !self.disabled
            && drag.list == cx.entity_id()
            && drag.revision == self.revision
            && drag.from < self.actions.len()
    }

    fn slot_at_pointer(&self, window: &Window) -> Option<usize> {
        let scroll = &self.scroll_handle.0.borrow().base_handle;
        let bounds = scroll.bounds();
        let pointer = window.mouse_position();
        if !bounds.contains(&pointer) || pointer.x >= bounds.right() - px(16.) {
            return None;
        }
        Some(insertion_slot(
            f32::from(pointer.y - bounds.top() - scroll.offset().y),
            self.actions.len(),
        ))
    }

    fn update_drag(&mut self, window: &Window, cx: &mut Context<Self>) {
        let slot = self.slot_at_pointer(window);
        if self.drop_slot != slot {
            self.drop_slot = slot;
            if let (Some(drag), Some(motion)) = (&self.dragging, &mut self.motion) {
                let to = slot
                    .and_then(|slot| move_destination(drag.from, slot, self.actions.len()))
                    .unwrap_or(drag.from);
                motion.retarget(drag.from, to, Instant::now());
            }
            cx.notify();
        }
    }

    fn start_drag(&mut self, drag: &ActionDrag, window: &mut Window, cx: &mut Context<Self>) {
        if !self.accepts_drag(drag, cx) {
            return;
        }
        self.dragging = Some(drag.clone());
        self.motion = Some(ActionMotion::new(self.actions.len(), Instant::now()));
        self.drag_generation = self.drag_generation.wrapping_add(1);
        let generation = self.drag_generation;
        self.focus.focus(window, cx);
        self.update_drag(window, cx);
        // One GPUI task per gesture, not a thread or task per row/mouse move.
        // It also clears the marker when a drop outside the list cancels the drag.
        cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                if !matches!(
                    this.update_in(cx, |this, window, cx| {
                        if this.drag_generation != generation {
                            return false;
                        }
                        if !cx.has_active_drag()
                            || this
                                .dragging
                                .as_ref()
                                .is_none_or(|drag| !this.accepts_drag(drag, cx))
                        {
                            this.dragging = None;
                            this.motion = None;
                            this.drop_slot = None;
                            cx.notify();
                            return false;
                        }
                        let scroll = this.scroll_handle.0.borrow().base_handle.clone();
                        let bounds = scroll.bounds();
                        let pointer = window.mouse_position();
                        if bounds.contains(&pointer) && pointer.x < bounds.right() - px(16.) {
                            let step = edge_scroll_step(
                                f32::from(pointer.y - bounds.top()),
                                f32::from(bounds.size.height),
                            );
                            let old = scroll.offset();
                            let y = (old.y + px(step)).clamp(-scroll.max_offset().y, px(0.));
                            if y != old.y {
                                scroll.set_offset(point(old.x, y));
                                cx.notify();
                            }
                        }
                        this.update_drag(window, cx);
                        if this
                            .motion
                            .as_mut()
                            .is_some_and(|motion| motion.tick(Instant::now()))
                        {
                            cx.notify();
                        }
                        true
                    }),
                    Ok(true)
                ) {
                    break;
                }
            }
        })
        .detach();
        cx.notify();
    }

    fn drop_action(&mut self, drag: &ActionDrag, window: &Window, cx: &mut Context<Self>) {
        let to = self
            .accepts_drag(drag, cx)
            .then(|| self.slot_at_pointer(window))
            .flatten()
            .and_then(|slot| move_destination(drag.from, slot, self.actions.len()));
        self.dragging = None;
        self.motion = None;
        self.drop_slot = None;
        if let Some(to) = to {
            cx.emit(ActionsListEvent::Move {
                from: drag.from,
                to,
            });
        }
        cx.notify();
    }

    fn render_row(&self, visual_index: usize, now: Instant, cx: &mut Context<Self>) -> Div {
        let index = self
            .motion
            .as_ref()
            .map_or(visual_index, |motion| motion.original_index(visual_index));
        let offset = self.motion.as_ref().map_or(0., |motion| {
            (motion.position(index, now) - visual_index as f32) * ACTION_ROW_STRIDE
        });
        let gap = self.dragging.as_ref().and_then(|drag| {
            self.drop_slot.map(|slot| {
                move_destination(drag.from, slot, self.actions.len()).unwrap_or(drag.from)
            })
        });
        div()
            .relative()
            .h(px(ACTION_ROW_STRIDE))
            .pb(px(ACTION_ROW_SPACING))
            // Paint behind the moving cards, without affecting virtual row size
            // or adding a hit target that could interfere with the drop area.
            .when(gap == Some(visual_index), |row| {
                row.child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top_0()
                        .h(px(ACTION_ROW_HEIGHT))
                        .rounded_md()
                        .border_1()
                        .border_dashed()
                        .border_color(cx.theme().primary.opacity(0.5))
                        .bg(cx.theme().primary.opacity(0.08)),
                )
            })
            .child(
                div()
                    .relative()
                    .top(px(offset))
                    .h(px(ACTION_ROW_HEIGHT))
                    // Keep the virtual row's geometry and insertion marker while
                    // showing only the floating preview for the dragged action.
                    .when(
                        self.dragging
                            .as_ref()
                            .is_some_and(|drag| drag.from == index),
                        |card| card.opacity(0.),
                    )
                    .flex()
                    .gap_2()
                    .items_center()
                    .justify_between()
                    .p_2()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_md()
                    .child(ActionDragHandle {
                        drag: ActionDrag {
                            list: cx.entity_id(),
                            revision: self.revision,
                            from: index,
                            action: self.actions[index].clone(),
                        },
                        disabled: self.disabled,
                        on_start: Rc::new(
                            cx.listener(|this, drag, window, cx| this.start_drag(drag, window, cx)),
                        ),
                    })
                    .child(action_content(&self.actions[index], cx))
                    .child(
                        div()
                            .flex()
                            .flex_shrink_0()
                            .gap_2()
                            .child(
                                Button::new(("up", index))
                                    .small()
                                    .label("↑")
                                    .tooltip(text("editor.move_action_up"))
                                    .accessibility_label(text("editor.move_action_up"))
                                    .disabled(self.disabled || index == 0)
                                    .on_click(cx.listener(move |_, _, _, cx| {
                                        cx.emit(ActionsListEvent::Move {
                                            from: index,
                                            to: index.saturating_sub(1),
                                        });
                                    })),
                            )
                            .child(
                                Button::new(("down", index))
                                    .small()
                                    .label("↓")
                                    .tooltip(text("editor.move_action_down"))
                                    .accessibility_label(text("editor.move_action_down"))
                                    .disabled(self.disabled || index + 1 == self.actions.len())
                                    .on_click(cx.listener(move |_, _, _, cx| {
                                        cx.emit(ActionsListEvent::Move {
                                            from: index,
                                            to: index + 1,
                                        });
                                    })),
                            )
                            .child(
                                Button::new(("remove", index))
                                    .small()
                                    .icon(IconName::Close)
                                    .tooltip(text("common.remove"))
                                    .accessibility_label(text("common.remove"))
                                    .disabled(self.disabled)
                                    .on_click(cx.listener(move |_, _, _, cx| {
                                        cx.emit(ActionsListEvent::Remove(index));
                                    })),
                            ),
                    ),
            )
    }
}

struct ActionPresentation {
    icon: IconName,
    label: String,
    detail: String,
    is_key: bool,
}

fn action_presentation(action: &Action) -> ActionPresentation {
    match action {
        Action::Key { key, press } => ActionPresentation {
            icon: if *press {
                IconName::ArrowDownToLine
            } else {
                IconName::ArrowUpFromLine
            },
            label: if *press {
                text("action.press")
            } else {
                text("action.release")
            },
            detail: key_label(key),
            is_key: true,
        },
        Action::Wait { duration_ms } => ActionPresentation {
            icon: IconName::Timer,
            label: text("action.wait"),
            detail: rust_i18n::t!("action.duration", duration = duration_ms).into_owned(),
            is_key: false,
        },
    }
}

pub(super) fn action_content(action: &Action, cx: &App) -> Div {
    let presentation = action_presentation(action);
    let detail = if presentation.is_key {
        div()
            .px_2()
            .py_1()
            .rounded_sm()
            .bg(cx.theme().primary)
            .text_color(cx.theme().primary_foreground)
            .text_sm()
            .font_semibold()
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap()
            .child(presentation.detail)
    } else {
        div()
            .text_sm()
            .font_semibold()
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap()
            .child(presentation.detail)
    };
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .size(px(32.))
                .flex_shrink_0()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().muted)
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(presentation.icon)
                        .size(px(16.))
                        .text_color(cx.theme().foreground),
                ),
        )
        .child(
            div()
                .w(px(56.))
                .flex_shrink_0()
                .text_xs()
                .font_semibold()
                .text_color(cx.theme().muted_foreground)
                .child(presentation.label),
        )
        .child(div().flex_1().min_w_0().flex().child(detail))
}

impl Render for ActionsList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let drop_owner = cx.entity().downgrade();
        let content = if self.actions.is_empty() {
            div()
                .id("actions-scroll")
                .size_full()
                .pr_4()
                .track_scroll(&self.scroll_handle.0.borrow().base_handle)
                .overflow_y_scroll()
                .text_color(cx.theme().muted_foreground)
                .child(text("editor.empty_actions"))
                .into_any_element()
        } else {
            uniform_list(
                "actions-scroll",
                self.actions.len(),
                cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                    let now = Instant::now();
                    let first = range.start;
                    let last = range.end.saturating_sub(1);
                    range
                        .map(|index| {
                            let mut row = this.render_row(index, now, cx);
                            // One overscan card on either side keeps moving cards
                            // visible at viewport edges without rendering the list.
                            if this.motion.is_some() && index == first && index > 0 {
                                row = row.child(
                                    div()
                                        .absolute()
                                        .left_0()
                                        .right_0()
                                        .top(px(-ACTION_ROW_STRIDE))
                                        .child(this.render_row(index - 1, now, cx)),
                                );
                            }
                            if this.motion.is_some()
                                && index == last
                                && index + 1 < this.actions.len()
                            {
                                row = row.child(
                                    div()
                                        .absolute()
                                        .left_0()
                                        .right_0()
                                        .top(px(ACTION_ROW_STRIDE))
                                        .child(this.render_row(index + 1, now, cx)),
                                );
                            }
                            row
                        })
                        .collect::<Vec<_>>()
                }),
            )
            .size_full()
            .pr_4()
            .track_scroll(&self.scroll_handle)
            .into_any_element()
        };
        panel(cx).h(px(ACTION_PANEL_HEIGHT)).child(
            div()
                .id("action-drop-area")
                .track_focus(&self.focus)
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" && this.dragging.is_some() {
                        cx.stop_active_drag(window);
                        this.dragging = None;
                        this.motion = None;
                        this.drop_slot = None;
                        cx.stop_propagation();
                        cx.notify();
                    }
                }))
                .can_drop(move |value, window, cx| {
                    drop_owner
                        .update(cx, |this, cx| {
                            value
                                .downcast_ref::<ActionDrag>()
                                .is_some_and(|drag| this.accepts_drag(drag, cx))
                                && this.slot_at_pointer(window).is_some()
                        })
                        .unwrap_or(false)
                })
                .on_drag_move(
                    cx.listener(|this, event: &DragMoveEvent<ActionDrag>, window, cx| {
                        if this.accepts_drag(event.drag(cx), cx) {
                            this.update_drag(window, cx);
                        }
                    }),
                )
                .on_drop(cx.listener(|this, drag: &ActionDrag, window, cx| {
                    this.drop_action(drag, window, cx)
                }))
                .relative()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .child(content)
                .child(Scrollbar::vertical(&self.scroll_handle).mode(ScrollbarMode::Always)),
        )
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/view/ui/actions_list.rs"]
mod tests;
