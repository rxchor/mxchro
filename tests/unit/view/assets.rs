use super::AppAssets;
use gpui_kit::{AssetSource, assets::IconName};

#[test]
fn bundles_action_icons_without_losing_default_component_assets() {
    for icon in [
        IconName::ArrowDownToLine,
        IconName::ArrowUpFromLine,
        IconName::Timer,
        IconName::GripVertical,
        IconName::ChevronDown,
        IconName::Check,
        IconName::Close,
        IconName::CircleDot,
    ] {
        let path = icon.path();
        let bytes = AppAssets.load(path.as_ref()).unwrap().unwrap();
        assert!(!bytes.is_empty(), "Missing {path}");
        assert!(AppAssets.list("icons/").unwrap().contains(&path));
    }
}

#[test]
fn logo_variants_share_the_svg_geometry_and_follow_the_selected_theme() {
    use crate::{
        application::dto::{LogoChoice, ThemeChoice},
        view::{
            assets::{logo_path, logo_svg},
            theme,
        },
    };
    for choice in ThemeChoice::ALL {
        let path = logo_path(LogoChoice::FollowTheme, choice);
        let svg = logo_svg(path).unwrap();
        let palette = theme::palette(choice);
        assert!(svg.contains(&format!("#{:06X}", palette.logo_m)));
        assert!(svg.contains(&format!("#{:06X}", palette.logo_x)));
        assert!(svg.contains("M64 188V68"));
        assert!(AppAssets.load(path).unwrap().is_some());
    }
    assert_eq!(
        logo_svg(logo_path(LogoChoice::Monochrome, ThemeChoice::RosePine)).unwrap(),
        include_str!("../../../assets/icon.svg")
    );
    assert_eq!(
        logo_path(LogoChoice::Catppuccin, ThemeChoice::Monochrome),
        logo_path(LogoChoice::Catppuccin, ThemeChoice::RosePine)
    );
}
