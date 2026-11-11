//! Native settings journey; fake macro channels and an exclusive temporary file.
use gpui_kit::{
    component::{ActiveTheme, WindowExt},
    *,
};
use mxchro::{
    application::{
        commands::Command,
        dto::{AppSettingsDTO, LogoChoice, MacroDTO, MacroEditDTO, ThemeChoice},
        ports::{SettingsError, SettingsRepository},
        queries::{Query, Snapshot},
        request::Request,
    },
    domain::types::{Action, Mode},
    infrastructure::settings::FileSettingsRepository,
    view::{
        assets::AppAssets,
        backend::{
            CommandBus, Update,
            settings::{self, SettingsBus},
        },
        i18n, theme,
        ui::MxchroApp,
    },
};
use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};
#[path = "support/window_icon.rs"]
mod window_icon;

struct ControlledRepository {
    file: FileSettingsRepository,
    fail: Arc<AtomicBool>,
    written: mpsc::Sender<AppSettingsDTO>,
}

impl SettingsRepository for ControlledRepository {
    fn load(&self) -> Result<AppSettingsDTO, SettingsError> {
        self.file.load()
    }
    fn save(&self, settings: AppSettingsDTO) -> Result<(), SettingsError> {
        if self.fail.load(Ordering::Acquire) {
            return Err(SettingsError::Write);
        }
        self.file.save(settings)?;
        // Reopen the file on this worker, never on the GPUI thread.
        assert_eq!(self.file.load()?, settings);
        self.written.send(settings).unwrap();
        Ok(())
    }
}

fn snapshot() -> Snapshot {
    Snapshot {
        hotkey_issues: Vec::new(),
        macros: vec![MacroDTO {
            trigger_key: "F6".into(),
            mode: Mode::Once,
            actions_count: 1,
            block_original_event: false,
            ignore_injected_keys: true,
            enabled: false,
            collection_id: None,
            toggle_enabled_hotkey: None,
        }],
        collections: vec![],
        keys: vec!["A".into(), "F6".into()],
    }
}

fn click(handle: AnyWindowHandle, x: f32, y: f32, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        let position = point(px(x), px(y));
        window.dispatch_event(
            MouseMoveEvent {
                position,
                pressed_button: None,
                modifiers: Modifiers::default(),
            }
            .to_platform_input(),
            cx,
        );
        window.draw(cx).clear(cx);
        window.dispatch_event(
            MouseDownEvent {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
        window.draw(cx).clear(cx);
    })
    .unwrap();
}

fn bottom(handle: AnyWindowHandle, cx: &mut AsyncApp) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            ScrollEvent {
                position: point(px(800.), px(400.)),
                modifiers: Modifiers::default(),
                delta: ScrollDelta::Pixels(point(px(0.), px(-2000.))),
                touch_phase: TouchPhase::Moved,
                source: ScrollSource::Pointer,
            }
            .to_platform_input(),
            cx,
        );
        window.draw(cx).clear(cx);
    })
    .unwrap();
}

async fn settle(cx: &mut AsyncApp) {
    cx.background_executor()
        .timer(Duration::from_millis(250))
        .await;
}

fn assert_theme(choice: ThemeChoice, cx: &mut AsyncApp) {
    cx.update(|cx| {
        let p = theme::palette(choice);
        let theme = cx.theme();
        assert_eq!(theme.background, Hsla::from(rgb(p.background)));
        assert_eq!(theme.primary, Hsla::from(rgb(p.primary)));
        assert_eq!(theme.tokens.background.color, theme.background);
        assert_eq!(theme.tokens.button.color, theme.button);
        assert_eq!(theme.tokens.primary.color, theme.primary);
    });
}

