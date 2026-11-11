use crate::view::i18n::text;
use crate::view::ui::{
    components::{
        collection_name_popover::CollectionNameEvent,
        collection_sidebar::{CollectionSidebar, SidebarEvent},
        collection_toolbar::CollectionEvent,
        hotkey_warnings::HotkeyWarnings,
        macro_card::MacroCardEvent,
    },
    pages::{
        home_page::{HomeEvent, HomePage},
        macro_editor::{
            EditorEvent, MacroEditorPage, action_form::ActionFormEvent,
            actions_list::ActionsListEvent, macro_form::MacroFormEvent,
        },
        settings_page::SettingsPage,
    },
    state::{
        MacroDraft, MacroFilter,
        collection_hotkey::CollectionHotkeyDraft,
        collection_name::{CollectionNameDraft, CollectionNameTarget},
    },
};
use crate::{
    application::{
        commands::Command,
        dto::{AppSettingsDTO, MacroEditDTO},
        ports::{CaptureSession, KeyCaptureBackend, WindowIconBackend},
        queries::{Query, Snapshot},
        request::Request,
    },
    view::backend::{CommandBus, error as backend_error, settings::SettingsBus},
};
use gpui_kit::{
    component::{ActiveTheme, WindowExt, input::InputEvent, select::SelectEvent},
    *,
};
use std::{
    rc::Rc,
    sync::{Arc, mpsc::TryRecvError},
    time::Duration,
};
mod backend;
mod capture;
mod collections;
mod confirmation;
mod editor;
mod feedback;
mod home;
mod settings;
mod window_icon;
use capture::{ActiveCapture, CaptureTarget};
use confirmation::{DeleteTarget, PendingConfirmation};
use feedback::PendingOperation;

/// Owns application state and backend communication. Pages and components only
/// render controlled data and emit typed events back to this entity.
pub struct MxchroApp {
    command_bus: CommandBus,
    settings_bus: SettingsBus,
    settings: AppSettingsDTO,
    settings_open: bool,
    settings_pending: bool,
    settings_error: Option<String>,
    window_icon: Option<Box<dyn WindowIconBackend>>,
    window_icon_path: Option<&'static str>,
    key_capture: Option<Arc<dyn KeyCaptureBackend>>,
    capture: Option<ActiveCapture>,
    capture_notification: Option<CaptureSession>,
    data: Snapshot,
    filter: MacroFilter,
    editor: Option<MacroDraft>,
    collection_name: CollectionNameDraft,
    _collection_name_subscription: Subscription,
    editor_field_subscriptions: Vec<Subscription>,
    collection_hotkey: Option<CollectionHotkeyDraft>,
    collection_hotkey_subscription: Option<Subscription>,
    sync_collection_hotkey: bool,
    notification_errors: Vec<String>,
    pending_operation: Option<PendingOperation>,
    backend_stopped_reported: bool,
    pending: bool,
    toggle_pending: bool,
    ready: bool,
    confirmation: Option<PendingConfirmation>,
    confirmation_id: u64,
    _poll: Task<()>,
}

