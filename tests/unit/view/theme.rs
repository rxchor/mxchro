use super::*;

#[test]
fn palette_uses_black_background_and_consistent_component_colors() {
    let mut colors = ThemeColor::default();
    apply_palette(&mut colors, MONOCHROME);
    assert_eq!(colors.background, color(BACKGROUND));
    assert_eq!(colors.foreground, color(TEXT));
    assert_eq!(colors.sidebar, color(SURFACE));
    assert_eq!(colors.button_primary, colors.primary);
    assert_eq!(colors.button_primary_foreground, colors.primary_foreground);
    assert_eq!(colors.button_hover, color(HOVER));
    assert_eq!(colors.input, colors.border);
}

#[test]
fn every_palette_updates_all_interactive_and_status_colors() {
    let mut colors = ThemeColor::default();
    for choice in ThemeChoice::ALL {
        let p = palette(choice);
        apply_palette(&mut colors, p);
        assert_eq!(colors.background, color(p.background));
        assert_eq!(colors.foreground, color(p.text));
        assert_eq!(colors.sidebar, color(p.surface));
        assert_eq!(colors.primary, color(p.primary));
        assert_eq!(colors.button_primary, colors.primary);
        assert_eq!(colors.button_primary_foreground, colors.primary_foreground);
        assert_eq!(colors.button_danger, colors.danger);
        assert_eq!(colors.button_success, colors.success);
        assert_eq!(colors.button_info, colors.info);
        assert_eq!(colors.button_warning, colors.warning);
    }
    apply_palette(&mut colors, MONOCHROME);
    assert_eq!(colors.background, color(BACKGROUND));
}