fn main() {
    let directory = std::env::temp_dir()
        .join("opencode")
        .join(format!("mxchro-settings-ui-{}", uuid::Uuid::new_v4()));
    let path = directory.join("settings.toml");
    let fail = Arc::new(AtomicBool::new(true));
    let (written, saved) = mpsc::channel();
    let repository = ControlledRepository {
        file: FileSettingsRepository::new(path.clone()),
        fail: fail.clone(),
        written,
    };
    let initial = repository.load().unwrap();
    let (requests, receiver) = mpsc::channel();
    let (sender, updates) = mpsc::channel();
    let worker = std::thread::spawn(move || settings::run(&repository, receiver, sender));
    let bus = SettingsBus {
        initial,
        load_error: None,
        requests,
        updates,
    };
    i18n::init();
    gpui_kit::application().with_assets(AppAssets).run(move |cx| {
        gpui_kit::init(cx);
        theme::init(cx);
        let (commands, macro_requests) = mpsc::channel();
        let (macro_updates, receiver) = mpsc::channel();
        let (handle, _) = gpui_kit::open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(20.), px(20.)), size: size(px(1160.), px(780.)) })),
            ..Default::default()
        }, cx, |window, cx| {
            let icon = window_icon::backend(window);
            cx.new(|cx| MxchroApp::new(CommandBus { requests: commands, updates: receiver }, bus, Some(icon), window, cx))
        }).unwrap();
        macro_updates.send(Update { request_completed: false, snapshot: snapshot(), error: None, saved: false, editor: None }).unwrap();
        cx.spawn(async move |cx| {
            settle(cx).await;
            cx.update_window(handle, |_, window, _| window_icon::check_failure_and_restore(window)).unwrap();
            let monochrome_icons = cx.update_window(handle, |_, window, _| window_icon::handles(window)).unwrap();
            let monochrome_digest = window_icon::digests(monochrome_icons);
            click(handle, 947., 230., cx);
            assert!(matches!(macro_requests.try_recv(), Ok(Request::Query(Query::EditMacro(_)))));
            macro_updates.send(Update {
                request_completed: true, snapshot: snapshot(), error: None, saved: false,
                editor: Some(MacroEditDTO { trigger_key: "F6".into(), mode: Mode::Once,
                    actions: vec![Action::Wait { duration_ms: 123 }], block_original_event: false,
                    ignore_injected_keys: true, enabled: false, collection_id: None, toggle_enabled_hotkey: None }),
            }).unwrap();
            settle(cx).await;
            click(handle, 208., 750., cx);
            settle(cx).await;
            click(handle, 716., 284., cx);
            settle(cx).await;
            assert_theme(ThemeChoice::Catppuccin, cx);
            let mut previous_icons = cx.update_window(handle, |_, window, _| window_icon::handles(window)).unwrap();
            let catppuccin_digest = window_icon::digests(previous_icons);
            assert_ne!(catppuccin_digest, monochrome_digest);
            window_icon::assert_released(monochrome_icons);
            assert!(saved.try_recv().is_err(), "failed preference write reported success");
            assert_eq!(cx.update_window(handle, |_, window, cx| window.notifications(cx).len()).unwrap(), 1);
            fail.store(false, Ordering::Release);
            bottom(handle, cx);
            click(handle, 700., 740., cx); // Retry, at the bottom of the settings page.
            settle(cx).await;
            assert_eq!(saved.try_recv().unwrap(), AppSettingsDTO { theme: ThemeChoice::Catppuccin, logo: LogoChoice::FollowTheme });
            cx.update_window(handle, |_, window, cx| window.clear_notifications(cx)).unwrap();
            settle(cx).await;

            let mut previous_path = mxchro::view::assets::logo_path(LogoChoice::FollowTheme, ThemeChoice::Catppuccin);
            for (x, y, expected) in [
                (496., 511., AppSettingsDTO { theme: ThemeChoice::Catppuccin, logo: LogoChoice::Monochrome }),
                (1019., 296., AppSettingsDTO { theme: ThemeChoice::RosePine, logo: LogoChoice::Monochrome }),
                (413., 296., AppSettingsDTO { theme: ThemeChoice::Monochrome, logo: LogoChoice::Monochrome }),
                (344., 511., AppSettingsDTO { theme: ThemeChoice::Monochrome, logo: LogoChoice::FollowTheme }),
                (716., 296., AppSettingsDTO { theme: ThemeChoice::Catppuccin, logo: LogoChoice::FollowTheme }),
                (496., 511., AppSettingsDTO { theme: ThemeChoice::Catppuccin, logo: LogoChoice::Monochrome }),
                (648., 511., AppSettingsDTO { theme: ThemeChoice::Catppuccin, logo: LogoChoice::Catppuccin }),
                (800., 511., AppSettingsDTO { theme: ThemeChoice::Catppuccin, logo: LogoChoice::RosePine }),
            ] {
                click(handle, x, y, cx);
                settle(cx).await;
                let received = saved.try_recv();
                assert_eq!(received.unwrap_or_else(|error| panic!("settings choice missed at {x}/{y}: {error}")), expected);
                assert_theme(expected.theme, cx);
                let icons = cx.update_window(handle, |_, window, _| window_icon::handles(window)).unwrap();
                let path = mxchro::view::assets::logo_path(expected.logo, expected.theme);
                if path == previous_path {
                    assert_eq!(icons, previous_icons, "fixed logo was regenerated on a theme-only change");
                } else {
                    assert_ne!(icons, previous_icons);
                    window_icon::assert_released(previous_icons);
                }
                let digest = window_icon::digests(icons);
                if path.ends_with("monochrome.svg") { assert_eq!(digest, monochrome_digest); }
                if path.ends_with("catppuccin.svg") { assert_eq!(digest, catppuccin_digest); }
                previous_icons = icons;
                previous_path = path;
                assert!(macro_requests.try_recv().is_err(), "appearance changes touched the macro backend");
            }
            click(handle, 1100., 40., cx); // Back to the retained editor.
            settle(cx).await;
            bottom(handle, cx);
            click(handle, 1015., 740., cx);
            let Ok(Request::Command(Command::SaveMacro { config, .. })) = macro_requests.try_recv() else { panic!("settings did not return to the editor"); };
            assert_eq!(config.trigger_key, "F6");
            assert!(matches!(config.actions.as_slice(), [Action::Wait { duration_ms: 123 }]));
            println!("Native settings passed: live palettes/tokens, four logo choices, Windows small/large icon pixels, fixed/follow behavior, icon ownership/restore, isolated persistence/retry and retained macro draft.");
            cx.update(|cx| cx.quit());
        }).detach();
    });
    worker.join().unwrap();
    assert_eq!(
        FileSettingsRepository::new(path).load().unwrap(),
        AppSettingsDTO {
            theme: ThemeChoice::Catppuccin,
            logo: LogoChoice::RosePine
        }
    );
    fs::remove_dir_all(directory).unwrap();
}
