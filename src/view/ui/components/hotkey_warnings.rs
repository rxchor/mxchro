use crate::{
    application::dto::{CollectionDTO, HotkeyIssueDTO, HotkeyIssueKind, HotkeyOwnerDTO},
    view::i18n::text,
};
use gpui_kit::{base::StyledExt, component::ActiveTheme, *};

#[derive(IntoElement)]
pub struct HotkeyWarnings {
    pub issues: Vec<HotkeyIssueDTO>,
    pub collections: Vec<CollectionDTO>,
}

impl RenderOnce for HotkeyWarnings {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .id("hotkey-warnings")
            .role(Role::Alert)
            .flex_shrink_0()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().warning)
            .flex()
            .flex_col()
            .gap_2()
            .child(div().font_semibold().child(text("hotkey_issue.title")))
            .child(div().text_sm().child(text("hotkey_issue.help")))
            .child(
                div()
                    .id("hotkey-warning-items")
                    .max_h(px(140.))
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(self.issues.into_iter().map(|issue| {
                        let owner = match &issue.owner {
                            HotkeyOwnerDTO::Macro(key) => {
                                rust_i18n::t!("hotkey_issue.macro", name = key).into_owned()
                            }
                            HotkeyOwnerDTO::Collection(id) => {
                                let name = self
                                    .collections
                                    .iter()
                                    .find(|collection| &collection.id == id)
                                    .map_or(id.as_str(), |collection| collection.name.as_str());
                                rust_i18n::t!("hotkey_issue.collection", name = name).into_owned()
                            }
                        };
                        let key = match issue.kind {
                            HotkeyIssueKind::GroupedMacro => "hotkey_issue.grouped",
                            HotkeyIssueKind::Duplicate => "hotkey_issue.duplicate",
                            HotkeyIssueKind::InvalidKey => "hotkey_issue.invalid",
                        };
                        div().text_sm().child(
                            rust_i18n::t!(key, owner = owner, key = issue.hotkey).into_owned(),
                        )
                    })),
            )
    }
}
