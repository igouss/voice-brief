use std::path::Path;
use std::{fs, process};

use resvg::{tiny_skia, usvg};

fn main() {
    let Some(out) = std::env::args().nth(1) else {
        eprintln!("usage: preview <out.png>");
        process::exit(2);
    };
    let svg: String = vb_views::talk_machine(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../talk"))
        .views
        .into_iter()
        .find(|v| v.path.extension().is_some_and(|e| e == "svg"))
        .map(|v| v.text)
        .unwrap_or_default();
    let mut options: usvg::Options<'_> = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree: usvg::Tree = match usvg::Tree::from_str(&svg, &options) {
        Ok(tree) => tree,
        Err(e) => {
            eprintln!("preview: the SVG does not parse: {e}");
            process::exit(1);
        }
    };
    let scale: f32 = 1.5;
    let size: tiny_skia::IntSize = tree.size().to_int_size().scale_by(scale).unwrap_or(tree.size().to_int_size());
    let Some(mut pixmap) = tiny_skia::Pixmap::new(size.width(), size.height()) else {
        eprintln!("preview: empty diagram");
        process::exit(1);
    };
    pixmap.fill(tiny_skia::Color::WHITE);
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    if let Err(e) = pixmap.save_png(&out).map_err(|e| e.to_string()).and_then(|()| fs::metadata(&out).map(|_| ()).map_err(|e| e.to_string())) {
        eprintln!("preview: {out}: {e}");
        process::exit(1);
    }
    println!("{out}");
}
