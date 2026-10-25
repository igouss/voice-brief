use std::path::Path;
use std::{fs, process};

fn main() {
    let rendered: vb_views::Rendered = vb_views::talk_machine(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../talk"));
    if !rendered.warnings.is_empty() {
        eprintln!("vb-views: the push-to-talk diagram parses with warnings: {:?}", rendered.warnings);
        process::exit(1);
    }
    for view in rendered.views {
        if let Err(e) = fs::write(&view.path, view.text) {
            eprintln!("vb-views: {}: {e}", view.path.display());
            process::exit(1);
        }
        println!("{}", view.path.display());
    }
}