impl MxchroApp {
    pub fn new(
        command_bus: CommandBus,
        settings_bus: SettingsBus,
        window_icon: Option<Box<dyn WindowIconBackend>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let settings = settings_bus.initial;
        crate::view::theme::apply(settings.theme, cx);
        let settings_error = settings_bus.load_error.map(settings::settings_error);
        cx.observe_window_activation(window, |this, window, cx| {
            if !window.is_window_active() && this.capture.is_some() {
                this.cancel_capture(cx);
                cx.notify();
            }
        })
        .detach();
        let collection_name = CollectionNameDraft::new(window, cx);
        let collection_name_subscription = cx.subscribe_in(
            &collection_name.input,
            window,
            |this: &mut Self, _, event, window, cx| match event {
                InputEvent::Change => {
                    this.collection_name.error = None;
                    cx.notify();
                }
                InputEvent::PressEnter {
                    secondary: false,
                    shift: false,
                } => {
                    if let Some(target) = this.collection_name.target.clone() {
                        this.handle_collection_name(
                            &CollectionNameEvent::Submit(target),
                            window,
                            cx,
                        );
                    }
                }
                _ => {}
            },
        );
        let poll = cx.spawn_in(window, async |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                if this
                    .update_in(cx, |this, window, cx| this.poll(window, cx))
                    .is_err()
                {
                    break;
                }
            }
        });
        let mut app = Self {
            command_bus,
            settings_bus,
            settings,
            settings_open: false,
            settings_pending: false,
            settings_error: settings_error.clone(),
            window_icon,
            window_icon_path: None,
            key_capture: None,
            capture: None,
            capture_notification: None,
            data: Snapshot::default(),
            filter: MacroFilter::All,
            editor: None,
            collection_name,
            _collection_name_subscription: collection_name_subscription,
            editor_field_subscriptions: Vec::new(),
            collection_hotkey: None,
            collection_hotkey_subscription: None,
            sync_collection_hotkey: true,
            notification_errors: settings_error.into_iter().collect(),
            pending_operation: None,
            backend_stopped_reported: false,
            pending: false,
            toggle_pending: false,
            ready: false,
            confirmation: None,
            confirmation_id: 0,
            _poll: poll,
        };
        app.sync_window_icon();
        app
    }
}

impl Render for MxchroApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_collection_selector(window, cx);
        if let Some(draft) = &mut self.editor {
            draft.sync_collections(&self.data.collections, window, cx);
        }
        let sidebar = CollectionSidebar {
            collections: self.data.collections.clone(),
            selected: (!self.settings_open && self.editor.is_none()).then(|| self.filter.clone()),
            name_form: self.collection_name.clone(),
            settings_open: self.settings_open,
            logo: crate::view::assets::logo_path(self.settings.logo, self.settings.theme),
            disabled: self.pending
                || !self.ready
                || self.capture.is_some()
                || self.confirmation.is_some(),
            navigation_disabled: self.pending || !self.ready || self.confirmation.is_some(),
            stable_pending: self.toggle_pending,
            on_action: Rc::new(
                cx.listener(|this, event, window, cx| this.handle_sidebar(event, window, cx)),
            ),
        };
        let mut main = div()
            .id("main")
            .flex_1()
            .min_w_0()
            .min_h_0()
            .h_full()
            .overflow_y_scroll()
            .p_6()
            .flex()
            .flex_col()
            .gap_4();
        if !self.data.hotkey_issues.is_empty() {
            main = main.child(HotkeyWarnings {
                issues: self.data.hotkey_issues.clone(),
                collections: self.data.collections.clone(),
            });
        }
        if self.settings_open {
            main = main.child(SettingsPage {
                settings: self.settings,
                pending: self.settings_pending,
                error: self.settings_error.clone(),
                on_action: Rc::new(
                    cx.listener(|this, event, _, cx| this.handle_settings(event, cx)),
                ),
            });
        } else if let Some(draft) = &self.editor {
            main = main.child(MacroEditorPage {
                draft: draft.clone(),
                pending: self.pending,
                interaction_locked: self.capture.is_some() || self.confirmation.is_some(),
                capture_enabled: self.key_capture.is_some(),
                on_action: Rc::new(
                    cx.listener(|this, event, window, cx| this.handle_editor(event, window, cx)),
                ),
            });
        } else {
            main = main.child(HomePage {
                hotkey_issues: self.data.hotkey_issues.clone(),
                collection_name_form: self.collection_name.clone(),
                capture_enabled: self.key_capture.is_some(),
                collection_hotkey: self
                    .collection_hotkey
                    .as_ref()
                    .map(|draft| draft.select.clone()),
                macros: self.data.macros.clone(),
                collections: self.data.collections.clone(),
                filter: self.filter.clone(),
                pending: self.pending || self.capture.is_some() || self.confirmation.is_some(),
                ready: self.ready,
                stable_pending: self.toggle_pending,
                on_action: Rc::new(
                    cx.listener(|this, event, window, cx| this.handle_home(event, window, cx)),
                ),
            });
        }
        div()
            .size_full()
            .flex()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .text_sm()
            .child(sidebar)
            .child(main)
    }
}
