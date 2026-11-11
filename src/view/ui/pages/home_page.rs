use crate::view::i18n::{macro_count, text};
use crate::{
    application::dto::{CollectionDTO, HotkeyIssueDTO, HotkeyOwnerDTO, MacroDTO},
    view::ui::{
        components::{
            AppNotice, EventHandler, NoticeKind,
            app_select::AppSelectState,
            collection_toolbar::{CollectionEvent, CollectionToolbar},
            macro_card::{MacroCard, MacroCardEvent},
            pending_style::{ButtonPalette, PendingButtonExt},
            toggle_hotkey::toggle_hotkey_tag,
        },
        state::MacroFilter,
        state::collection_name::CollectionNameDraft,
    },
};
use gpui_kit::{
    base::{Disableable, StyledExt},
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
    },
    *,
};
use std::{collections::HashMap, rc::Rc};

struct MacroGroup {
    collection_id: Option<String>,
    name: String,
    toggle_hotkey: Option<String>,
    macros: Vec<MacroDTO>,
}

/// Presentation only: preserve macro order and never hide orphaned assignments.
fn group_macros(macros: Vec<MacroDTO>, collections: &[CollectionDTO]) -> Vec<MacroGroup> {
    let mut collections: Vec<_> = collections.iter().collect();
    collections.sort_by_key(|collection| collection.order);
    let mut groups: Vec<_> = collections
        .into_iter()
        .map(|collection| MacroGroup {
            collection_id: Some(collection.id.clone()),
            name: collection.name.clone(),
            toggle_hotkey: collection.toggle_enabled_hotkey.clone(),
            macros: Vec::new(),
        })
        .collect();
    let indices: HashMap<_, _> = groups
        .iter()
        .enumerate()
        .map(|(index, group)| (group.collection_id.clone().unwrap(), index))
        .collect();
    let unassigned = groups.len();
    groups.push(MacroGroup {
        collection_id: None,
        name: text("common.no_collection"),
        toggle_hotkey: None,
        macros: Vec::new(),
    });
    for config in macros {
        let index = config
            .collection_id
            .as_ref()
            .and_then(|id| indices.get(id))
            .copied()
            .unwrap_or(unassigned);
        groups[index].macros.push(config);
    }
    groups.retain(|group| !group.macros.is_empty());
    groups
}

fn macro_card(
    config: MacroDTO,
    issues: &[HotkeyIssueDTO],
    disabled: bool,
    stable_pending: bool,
    on_action: EventHandler<HomeEvent>,
) -> MacroCard {
    let hotkey_inactive =
        has_hotkey_issue(issues, &HotkeyOwnerDTO::Macro(config.trigger_key.clone()));
    MacroCard {
        config,
        disabled,
        stable_pending,
        hotkey_inactive,
        on_action: Rc::new(move |event, window, cx| {
            on_action(&HomeEvent::Macro(event.clone()), window, cx)
        }),
    }
}

fn has_hotkey_issue(issues: &[HotkeyIssueDTO], owner: &HotkeyOwnerDTO) -> bool {
    issues.iter().any(|issue| &issue.owner == owner)
}

pub enum HomeEvent {
    NewMacro,
    Macro(MacroCardEvent),
    Collection(CollectionEvent),
}

#[derive(IntoElement)]
pub struct HomePage {
    pub collection_name_form: CollectionNameDraft,
    pub collection_hotkey: Option<Entity<AppSelectState>>,
    pub capture_enabled: bool,
    pub macros: Vec<MacroDTO>,
    pub collections: Vec<CollectionDTO>,
    pub hotkey_issues: Vec<HotkeyIssueDTO>,
    pub filter: MacroFilter,
    pub pending: bool,
    pub stable_pending: bool,
    pub ready: bool,
    pub on_action: EventHandler<HomeEvent>,
}

impl RenderOnce for HomePage {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let macros: Vec<_> = self
            .macros
            .into_iter()
            .filter(|m| self.filter.matches(m))
            .collect();
        let new_macro = self.on_action.clone();
        let selected_collection = self
            .collections
            .iter()
            .find(|c| Some(c.id.as_str()) == self.filter.collection_id());
        let selected_hotkey = selected_collection.and_then(|collection| {
            toggle_hotkey_tag(
                collection.toggle_enabled_hotkey.as_deref(),
                has_hotkey_issue(
                    &self.hotkey_issues,
                    &HotkeyOwnerDTO::Collection(collection.id.clone()),
                ),
            )
        });
        let mut content = div().flex().flex_col().gap_4().child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .child(div().text_2xl().child(self.filter.title(&self.collections)))
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .gap_2()
                                .child(macro_count(macros.len()))
                                .children(selected_hotkey),
                        ),
                )
                .child(
                    Button::new("new-macro")
                        .primary()
                        .label(text("home.new_macro"))
                        .disabled(self.pending || !self.ready)
                        .stable_pending(self.stable_pending, ButtonPalette::Primary, cx)
                        .on_click(move |_, window, cx| new_macro(&HomeEvent::NewMacro, window, cx)),
                ),
        );
        if let Some(collection) = selected_collection {
            let on_action = self.on_action.clone();
            content = content.child(CollectionToolbar {
                name_form: self.collection_name_form,
                collection: collection.clone(),
                hotkey: self.collection_hotkey.clone(),
                disabled: self.pending,
                stable_pending: self.stable_pending,
                capture_enabled: self.capture_enabled,
                on_action: Rc::new(move |event, window, cx| {
                    on_action(&HomeEvent::Collection(event.clone()), window, cx)
                }),
            });
        }
        if macros.is_empty() {
            content = content.child(AppNotice::new(
                if self.ready {
                    text("home.empty")
                } else {
                    text("home.loading")
                },
                NoticeKind::Empty,
            ));
        }
        if self.filter == MacroFilter::All {
            for group in group_macros(macros, &self.collections) {
                let mut container = div()
                    .id(SharedString::from(match &group.collection_id {
                        Some(id) => format!("collection-group-{id}"),
                        None => "unassigned-group".into(),
                    }))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().sidebar)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_3()
                            .pb_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .items_center()
                                    .gap_2()
                                    .min_w_0()
                                    .child(
                                        div().text_lg().font_semibold().child(group.name.clone()),
                                    )
                                    .children(toggle_hotkey_tag(
                                        group.toggle_hotkey.as_deref(),
                                        group.collection_id.as_ref().is_some_and(|id| {
                                            has_hotkey_issue(
                                                &self.hotkey_issues,
                                                &HotkeyOwnerDTO::Collection(id.clone()),
                                            )
                                        }),
                                    )),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .flex_shrink_0()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(macro_count(group.macros.len())),
                            ),
                    );
                for config in group.macros {
                    container = container.child(macro_card(
                        config,
                        &self.hotkey_issues,
                        self.pending,
                        self.stable_pending,
                        self.on_action.clone(),
                    ));
                }
                content = content.child(container);
            }
            return content;
        }
        for config in macros {
            content = content.child(macro_card(
                config,
                &self.hotkey_issues,
                self.pending,
                self.stable_pending,
                self.on_action.clone(),
            ));
        }
        content
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/view/ui/home_page.rs"]
mod tests;
