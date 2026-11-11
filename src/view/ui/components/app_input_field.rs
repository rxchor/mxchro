use gpui_kit::{
    component::input::{Input, InputState},
    *,
};

pub fn text_input(
    value: &str,
    placeholder: &str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<InputState> {
    cx.new(|cx| {
        let mut state = InputState::new(window, cx).placeholder(placeholder.to_owned());
        state.set_value(value.to_owned(), window, cx);
        state
    })
}

#[derive(IntoElement)]
pub struct AppInputField {
    label: SharedString,
    input: Entity<InputState>,
    disabled: bool,
    error: Option<String>,
}

impl AppInputField {
    pub fn new(label: impl Into<SharedString>, input: &Entity<InputState>) -> Self {
        Self {
            label: label.into(),
            input: input.clone(),
            disabled: false,
            error: None,
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
}

impl RenderOnce for AppInputField {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.label)
            .child(Input::new(&self.input).disabled(self.disabled))
            .children(
                self.error
                    .map(|message| super::field_error::field_error(message, cx)),
            )
    }
}
