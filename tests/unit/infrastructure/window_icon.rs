use super::icon_pixels;

#[test]
fn icon_pixels_keep_premultiplied_alpha_and_build_a_word_aligned_transparency_mask() {
    let rgba = [10, 20, 30, 255, 5, 6, 7, 128, 0, 0, 0, 0, 1, 2, 3, 255];
    let (bgra, mask) = icon_pixels(&rgba, 2);
    assert_eq!(
        bgra,
        [30, 20, 10, 255, 7, 6, 5, 128, 0, 0, 0, 0, 3, 2, 1, 255]
    );
    assert_eq!(mask, [0, 0, 0x80, 0]);
}
