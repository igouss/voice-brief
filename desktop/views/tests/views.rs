use std::fs;
use std::path::{Path, PathBuf};

use vb_views::talk_machine;

fn talk_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../talk")
}

#[test]
fn the_committed_views_are_what_step_draws() {
    let stale: Vec<String> = talk_machine(&talk_dir())
        .views
        .into_iter()
        .filter(|v| fs::read_to_string(&v.path).ok().as_deref() != Some(v.text.as_str()))
        .map(|v| v.path.display().to_string())
        .collect();
    assert_eq!(stale, Vec::<String>::new(), "regenerate: cargo run -q -p vb-views");
}

#[test]
fn the_diagram_source_parses_without_warnings() {
    assert_eq!(talk_machine(&talk_dir()).warnings, Vec::<String>::new());
}

#[test]
fn rendering_twice_gives_the_same_bytes() {
    let first: Vec<String> = talk_machine(&talk_dir()).views.into_iter().map(|v| v.text).collect();
    let second: Vec<String> = talk_machine(&talk_dir()).views.into_iter().map(|v| v.text).collect();
    assert_eq!(first, second);
}

#[test]
fn the_svg_needs_no_css_custom_properties() {
    let svg: String = talk_machine(&talk_dir()).views.into_iter().find(|v| v.path.extension().is_some_and(|e| e == "svg")).map(|v| v.text).unwrap_or_default();
    assert_eq!((svg.starts_with("<svg"), svg.matches("var(--").count()), (true, 0));
}
