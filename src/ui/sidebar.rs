use crate::buffer::Buffer;
use crate::fs::tree::{FileNode, build_file_tree};
use gpui::*;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

pub struct SidebarView {
    pub files: Vec<FileNode>,
    pub active_buffer: Entity<Buffer>,
    pub expanded_dirs: HashSet<PathBuf>,
}

impl SidebarView {
    pub fn new(active_buffer: Entity<Buffer>, _cx: &mut Context<Self>) -> Self {
        let files = build_file_tree(".");
        let mut expanded_dirs = HashSet::new();

        expanded_dirs.insert(PathBuf::from("."));

        Self {
            files,
            active_buffer,
            expanded_dirs,
        }
    }

    fn toggle_dir(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if self.expanded_dirs.contains(&path) {
            self.expanded_dirs.remove(&path);
        } else {
            self.expanded_dirs.insert(path);
        }
        cx.notify();
    }

    fn open_file(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.active_buffer.update(cx, |buffer, cx_model| {
            if let Ok(text) = std::fs::read_to_string(&path) {
                buffer.text = text.into();
                buffer.file_path = Some(path);
                buffer.is_dirty = false;
                cx_model.notify();
            }
        });
    }
}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut visible_nodes = Vec::new();

        fn flatten_nodes<'a>(
            nodes: &'a [FileNode],
            expanded: &HashSet<PathBuf>,
            depth: usize,
            result: &mut Vec<(usize, &'a FileNode)>,
        ) {
            for node in nodes {
                result.push((depth, node));
                if node.is_dir
                    && expanded.contains(&node.path)
                    && let Some(children) = &node.children
                {
                    flatten_nodes(children, expanded, depth + 1, result);
                }
            }
        }

        flatten_nodes(&self.files, &self.expanded_dirs, 0, &mut visible_nodes);

        let mut list = div()
            .flex()
            .flex_col()
            .w_64()
            .h_full()
            .bg(rgb(0x181825))
            .text_color(rgb(0xcdd6f4))
            .p_2()
            .text_sm();

        for (depth, node) in visible_nodes {
            let path_clone = node.path.clone();
            let is_dir = node.is_dir;
            let is_expanded = self.expanded_dirs.contains(&node.path);
            let indent = depth as f32 * 12.0;

            let mut hasher = DefaultHasher::new();
            path_clone.hash(&mut hasher);
            let path_hash = hasher.finish() as usize;

            let mut item = div()
                .id(("tree_node", path_hash))
                .flex()
                .flex_row()
                .items_center()
                .pl(px(indent + 4.0))
                .py_1()
                .hover(|style| style.bg(rgb(0x313244)))
                .cursor_pointer();

            if is_dir {
                let icon = if is_expanded { "v " } else { "> " };
                item = item
                    .on_click(cx.listener(move |this, _event, _win, cx| {
                        this.toggle_dir(path_clone.clone(), cx);
                    }))
                    .child(div().w_4().text_color(rgb(0x9399b2)).child(icon))
                    .child(
                        div()
                            .text_color(rgb(0x89b4fa))
                            .font_weight(FontWeight::BOLD)
                            .child(node.name.clone()),
                    );
            } else {
                item = item
                    .on_click(cx.listener(move |this, _event, _win, cx| {
                        this.open_file(path_clone.clone(), cx);
                    }))
                    .child(div().w_4().child("  "))
                    .child(div().child(node.name.clone()));
            }

            list = list.child(item);
        }

        list
    }
}
