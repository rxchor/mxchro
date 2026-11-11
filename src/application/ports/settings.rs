use crate::application::dto::AppSettingsDTO;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsError {
    Read,
    Invalid,
    Write,
}

pub trait SettingsRepository: Send + Sync {
    fn load(&self) -> Result<AppSettingsDTO, SettingsError>;
    fn save(&self, settings: AppSettingsDTO) -> Result<(), SettingsError>;
}
