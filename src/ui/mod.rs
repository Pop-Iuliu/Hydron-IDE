pub mod editor;
pub mod sidebar;

use crate::buffer::Buffer;
use editor::EditorView;
use gpui::*;
use sidebar::SidebarView;

pub struct Workspace {
    sidebar: Entity<SidebarView>,
    editor: Entity<EditorView>,
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let buffer_entity = cx.new(|_| Buffer::new());

        let sidebar = cx.new(|cx| SidebarView::new(buffer_entity.clone(), cx));
        let editor = cx.new(|cx| EditorView::new(buffer_entity, cx));

        Self { sidebar, editor }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .child(self.sidebar.clone())
            .child(self.editor.clone())
    }
}
