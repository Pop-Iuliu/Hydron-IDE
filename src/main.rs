mod buffer;
mod fs;
mod ui;

use gpui::*;
use ui::Workspace;
use ui::editor::Save;

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("cmd-s", Save, Some("Editor")),
            KeyBinding::new("ctrl-s", Save, Some("Editor")),
        ]);

        cx.open_window(
            WindowOptions::default(),
            |_window: &mut Window, cx: &mut App| cx.new(|cx| Workspace::new(cx)),
        )
        .unwrap();
    });
}
