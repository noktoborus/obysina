//! Draws the icon of the program from `assets/icon.svg`.
//!
//! Every target gets `icon.png`, the icon of the window. A windows target also
//! gets `icon.ico` in a resource linked into `obusina.exe`, so the file and its
//! shortcuts show it. The SVG is the one source: no picture of it is kept.

use std::path::{Path, PathBuf};

/// The source of every picture of the icon.
const SOURCE: &str = "assets/icon.svg";
/// The side of the icon of the window. The desktop scales it down.
const WINDOW_SIDE: u32 = 128;
/// The sides of the pictures in `icon.ico`: what Windows asks for at every
/// scale of the screen, from the list of files to the large tiles.
const ICO_SIDES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];
/// The side from which a picture of `icon.ico` is a PNG; a smaller one is a
/// bitmap, the one form every part of Windows reads.
const ICO_PNG_SIDE: u32 = 256;

fn main() {
    println!("cargo:rerun-if-changed={SOURCE}");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let svg = std::fs::read(SOURCE).expect("the icon is readable");
    let tree = resvg::usvg::Tree::from_data(&svg, &resvg::usvg::Options::default())
        .expect("the icon is an SVG");

    picture(&tree, WINDOW_SIDE)
        .write_png(file(&out.join("icon.png")))
        .expect("the icon of the window is written");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        windows_resource(&tree, &out);
    }
}

/// Writes `icon.ico` and the resource that names it, and links the resource
/// into the binaries.
fn windows_resource(tree: &resvg::usvg::Tree, out: &Path) {
    let mut icons = ico::IconDir::new(ico::ResourceType::Icon);
    for side in ICO_SIDES {
        let image = picture(tree, side);
        let entry = if side >= ICO_PNG_SIDE {
            ico::IconDirEntry::encode_as_png(&image)
        } else {
            ico::IconDirEntry::encode_as_bmp(&image)
        };
        icons.add_entry(entry.expect("a picture is encoded"));
    }
    icons
        .write(file(&out.join("icon.ico")))
        .expect("icon.ico is written");
    let resource = out.join("icon.rc");
    std::fs::write(&resource, "1 ICON \"icon.ico\"\n").expect("icon.rc is written");
    embed_resource::compile(&resource, embed_resource::NONE)
        .manifest_required()
        .expect("the resource of the icon is compiled");
}

/// The icon drawn on a square of `side` pixels, its colours not
/// premultiplied.
fn picture(tree: &resvg::usvg::Tree, side: u32) -> ico::IconImage {
    let mut pixmap = resvg::tiny_skia::Pixmap::new(side, side).expect("a side is not zero");
    let scale = side as f32 / tree.size().width().max(tree.size().height());
    resvg::render(
        tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let colour = pixel.demultiply();
            [colour.red(), colour.green(), colour.blue(), colour.alpha()]
        })
        .collect();
    ico::IconImage::from_rgba_data(side, side, rgba)
}

/// A new file at `path`.
fn file(path: &Path) -> std::fs::File {
    std::fs::File::create(path).expect("a file of OUT_DIR is created")
}
