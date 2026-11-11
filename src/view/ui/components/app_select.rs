use super::EventHandler;
use crate::view::i18n::text;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    base::Disableable,
    component::{
        ActiveTheme, Icon, IconName,
        button::Button,
        select::{SearchableVec, Select, SelectItem, SelectState},
    },
    *,
};

/// Display text is independent from the exact value understood by the domain.
#[derive(Clone, PartialEq, Eq)]
pub struct SelectOption {
    pub value: String,
    pub label: SharedString,
}

impl SelectItem for SelectOption {
    type Value = String;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &String {
        &self.value
    }
}

pub type AppSelectState = SelectState<SearchableVec<SelectOption>>;

pub fn select_state(
    options: Vec<SelectOption>,
    value: Option<&str>,
    searchable: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<AppSelectState> {
    let selected = value
        .and_then(|value| options.iter().position(|option| option.value == value))
        .map(|row| gpui_kit::component::IndexPath::default().row(row));
    cx.new(|cx| {
        SelectState::new(SearchableVec::new(options), selected, window, cx).searchable(searchable)
    })
}

/// Styled facade over the native select; its entity is retained by the form owner,
/// just like InputState. Searching never changes the committed value.
#[derive(IntoElement)]
pub struct AppSelect {
    label: SharedString,
    state: Entity<AppSelectState>,
    disabled: bool,
    stable_pending: bool,
    optional: bool,
    placeholder: Option<SharedString>,
    error: Option<String>,
    on_capture: Option<EventHandler<()>>,
}

impl AppSelect {
    pub fn new(label: impl Into<SharedString>, state: &Entity<AppSelectState>) -> Self {
        Self {
            label: label.into(),
            state: state.clone(),
            disabled: false,
            stable_pending: false,
            optional: false,
            placeholder: None,
            error: None,
            on_capture: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn error(mut self, error: Option<String>) -> Self {
        self.error = error;
        self
    }

    pub fn optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// Retain the normal input colors during a short request without enabling it.
    pub fn stable_pending(mut self, stable: bool) -> Self {
        self.stable_pending = stable;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Optional input-capture action; the retained selector remains the editor.
    pub fn on_capture(mut self, on_capture: EventHandler<()>) -> Self {
        self.on_capture = Some(on_capture);
        self
    }
}

impl RenderOnce for AppSelect {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .child(self.label.clone())
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div().flex_1().min_w_0().child(
                            Select::new(&self.state)
                                .w_full()
                                .icon(
                                    Icon::new(IconName::ChevronDown)
                                        .text_color(cx.theme().muted_foreground),
                                )
                                .accessibility_label(self.label.clone())
                                .placeholder(self.placeholder.unwrap_or_else(|| {
                                    if self.optional {
                                        text("select.unassigned")
                                    } else {
                                        text("select.placeholder")
                                    }
                                    .into()
                                }))
                                .search_placeholder(text("select.search"))
                                .cleanable(true)
                                .disabled(self.disabled)
                                .when(self.stable_pending, |select| {
                                    select
                                        .opacity(1.)
                                        .bg(cx.theme().input_background())
                                        .text_color(cx.theme().foreground)
                                })
                                .empty(|_, cx| {
                                    div()
                                        .p_4()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(text("select.empty"))
                                }),
                        ),
                    )
                    .children(self.on_capture.map(|on_capture| {
                        let description =
                            rust_i18n::t!("capture.for_field", field = self.label).into_owned();
                        Button::new(("capture-input", self.state.entity_id()))
                            .icon(gpui_kit::assets::IconName::CircleDot)
                            .w_8()
                            .h_8()
                            .flex_shrink_0()
                            .accessibility_label(description.clone())
                            .tooltip(description)
                            .disabled(self.disabled)
                            .on_click(move |_, window, cx| on_capture(&(), window, cx))
                    })),
            )
            .children(
                self.error
                    .map(|message| super::field_error::field_error(message, cx)),
            )
            .text_color(cx.theme().foreground)
    }
}
