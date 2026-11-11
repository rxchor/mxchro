use super::*;

impl MxchroApp {
    pub(super) fn handle_sidebar(
        &mut self,
        event: &SidebarEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let SidebarEvent::NameForm(event) = event {
            self.handle_collection_name(event, window, cx);
            return;
        }
        if self.pending || !self.ready || self.confirmation.is_some() {
            return;
        }
        self.cancel_capture(cx);
        self.collection_name.close();
        match event {
            SidebarEvent::Settings => {
                self.settings_open = true;
                cx.notify();
            }
            SidebarEvent::Select(filter) => {
                self.close_editor();
                self.settings_open = false;
                self.filter = filter.clone();
                cx.notify();
            }
            SidebarEvent::NameForm(_) => unreachable!(),
        }
    }

    pub(super) fn handle_home(
        &mut self,
        event: &HomeEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let HomeEvent::Collection(CollectionEvent::NameForm(event)) = event {
            self.handle_collection_name(event, window, cx);
            return;
        }
        if self.pending || self.capture.is_some() || self.confirmation.is_some() {
            return;
        }
        match event {
            HomeEvent::Collection(CollectionEvent::CaptureHotkey(id)) => {
                self.start_capture(CaptureTarget::Collection(id.clone()), cx);
            }
            HomeEvent::NewMacro => self.open_editor(None, window, cx),
            HomeEvent::Macro(MacroCardEvent::Edit(key)) => {
                self.request(Query::EditMacro(key.clone()).into(), cx)
            }
            HomeEvent::Macro(MacroCardEvent::Toggle(key, enabled)) => {
                self.send(Command::SetMacroEnabled(key.clone(), *enabled), cx)
            }
            HomeEvent::Macro(MacroCardEvent::Delete(key)) => {
                self.confirm_deletion(DeleteTarget::Macro(key.clone()), window, cx)
            }
            HomeEvent::Collection(CollectionEvent::Toggle(id, enabled)) => {
                self.send(Command::SetCollectionEnabled(id.clone(), *enabled), cx)
            }
            HomeEvent::Collection(CollectionEvent::NameForm(_)) => unreachable!(),
            HomeEvent::Collection(CollectionEvent::Delete(id)) => {
                self.confirm_deletion(DeleteTarget::Collection(id.clone()), window, cx)
            }
        }
    }
}
