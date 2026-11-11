//! Composition root: the only place that assembles production storage, engine,
//! input hooks, ordered worker and the desktop interface.
use crate::{
    application::{
        engine::Managers,
        ports::{InputBackend, SettingsRepository, WindowIconBackend},
    },
    infrastructure::{
        input::{capture::WindowsKeyCapture, hook::Hook, mapper::KeyMapper},
        persistence::Persistence,
        settings::FileSettingsRepository,
        window_icon::WindowsWindowIcon,
    },
    view::{
        assets::AppAssets,
        backend::{
            CommandBus,
            settings::{self, SettingsBus},
            worker,
        },
        i18n, theme,
        ui::MxchroApp,
    },
};
use gpui_kit::*;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

fn start_backend(input: Arc<dyn InputBackend>, capture: Arc<WindowsKeyCapture>) -> CommandBus {
    let (requests, receiver) = mpsc::channel();
    let (sender, updates) = mpsc::channel();
    std::thread::spawn(move || {
        // Database opening/loading and native assembly stay off the GPUI thread.
        let persistence = Persistence::new();
        let managers = Managers::new(input, persistence.repositories());
        let dirty = Arc::new(AtomicBool::new(false));
        let hook_dirty = Arc::clone(&dirty);
        Hook::install(Arc::clone(&managers.macro_manager), capture, move || {
            hook_dirty.store(true, Ordering::Release);
        });
        worker::run(
            &managers.macro_manager,
            &managers.collection_manager,
            receiver,
            sender,
            &dirty,
        );
        managers.macro_manager.clear_macros();
    });
    CommandBus { requests, updates }
}

/// Initial preferences are read before GPUI starts; subsequent writes use a worker.
fn start_settings() -> SettingsBus {
    let repository = FileSettingsRepository::new(FileSettingsRepository::configured_path());
    let loaded = repository.load();
    let initial = loaded.as_ref().copied().unwrap_or_default();
    let load_error = loaded.err();
    let (requests, receiver) = mpsc::channel();
    let (sender, updates) = mpsc::channel();
    std::thread::spawn(move || settings::run(&repository, receiver, sender));
    SettingsBus {
        initial,
        load_error,
        requests,
        updates,
    }
}

pub fn run() {
    dotenvy::dotenv().ok();
    env_logger::init();
    i18n::init();
    let settings = start_settings();
    let input: Arc<dyn InputBackend> = Arc::new(KeyMapper::new());
    let capture = Arc::new(WindowsKeyCapture::new(input.clone()));
    let commands = start_backend(input, capture.clone());
    gpui_kit::application()
        .with_assets(AppAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            theme::init(cx);
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1160.), px(780.)),
                        cx,
                    ))),
                    window_min_size: Some(size(px(900.), px(600.))),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Mxchro".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                cx,
                move |window, cx| {
                    let window_icon = match HasWindowHandle::window_handle(window)
                        .map(|handle| handle.as_raw())
                    {
                        Ok(RawWindowHandle::Win32(handle)) => {
                            capture.bind_window(handle.hwnd.get());
                            Some(Box::new(WindowsWindowIcon::new(handle.hwnd.get()))
                                as Box<dyn WindowIconBackend>)
                        }
                        other => {
                            log::warn!("Native window icon unavailable: {other:?}");
                            None
                        }
                    };
                    cx.new(|cx| {
                        MxchroApp::new(commands, settings, window_icon, window, cx)
                            .with_key_capture(capture)
                    })
                },
            )
            .expect("Could not open the Mxchro window");
            cx.activate(true);
        });
}
