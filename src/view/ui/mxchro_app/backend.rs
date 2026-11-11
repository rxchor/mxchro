use super::*;

impl MxchroApp {
    pub(super) fn poll(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        loop {
            match self.command_bus.updates.try_recv() {
                Ok(update) => {
                    self.data = update.snapshot;
                    self.sync_collection_hotkey = true;
                    self.ready = true;
                    let operation = if update.request_completed {
                        self.pending = false;
                        self.toggle_pending = false;
                        self.pending_operation.take()
                    } else {
                        None
                    };
                    if let Some(error) = update.error {
                        self.report_error(operation, error);
                    } else if let Some(operation) = operation {
                        if matches!(
                            operation,
                            PendingOperation::CreateCollection | PendingOperation::RenameCollection
                        ) {
                            self.collection_name.close();
                        }
                        // Save success requires the explicit accepted-save outcome.
                        if operation != PendingOperation::SaveMacro || update.saved {
                            Self::notify_success(operation, window, cx);
                        }
                    }
                    if update.saved {
                        self.editor = None;
                        self.editor_field_subscriptions.clear();
                    }
                    if let Some(editor) = update.editor {
                        self.open_editor(Some(editor), window, cx);
                    }
                    if let MacroFilter::Collection(id) = &self.filter
                        && !self.data.collections.iter().any(|c| &c.id == id)
                    {
                        self.filter = MacroFilter::All;
                    }
                    self.reconcile_collection_name();
                    self.sync_actions_disabled(cx);
                    cx.notify();
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if !self.backend_stopped_reported {
                        self.backend_stopped_reported = true;
                        self.notification_errors.push(text("error.backend_stopped"));
                        self.pending_operation = None;
                        self.pending = true;
                        self.toggle_pending = false;
                        self.sync_collection_hotkey = true;
                        self.sync_actions_disabled(cx);
                        cx.notify();
                    }
                    break;
                }
            }
        }
        self.poll_settings(cx);
        self.poll_capture(window, cx);
        self.reconcile_confirmation(window, cx);
        self.sync_capture_notification(window, cx);
        self.flush_notification_errors(window, cx);
    }

    pub(super) fn send(&mut self, command: Command, cx: &mut Context<Self>) {
        self.request(command.into(), cx);
    }

    pub(super) fn request(&mut self, request: Request, cx: &mut Context<Self>) {
        if self.pending || self.confirmation.is_some() {
            return;
        }
        self.cancel_capture(cx);
        let operation = PendingOperation::for_request(&request);
        if matches!(
            operation,
            PendingOperation::CreateCollection | PendingOperation::RenameCollection
        ) {
            self.collection_name.error = None;
        }
        let toggle = matches!(
            request,
            Request::Command(Command::SetMacroEnabled(..) | Command::SetCollectionEnabled(..))
        );
        self.pending = self.command_bus.requests.send(request).is_ok();
        self.pending_operation = self.pending.then_some(operation);
        self.toggle_pending = self.pending && toggle;
        if !self.pending {
            self.notification_errors
                .push(text("error.backend_unavailable"));
            self.backend_stopped_reported = true;
            self.pending = true;
            self.sync_collection_hotkey = true;
        }
        self.sync_actions_disabled(cx);
        cx.notify();
    }
}
