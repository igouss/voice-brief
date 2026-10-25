mod view;

use gpui_kit::{AppContext as _, Bounds, TitlebarOptions, WindowBounds, WindowOptions, point, px, size};
use voice_brief_desktop::store::data_root;

use crate::view::Desk;

fn main() {
    let home: Option<std::path::PathBuf> = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let root = data_root(std::env::var("XDG_DATA_HOME").ok().as_deref(), std::env::var("HOME").ok().as_deref());
    gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(move |cx| {
        gpui_omarchy::init(cx);
        let options: WindowOptions = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(point(px(0.), px(0.)), size(px(1100.), px(680.))))),
            titlebar: Some(TitlebarOptions { title: Some("Voice brief".into()), ..Default::default() }),
            app_id: Some("voice-brief".to_owned()),
            window_min_size: Some(size(px(720.), px(420.))),
            ..Default::default()
        };
        let window = cx.open_window(options, move |window, cx| cx.new(|cx| Desk::new(root.clone(), home.clone(), window, cx))).expect("open the voice-brief window");
        let _ = window.update(cx, |desk, window, cx| window.focus(&desk.focus, cx));
        cx.activate(true);
    });
}
