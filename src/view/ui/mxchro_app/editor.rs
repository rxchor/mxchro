use super::*;
use crate::view::ui::state::field_errors::EditorField;

impl MxchroApp {
    pub(super) fn close_editor(&mut self) {
        self.editor = None;
        self.editor_field_subscriptions.clear();
    }

    pub(super) fn sync_actions_disabled(&self, cx: &mut App) {
        if let Some(editor) = &self.editor {
            editor.actions_list.update(cx, |list, cx| {
                list.set_disabled(
                    self.pending || self.capture.is_some() || self.confirmation.is_some(),
                    cx,
                )
            });
        }
    }

    pub(super) fn open_editor(
        &mut self,
        config: Option<MacroEditDTO>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cancel_capture(cx);
        self.collection_name.close();
        let editor = MacroDraft::new(
            config,
            self.filter.collection_id().map(str::to_owned),
            &self.data.keys,
            &self.data.collections,
            window,
            cx,
        );
        cx.subscribe_in(
            &editor.actions_list,
            window,
            |this, _, event, window, cx| {
                this.handle_editor(&EditorEvent::Actions(event.clone()), window, cx);
            },
        )
        .detach();
        self.editor_field_subscriptions.clear();
        for (select, field) in [
            (&editor.trigger, EditorField::Trigger),
            (&editor.mode, EditorField::Mode),
            (&editor.collection, EditorField::Collection),
            (&editor.hotkey, EditorField::Hotkey),
            (&editor.action_key, EditorField::ActionKey),
        ] {
            self.editor_field_subscriptions.push(cx.subscribe(
                select,
                move |this, _, event, cx| {
                    let SelectEvent::Confirm(_) = event;
                    if let Some(editor) = &mut this.editor {
                        editor.errors.set(field, None);
                        cx.notify();
                    }
                },
            ));
        }
        self.editor_field_subscriptions
            .push(cx.subscribe(&editor.wait, |this, _, event, cx| {
                if matches!(event, gpui_kit::component::input::InputEvent::Change)
                    && let Some(editor) = &mut this.editor
                {
                    editor.errors.set(EditorField::Wait, None);
                    cx.notify();
                }
            }));
        self.editor = Some(editor);
        cx.notify();
    }

    pub(super) fn handle_editor(
        &mut self,
        event: &EditorEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.pending || self.capture.is_some() || self.confirmation.is_some() {
            return;
        }
        match event {
            EditorEvent::Form(MacroFormEvent::Capture(field)) => {
                self.start_capture(CaptureTarget::Editor(*field), cx);
                return;
            }
            EditorEvent::AddAction(ActionFormEvent::CaptureKey) => {
                self.start_capture(CaptureTarget::Editor(EditorField::ActionKey), cx);
                return;
            }
            _ => {}
        }
        let Some(editor) = self.editor.as_mut() else {
            return;
        };
        let actions_changed = matches!(event, EditorEvent::AddAction(_) | EditorEvent::Actions(_));
        match event {
            EditorEvent::Form(MacroFormEvent::Capture(_))
            | EditorEvent::AddAction(ActionFormEvent::CaptureKey) => unreachable!(),
            EditorEvent::Form(MacroFormEvent::SetOption(option, checked)) => {
                option.set(&mut editor.config, *checked)
            }
            EditorEvent::Form(MacroFormEvent::ClearHotkey) => {
                editor
                    .hotkey
                    .update(cx, |select, cx| select.set_selected_index(None, window, cx));
                editor.errors.set(EditorField::Hotkey, None);
            }
            EditorEvent::AddAction(ActionFormEvent::AddKey(press)) => {
                let error = editor.add_key(*press, &self.data.keys, cx).err();
                editor.errors.set(EditorField::ActionKey, error);
            }
            EditorEvent::AddAction(ActionFormEvent::AddWait) => {
                let error = editor.add_wait(cx).err();
                editor.errors.set(EditorField::Wait, error);
            }
            EditorEvent::Actions(ActionsListEvent::Move { from, to }) => {
                editor.move_action(*from, *to);
            }
            EditorEvent::Actions(ActionsListEvent::Remove(index)) => {
                if *index < editor.config.actions.len() {
                    editor.config.actions.remove(*index);
                }
            }
            EditorEvent::Save => {
                editor.errors.clear();
                match editor.read_config(cx) {
                    Ok(config) => {
                        let command = Command::SaveMacro {
                            old_key: editor.old_key.clone(),
                            config,
                        };
                        self.send(command, cx);
                    }
                    Err(error) => editor.errors.set(EditorField::Mode, Some(error)),
                }
            }
            EditorEvent::Cancel => {
                self.close_editor();
            }
        }
        if actions_changed {
            self.sync_editor_actions(cx);
        }
        cx.notify();
    }

    fn sync_editor_actions(&mut self, cx: &mut Context<Self>) {
        if let Some(editor) = &self.editor {
            editor.actions_list.update(cx, |list, cx| {
                list.replace_actions(editor.config.actions.clone(), cx)
            });
        }
        if let Some(editor) = &mut self.editor
            && !editor.config.actions.is_empty()
        {
            editor.errors.set(EditorField::Actions, None);
        }
    }
}
