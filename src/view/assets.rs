//! Default GPUI-KIT assets plus action, drag-handle and input-recording icons.
use crate::application::dto::{LogoChoice, ThemeChoice};
use crate::view::theme;
use gpui_kit::{AssetSource, Result, SharedString};
use std::borrow::Cow;

const LOGO_PATHS: [&str; 3] = [
    "logos/monochrome.svg",
    "logos/catppuccin.svg",
    "logos/rose-pine.svg",
];

pub fn logo_path(choice: LogoChoice, selected_theme: ThemeChoice) -> &'static str {
    match choice {
        LogoChoice::Monochrome => LOGO_PATHS[0],
        LogoChoice::Catppuccin => LOGO_PATHS[1],
        LogoChoice::RosePine => LOGO_PATHS[2],
        LogoChoice::FollowTheme => match selected_theme {
            ThemeChoice::Monochrome => LOGO_PATHS[0],
            ThemeChoice::Catppuccin => LOGO_PATHS[1],
            ThemeChoice::RosePine => LOGO_PATHS[2],
        },
    }
}

fn logo_svg(path: &str) -> Option<String> {
    let index = LOGO_PATHS.iter().position(|candidate| *candidate == path)?;
    let source = include_str!("../../assets/icon.svg");
    if index == 0 {
        return Some(source.to_owned());
    }
    let palette = theme::palette(ThemeChoice::ALL[index]);
    Some(
        source
            .replace(
                &format!("#{:06X}", theme::ORIGINAL_LOGO_BACKGROUND),
                &format!("#{:06X}", palette.elevated),
            )
            .replace(
                &format!("#{:06X}", theme::ORIGINAL_LOGO_M),
                &format!("#{:06X}", palette.logo_m),
            )
            .replace(
                &format!("#{:06X}", theme::ORIGINAL_LOGO_X),
                &format!("#{:06X}", palette.logo_x),
            ),
    )
}

gpui_kit::assets::icon_assets!(
    ActionIcons,
    [
        ArrowDownToLine,
        ArrowUpFromLine,
        Timer,
        GripVertical,
        CircleDot
    ]
);

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(svg) = logo_svg(path) {
            return Ok(Some(Cow::Owned(svg.into_bytes())));
        }
        match ActionIcons.load(path)? {
            Some(asset) => Ok(Some(asset)),
            None => gpui_kit::assets::Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = gpui_kit::assets::Assets.list(path)?;
        paths.extend(ActionIcons.list(path)?);
        paths.extend(
            LOGO_PATHS
                .iter()
                .filter(|candidate| candidate.starts_with(path))
                .map(|path| SharedString::from(*path)),
        );
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/view/assets.rs"]
mod tests;
