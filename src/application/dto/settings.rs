/// Stable preference values, independent from display labels and file formats.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeChoice {
    #[default]
    Monochrome,
    Catppuccin,
    RosePine,
}

impl ThemeChoice {
    pub const ALL: [Self; 3] = [Self::Monochrome, Self::Catppuccin, Self::RosePine];

    pub fn key(self) -> &'static str {
        match self {
            Self::Monochrome => "monochrome",
            Self::Catppuccin => "catppuccin",
            Self::RosePine => "rose_pine",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|choice| choice.key() == key)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogoChoice {
    #[default]
    FollowTheme,
    Monochrome,
    Catppuccin,
    RosePine,
}

impl LogoChoice {
    pub const ALL: [Self; 4] = [
        Self::FollowTheme,
        Self::Monochrome,
        Self::Catppuccin,
        Self::RosePine,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::FollowTheme => "follow_theme",
            Self::Monochrome => "monochrome",
            Self::Catppuccin => "catppuccin",
            Self::RosePine => "rose_pine",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|choice| choice.key() == key)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppSettingsDTO {
    pub theme: ThemeChoice,
    pub logo: LogoChoice,
}
