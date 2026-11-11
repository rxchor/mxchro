use super::*;
use crate::{
    application::ports::{CaptureError, CaptureOutcome, CaptureSession, KeyCaptureBackend},
    view::ui::{components::app_select::AppSelectState, state::field_errors::EditorField},
};
use std::{sync::Arc, time::Instant};

#[derive(Clone)]
pub(super) enum CaptureTarget {
    Editor(EditorField),
    Collection(String),
}

pub(super) struct ActiveCapture {
    session: CaptureSession,
    target: CaptureTarget,
    started: Instant,
}

impl MxchroApp {
    /// Bootstrap supplies an adapter already connected to the existing hooks.
    pub fn with_key_capture(mut self, backend: Arc<dyn KeyCaptureBackend>) -> Self {
        self.key_capture = Some(backend);
        self
    }

    fn capture_select(&self, target: &CaptureTarget, cx: &App) -> Option<Entity<AppSelectState>> {
        if self.pending || self.settings_open || !self.ready || self.confirmation.is_some() {
            return None;
        }
        match target {
            CaptureTarget::Editor(field) => {
                let editor = self.editor.as_ref()?;
                match field {
                    EditorField::Trigger => Some(editor.trigger.clone()),
                    EditorField::ActionKey => Some(editor.action_key.clone()),
                    EditorField::Hotkey
                        if editor.collection.read(cx).selected_value().is_none() =>
                    {
                        Some(editor.hotkey.clone())
                    }
                    _ => None,
                }
            }
            CaptureTarget::Collection(id) => {
                if self.editor.is_some()
                    || self.filter.collection_id() != Some(id.as_str())
                    || !self
                        .data
                        .collections
                        .iter()
                        .any(|collection| &collection.id == id)
                {
                    return None;
                }
                self.collection_hotkey
                    .as_ref()
                    .filter(|draft| &draft.id == id)
                    .map(|draft| draft.select.clone())
            }
        }
    }

    pub(super) fn start_capture(&mut self, target: CaptureTarget, cx: &mut Context<Self>) {
        if self.capture.is_some() || self.capture_select(&target, cx).is_none() {
            return;
        }
        let Some(backend) = &self.key_capture else {
            return;
        };
        match backend.start() {
            Ok(session) => {
                self.collection_name.close();
                self.capture = Some(ActiveCapture {
                    session,
                    target,
                    started: Instant::now(),
                });
                self.sync_actions_disabled(cx);
            }
            Err(error) => self.notification_errors.push(text(match error {
                CaptureError::Unavailable => "capture.unavailable",
                CaptureError::Busy => "capture.busy",
            })),
        }
        cx.notify();
    }

    pub(super) fn capture_session(&self) -> Option<CaptureSession> {
        self.capture.as_ref().map(|active| active.session)
    }

    pub(super) fn cancel_capture(&mut self, cx: &mut App) {
        if let Some(active) = self.capture.take() {
            if let Some(backend) = &self.key_capture {
                backend.cancel(active.session);
            }
            self.sync_actions_disabled(cx);
        }
    }

    pub(super) fn poll_capture(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(active) = &self.capture else {
            return;
        };
        let target = active.target.clone();
        let timed_out = active.started.elapsed() >= Duration::from_secs(10);
        if !window.is_window_active() || timed_out || self.capture_select(&target, cx).is_none() {
            self.cancel_capture(cx);
            if timed_out {
                self.notification_errors.push(text("capture.timeout"));
            }
            cx.notify();
            return;
        }
        let Some(outcome) = self
            .key_capture
            .as_ref()
            .and_then(|backend| backend.poll(active.session))
        else {
            return;
        };
        self.cancel_capture(cx);
        if let CaptureOutcome::Key { vk, key } = outcome {
            match key.filter(|key| self.data.keys.contains(key)) {
                Some(key) => {
                    if let Some(select) = self.capture_select(&target, cx) {
                        select.update(cx, |select, cx| select.set_selected_value(&key, window, cx));
                        match target {
                            CaptureTarget::Editor(field) => {
                                if let Some(editor) = &mut self.editor {
                                    editor.errors.set(field, None);
                                }
                            }
                            // Programmatic select updates are deliberately silent;
                            // collection capture must send exactly one autosave.
                            CaptureTarget::Collection(id) => {
                                self.send(Command::SetCollectionHotkey(id, Some(key)), cx)
                            }
                        }
                    }
                }
                None => self
                    .notification_errors
                    .push(rust_i18n::t!("capture.unsupported", vk = vk).into_owned()),
            }
        }
        cx.notify();
    }
}

impl Drop for MxchroApp {
    fn drop(&mut self) {
        if let Some(active) = &self.capture
            && let Some(backend) = &self.key_capture
        {
            backend.cancel(active.session);
        }
    }
}
