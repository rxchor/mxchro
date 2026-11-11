use gpui_kit::{
    base::Disableable,
    component::{ActiveTheme, checkbox::Checkbox},
    *,
};

/// Application checkbox with consistent spacing and the accessible GPUI-KIT
/// control (keyboard focus, label and checked state), not a simulated button.
/// The card is the checkbox itself, so its entire padded area is interactive.
#[derive(IntoElement)]
pub struct AppCheckBox {
    control: Checkbox,
}

impl AppCheckBox {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            control: Checkbox::new(id).label(label.into()),
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.control = self.control.checked(checked);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.control = self.control.disabled(disabled);
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.control = self.control.on_change(handler);
        self
    }
}

impl RenderOnce for AppCheckBox {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.control
            .w_full()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
    }
}
