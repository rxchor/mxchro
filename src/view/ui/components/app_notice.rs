use gpui_kit::{component::ActiveTheme, *};

pub enum NoticeKind {
    Error,
    Empty,
}

#[derive(IntoElement)]
pub struct AppNotice {
    message: SharedString,
    kind: NoticeKind,
}

impl AppNotice {
    pub fn new(message: impl Into<SharedString>, kind: NoticeKind) -> Self {
        Self {
            message: message.into(),
            kind,
        }
    }
}

impl RenderOnce for AppNotice {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = div().rounded_md().border_1().child(self.message);
        match self.kind {
            NoticeKind::Error => base.p_3().border_color(cx.theme().danger),
            NoticeKind::Empty => base.p_8().min_h(px(250.)).border_color(cx.theme().border),
        }
    }
}
