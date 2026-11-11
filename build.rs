use image::{
    ExtendedColorType,
    codecs::ico::{IcoEncoder, IcoFrame},
};
use resvg::{tiny_skia, usvg};
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.svg");
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let svg = fs::read("assets/icon.svg").expect("Could not read the application icon SVG");
    let tree = usvg::Tree::from_data(&svg, &usvg::Options::default())
        .expect("Could not parse the application icon SVG");
    let frames: Vec<_> = [16, 24, 32, 48, 64, 128, 256]
        .into_iter()
        .map(|size| {
            let mut pixmap =
                tiny_skia::Pixmap::new(size, size).expect("Could not allocate icon pixels");
            let scale = (size as f32 / tree.size().width()).min(size as f32 / tree.size().height());
            let transform = tiny_skia::Transform::from_row(
                scale,
                0.,
                0.,
                scale,
                (size as f32 - tree.size().width() * scale) / 2.,
                (size as f32 - tree.size().height() * scale) / 2.,
            );
            resvg::render(&tree, transform, &mut pixmap.as_mut());
            // tiny-skia uses premultiplied alpha; PNG expects straight RGBA.
            let rgba: Vec<u8> = pixmap
                .pixels()
                .iter()
                .flat_map(|pixel| {
                    let pixel = pixel.demultiply();
                    [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]
                })
                .collect();
            IcoFrame::as_png(&rgba, size, size, ExtendedColorType::Rgba8)
                .expect("Could not encode an application icon frame")
        })
        .collect();
    let icon_path = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo did not set OUT_DIR"))
        .join("mxchro.ico");
    IcoEncoder::new(fs::File::create(&icon_path).expect("Could not create the application ICO"))
        .encode_images(&frames)
        .expect("Could not encode the application ICO");

    // GPUI's Windows platform loads the executable's icon resource with ID 1.
    winresource::WindowsResource::new()
        .set_icon_with_id(icon_path.to_str().expect("Invalid icon output path"), "1")
        .compile()
        .expect("Could not embed the Windows application icon");
}
