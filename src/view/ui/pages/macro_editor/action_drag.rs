//! Pointer-only drag handle and short-lived preview for the virtualized sequence.
use super::actions_list::action_content;
use super::layout::{ACTION_ROW_HEIGHT, ACTION_ROW_STRIDE};
use crate::{
    domain::types::Action,
    view::{i18n::text, ui::components::EventHandler},
};
use gpui_kit::{
    assets::IconName,
    base::Disableable,
    component::{ActiveTheme, Sizable, button::Button},
    prelude::FluentBuilder,
    *,
};

const EDGE_ZONE: f32 = 36.;

#[derive(Clone)]
pub(super) struct ActionDrag {
    pub list: EntityId,
    pub revision: u64,
    pub from: usize,
    pub action: Action,
}

#[derive(IntoElement)]
pub(super) struct ActionDragHandle {
    pub drag: ActionDrag,
    pub disabled: bool,
    pub on_start: EventHandler<ActionDrag>,
}

impl RenderOnce for ActionDragHandle {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .id(("action-drag", self.drag.from))
            .flex_shrink_0()
            .when(!self.disabled, |handle| {
                handle.cursor(CursorStyle::OpenHand).on_drag(
                    self.drag,
                    move |drag, _, window, cx| {
                        (self.on_start)(drag, window, cx);
                        cx.stop_propagation();
                        cx.new(|_| ActionDragPreview { drag: drag.clone() })
                    },
                )
            })
            .child(
                Button::new("handle")
                    .small()
                    .icon(IconName::GripVertical)
                    .tooltip(text("editor.drag_action"))
                    .accessibility_label(text("editor.drag_action"))
                    .disabled(self.disabled),
            )
    }
}

struct ActionDragPreview {
    drag: ActionDrag,
}

impl Render for ActionDragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(300.))
            .h(px(ACTION_ROW_HEIGHT))
            .p_2()
            .flex()
            .items_center()
            .bg(cx.theme().sidebar)
            .border_1()
            .border_color(cx.theme().primary)
            .rounded_md()
            .opacity(0.9)
            .child(action_content(&self.drag.action, cx))
    }
}

/// Nearest insertion boundary, not an index into the visible rows.
pub(super) fn insertion_slot(content_y: f32, count: usize) -> usize {
    ((content_y / ACTION_ROW_STRIDE + 0.5).floor().max(0.) as usize).min(count)
}

/// Convert a boundary in the original sequence into the final moved index.
pub(super) fn move_destination(from: usize, slot: usize, count: usize) -> Option<usize> {
    if from >= count || slot > count {
        return None;
    }
    let to = slot.saturating_sub(usize::from(from < slot));
    (to != from).then_some(to)
}

pub(super) fn edge_scroll_step(y: f32, height: f32) -> f32 {
    if y < 0. || y > height {
        return 0.;
    }
    if y < EDGE_ZONE {
        10. * (1. - y / EDGE_ZONE)
    } else if y > height - EDGE_ZONE {
        -10. * (1. - (height - y) / EDGE_ZONE)
    } else {
        0.
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/view/ui/action_drag.rs"]
mod tests;
