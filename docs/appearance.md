# Appearance, preferences and language

[Index](README.md) · [Data](data.md) · [EXE icon](packaging.md#executable-icon)

Read before changing themes, logos, window icons, settings or visible text.

## Themes and logos

Settings opens from the bottom of the sidebar. Three dark palettes, independent
of the system theme: Monochrome (default, black/white), Catppuccin Mocha and
Rosé Pine (main). Changes apply immediately without losing the editor draft.

- Centralize colors as RGB `0xRRGGBB` constants in `src/view/theme.rs`, including
  original SVG colors. Views consume `cx.theme()`; no scattered hex values.
  Only Settings swatches explicitly inspect other palettes.
- Run `theme::init` after `gpui_kit::init` and before opening windows.
- Use `Theme::update` to synchronize colors, tokens and Base. Do not modify only
  `Theme::global_mut` and leave copied values stale.
- The logo uses the single geometry in `assets/icon.svg`: Follow theme (default),
  Monochrome, Catppuccin or Rosé Pine. Do not restore the redundant Original option.
- Theme/logo buttons need separate ID scopes (`theme-options`, `logo-options`):
  shared values must not share GPUI state.

The base SVG and EXE resource are Monochrome. The internal logo and small/large
window icons change at runtime. Bootstrap injects a `WindowIconBackend` bound to
that window; the UI supplies SVG without knowing Windows APIs.
`infrastructure/window_icon.rs` rasterizes in memory and applies `WM_SETICON`
only on the owning thread. Prepare both sizes before changing them; release only
owned previous resources afterward, and restore borrowed icons when removing the
adapter if they are still installed. Failures preserve previous icons and show
a notification. No disk/hooks or changes to other windows, EXEs, pinned shortcuts or caches.

`settings_ui` checks icons/ownership; final taskbar/pinned appearance still needs
manual review. ICO generation: [packaging](packaging.md).

## TOML preferences

`AppSettingsDTO`, `ThemeChoice`, `LogoChoice` live in `application/dto/`;
`SettingsRepository` in ports; private TOML in `infrastructure/settings.rs`.
Default: `%APPDATA%/mxchro-native/settings.toml`, or
`./mxchro-native/settings.toml` without APPDATA or when it is empty. Never PoloDB.

```powershell
$env:MXCHRO_SETTINGS_PATH = "D:\Mxchro\settings.toml"
cargo run
```

Or in `.env`:

```dotenv
MXCHRO_SETTINGS_PATH=./portable/settings.toml
```

Relative paths start at the launch directory; preserve the exact path/extension.
An absent or empty variable retains the default. A path change requires restarting;
never automatically migrate or delete the previous file. Copy preferences manually
before launching if they should be preserved. This does not change database paths,
schema or macros; `MACROS_FILE_PATH` has a different contract: [data](data.md).

Bootstrap reads preferences before the GPUI loop and assembles an independent
write worker; the UI never accesses disk. A missing file uses defaults without
creating it. Invalid/unreadable files report an error and remain untouched, without
automatic reset, until the user chooses to save. On write failure the preview
remains active but is not considered persisted; show the failure and allow retry.
Writes use an exclusive temporary file beside the target and replacement by rename.
Native tests use only an exclusive temporary preferences file.

## Language and catalog

The UI is English-only, without a language selector or system-language detection.
`rust-i18n` loads `locales/en.yml` at compile time with English fallback. Labels
use `i18n::text`; messages with data use crate interpolation. Initialize
`i18n::init` before the backend and keep GPUI-KIT in the same language.

Do not translate mapper names, IDs, persisted values, user collection names or
technical logs. The domain does not import i18n. Mode labels are independent of
selector/persisted values. Counters need singular/plural forms; test the catalog,
interpolation and literal keys used in sources. Infrastructure for additional
catalogs does not mean other languages or their selection are implemented.
