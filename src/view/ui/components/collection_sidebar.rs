use super::EventHandler;
use super::collection_name_popover::{CollectionNameEvent, CollectionNamePopover};
use super::pending_style::{ButtonPalette, PendingButtonExt};
use crate::view::i18n::text;
use crate::view::ui::state::collection_name::{CollectionNameDraft, CollectionNameTarget};
use crate::{application::dto::CollectionDTO, view::ui::state::MacroFilter};
use gpui_kit::{
    base::{Disableable, Selectable},
    component::{ActiveTheme, Sizable, button::Button},
    *,
};

pub enum SidebarEvent {
    Settings,
    Select(MacroFilter),
    NameForm(CollectionNameEvent),
}

#[derive(IntoElement)]
pub struct CollectionSidebar {
    pub settings_open: bool,
    pub logo: &'static str,
    pub collections: Vec<CollectionDTO>,
    pub selected: Option<MacroFilter>,
    pub name_form: CollectionNameDraft,
    pub disabled: bool,
    pub navigation_disabled: bool,
    pub stable_pending: bool,
    pub on_action: EventHandler<SidebarEvent>,
}

impl RenderOnce for CollectionSidebar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut content = div()
            .flex()
            .flex_col()
            .gap_3()
            .w(px(250.))
            .flex_shrink_0()
            .p_4()
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .child(
                div()
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(img(self.logo).size(px(28.)))
                    .child(div().text_xl().child("mxchro")),
            );
        for (id, label, filter) in [
            ("all", text("sidebar.all_macros"), MacroFilter::All),
            (
                "uncategorized",
                text("common.no_collection"),
                MacroFilter::Uncategorized,
            ),
        ] {
            let selected = self.selected.as_ref() == Some(&filter);
            let on_action = self.on_action.clone();
            content = content.child(
                Button::new(id)
                    .label(label)
                    .selected(selected)
                    .disabled(self.navigation_disabled)
                    .stable_pending(
                        self.stable_pending,
                        if selected {
                            ButtonPalette::Selected
                        } else {
                            ButtonPalette::Default
                        },
                        cx,
                    )
                    .on_click(move |_, window, cx| {
                        on_action(&SidebarEvent::Select(filter.clone()), window, cx)
                    }),
            );
        }
        content = content.child(div().mt_4().text_sm().child(text("sidebar.collections")));
        for collection in self.collections {
            let filter = MacroFilter::Collection(collection.id.clone());
            let selected = self.selected.as_ref() == Some(&filter);
            let on_action = self.on_action.clone();
            content = content.child(
                Button::new(SharedString::from(format!("collection-{}", collection.id)))
                    .label(format!(
                        "{}  ·  {}",
                        collection.name, collection.macros_count
                    ))
                    .selected(selected)
                    .disabled(self.navigation_disabled)
                    .stable_pending(
                        self.stable_pending,
                        if selected {
                            ButtonPalette::Selected
                        } else {
                            ButtonPalette::Default
                        },
                        cx,
                    )
                    .on_click(move |_, window, cx| {
                        on_action(&SidebarEvent::Select(filter.clone()), window, cx)
                    }),
            );
        }
        let create = self.on_action.clone();
        content
            .child(div().flex_1())
            .child(CollectionNamePopover {
                target: CollectionNameTarget::Create,
                draft: self.name_form,
                disabled: self.disabled,
                stable_pending: self.stable_pending,
                on_action: std::rc::Rc::new(move |event, window, cx| {
                    create(&SidebarEvent::NameForm(event.clone()), window, cx)
                }),
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("rxchor · Mxchro"),
                    )
                    .child(
                        Button::new("settings")
                            .small()
                            .label(text("settings.title"))
                            .selected(self.settings_open)
                            .disabled(self.navigation_disabled)
                            .stable_pending(
                                self.stable_pending,
                                if self.settings_open {
                                    ButtonPalette::Selected
                                } else {
                                    ButtonPalette::Default
                                },
                                cx,
                            )
                            .on_click(move |_, window, cx| {
                                (self.on_action)(&SidebarEvent::Settings, window, cx)
                            }),
                    ),
            )
    }
}
