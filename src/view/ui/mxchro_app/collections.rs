use super::*;

impl MxchroApp {
    pub(super) fn handle_collection_name(
        &mut self,
        event: &CollectionNameEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.pending || !self.ready || self.capture.is_some() || self.confirmation.is_some() {
            // Escape can request dismissal while a save is in flight. Keep the
            // controlled form and its target locked until acknowledgement.
            cx.notify();
            return;
        }
        match event {
            CollectionNameEvent::Open(target) => {
                let value = match target {
                    CollectionNameTarget::Create => String::new(),
                    CollectionNameTarget::Rename(id) => {
                        if self.settings_open
                            || self.editor.is_some()
                            || self.filter.collection_id() != Some(id.as_str())
                        {
                            return;
                        }
                        let Some(collection) = self.data.collections.iter().find(|c| &c.id == id)
                        else {
                            return;
                        };
                        collection.name.clone()
                    }
                };
                self.collection_name
                    .input
                    .update(cx, |input, cx| input.set_value(value, window, cx));
                self.collection_name.error = None;
                self.collection_name.target = Some(target.clone());
            }
            CollectionNameEvent::Close(target) => {
                if self.collection_name.target.as_ref() != Some(target) {
                    return;
                }
                self.collection_name.close();
            }
            CollectionNameEvent::Submit(target) => {
                if self.collection_name.target.as_ref() != Some(target) {
                    return;
                }
                let name = self.collection_name.input.read(cx).value().to_string();
                let command = match target {
                    CollectionNameTarget::Create => Command::CreateCollection(name),
                    CollectionNameTarget::Rename(id) => {
                        if self.filter.collection_id() != Some(id.as_str())
                            || !self.data.collections.iter().any(|c| &c.id == id)
                        {
                            self.collection_name.close();
                            cx.notify();
                            return;
                        }
                        Command::RenameCollection(id.clone(), name)
                    }
                };
                self.send(command, cx);
            }
        }
        cx.notify();
    }

    pub(super) fn reconcile_collection_name(&mut self) {
        if let Some(CollectionNameTarget::Rename(id)) = &self.collection_name.target
            && (self.settings_open
                || self.editor.is_some()
                || self.filter.collection_id() != Some(id.as_str())
                || !self.data.collections.iter().any(|c| &c.id == id))
        {
            self.collection_name.close();
        }
    }

    pub(super) fn sync_collection_selector(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let collection = self
            .data
            .collections
            .iter()
            .find(|c| Some(c.id.as_str()) == self.filter.collection_id());
        let Some(collection) = collection else {
            self.collection_hotkey = None;
            self.collection_hotkey_subscription = None;
            return;
        };
        if self.collection_hotkey.as_ref().map(|draft| &draft.id) != Some(&collection.id) {
            let draft = CollectionHotkeyDraft::new(collection, &self.data.keys, window, cx);
            let id = draft.id.clone();
            self.collection_hotkey_subscription =
                Some(cx.subscribe(&draft.select, move |this, _, event, cx| {
                    let SelectEvent::Confirm(value) = event;
                    // The callback belongs to this ID, never the current filter implicitly.
                    if this.pending || this.filter.collection_id() != Some(id.as_str()) {
                        this.sync_collection_hotkey = true;
                        cx.notify();
                        return;
                    }
                    this.send(Command::SetCollectionHotkey(id.clone(), value.clone()), cx);
                }));
            self.collection_hotkey = Some(draft);
        } else if self.sync_collection_hotkey
            && let Some(draft) = &mut self.collection_hotkey
        {
            draft.sync(collection, &self.data.keys, window, cx);
        }
        self.sync_collection_hotkey = false;
    }
}
