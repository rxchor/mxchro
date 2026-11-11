//! In-process application data contracts, not serialized UI messages or domain entities.
mod collections;
mod hotkeys;
mod macros;
mod settings;
pub use collections::CollectionDTO;
pub use hotkeys::{HotkeyIssueDTO, HotkeyIssueKind, HotkeyOwnerDTO};
pub use macros::{MacroDTO, MacroEditDTO, SaveMacroDTO};
pub use settings::{AppSettingsDTO, LogoChoice, ThemeChoice};
