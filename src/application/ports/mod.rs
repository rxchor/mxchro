//! Contracts owned by application, implemented by infrastructure or test fakes.
mod input;
mod key_capture;
mod settings;
mod storage;
mod window_icon;
pub use input::InputBackend;
pub use key_capture::{CaptureError, CaptureOutcome, CaptureSession, KeyCaptureBackend};
pub use settings::{SettingsError, SettingsRepository};
pub use storage::{CollectionRepository, MacroRepository, Repositories};
pub use window_icon::{WindowIconBackend, WindowIconError};
