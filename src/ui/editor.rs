use crate::buffer::Buffer;
use gpui::*;

actions!(editor, [Save]);

pub struct EditorView {
    pub buffer: Entity<Buffer>,
    focus_handle: FocusHandle,
    cursor_offset: usize,
    pub selection_anchor: Option<usize>,
}

impl EditorView {
    pub fn new(buffer: Entity<Buffer>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        cx.observe(&buffer, |_this, _buffer, cx| {
            cx.notify();
        })
        .detach();

        Self {
            buffer,
            focus_handle,
            cursor_offset: 0,
            selection_anchor: None,
        }
    }

    fn handle_save(&mut self, _: &Save, _window: &mut Window, cx: &mut Context<Self>) {
        self.buffer.update(cx, |buffer: &mut Buffer, _| {
            if let Err(e) = buffer.save() {
                eprintln!("Eroare la salvarea fisierului: {:?}", e);
            } else {
                println!("Fisier salvat cu succes!");
            }
        });
    }
    // functie pentru event handling in caz de key press
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let is_ctrl_or_cmd =
            event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

        if is_ctrl_or_cmd && event.keystroke.key == "z" {
            self.buffer.update(cx, |buffer, _cx| {
                if let Some(new_cursor_pos) = buffer.undo() {
                    self.cursor_offset = new_cursor_pos;
                }
            });
            cx.notify();
            return;
        }

        if is_ctrl_or_cmd && event.keystroke.key == "c" {
            if let Some(range) = self.selection_range() {
                let text = self.buffer.read(cx).text.slice(range).to_string();
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
            }
            return;
        }

        if is_ctrl_or_cmd && event.keystroke.key == "x" {
            if let Some(range) = self.selection_range() {
                let text = self.buffer.read(cx).text.slice(range.clone()).to_string();
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));

                self.buffer.update(cx, |buffer, _cx| {
                    buffer.remove(range.clone());
                });
                self.cursor_offset = range.start;
                self.selection_anchor = None;
                cx.notify();
            }
            return;
        }

        if is_ctrl_or_cmd && event.keystroke.key == "v" {
            if let Some(clipboard_item) = cx.read_from_clipboard()
                && let Some(text) = clipboard_item.text()
            {
                let selection = self.selection_range();
                let mut current_offset = self.cursor_offset;
                self.buffer.update(cx, |buffer, _cx| {
                    if let Some(range) = &selection {
                        buffer.remove(range.clone());
                        current_offset = range.start;
                    }
                    buffer.insert(current_offset, &text);
                });

                self.cursor_offset = current_offset + text.chars().count();
                self.selection_anchor = None;
                cx.notify();
            }
            return;
        }

        if event.keystroke.key == "space" {
            self.buffer.update(cx, |buffer, _cx| {
                buffer.insert(self.cursor_offset, " ");
            });
            self.cursor_offset += 1;
            cx.notify();
            return;
        }

        let is_shift = event.keystroke.modifiers.shift;

        if matches!(
            event.keystroke.key.as_str(),
            "left" | "right" | "up" | "down"
        ) {
            if is_shift && self.selection_anchor.is_none() {
                self.selection_anchor = Some(self.cursor_offset);
                println!("quadeki\n");
            } else if !is_shift {
                self.selection_anchor = None;
            }
        }

        let key = event.keystroke.key.as_str();

        let max_len = self.buffer.read(cx).text.chars().count();
        if self.cursor_offset > max_len {
            self.cursor_offset = max_len;
        }

        let mut new_offset = self.cursor_offset;
        println!("Am apasat tasta: '{}'", key);
        match key {
            "backspace" => {
                if new_offset > 0 {
                    new_offset -= 1;
                    self.buffer.update(cx, |buffer, cx_model| {
                        buffer.remove(new_offset..new_offset + 1);
                        cx_model.notify();
                    });
                }
            }
            "enter" => {
                self.buffer.update(cx, |buffer, cx_model| {
                    buffer.insert(new_offset, "\n");
                    cx_model.notify();
                });
                new_offset += 1;
            }
            "left" => {
                new_offset = new_offset.saturating_sub(1);
            }
            "right" => {
                if new_offset < max_len {
                    new_offset += 1;
                }
            }
            "up" => {
                let text = self.buffer.read(cx).text.to_string();

                let mut current_line_start = 0;
                for (i, c) in text.char_indices() {
                    if i >= new_offset {
                        break;
                    }
                    if c == '\n' {
                        current_line_start = i + 1;
                    }
                }

                let column = new_offset - current_line_start;

                if current_line_start > 0 {
                    let mut prev_line_start = 0;
                    for (i, c) in text.char_indices() {
                        if i >= current_line_start - 1 {
                            break;
                        }
                        if c == '\n' {
                            prev_line_start = i + 1;
                        }
                    }

                    let prev_line_len = (current_line_start - 1) - prev_line_start;
                    new_offset = prev_line_start + std::cmp::min(column, prev_line_len);
                }
            }
            "down" => {
                let text = self.buffer.read(cx).text.to_string();

                let mut current_line_start = 0;
                for (i, c) in text.char_indices() {
                    if i >= new_offset {
                        break;
                    }
                    if c == '\n' {
                        current_line_start = i + 1;
                    }
                }
                let column = new_offset - current_line_start;

                let mut next_line_start = None;
                for (i, c) in text.char_indices().skip(new_offset) {
                    if c == '\n' {
                        next_line_start = Some(i + 1);
                        break;
                    }
                }

                if let Some(start) = next_line_start {
                    let mut next_line_len = 0;
                    for (_i, c) in text.char_indices().skip(start) {
                        if c == '\n' {
                            break;
                        }
                        next_line_len += 1;
                    }
                    new_offset = start + std::cmp::min(column, next_line_len);
                }
            }
            _ => {
                if key.chars().count() == 1 {
                    self.buffer.update(cx, |buffer, cx_model| {
                        buffer.insert(new_offset, key);
                        cx_model.notify();
                    });
                    new_offset += 1;
                }
            }
        }
        self.cursor_offset = new_offset;
        cx.notify();
    }

    fn selection_range(&self) -> Option<std::ops::Range<usize>> {
        self.selection_anchor.map(|anchor| {
            if anchor < self.cursor_offset {
                anchor..self.cursor_offset
            } else {
                self.cursor_offset..anchor
            }
        })
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let buffer_ref = self.buffer.read(cx);
        let content = buffer_ref.text.to_string();
        let is_dirty = buffer_ref.is_dirty;

        let filename = buffer_ref
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        let selection = self.selection_range();

        let sel_start = selection.as_ref().map(|r| r.start).unwrap_or(usize::MAX);
        let sel_end = selection.as_ref().map(|r| r.end).unwrap_or(usize::MAX);

        let mut text_container = div().flex().flex_col().w_full();
        let lines: Vec<&str> = content.split('\n').collect();

        let mut current_char_idx = 0;

        for (line_idx, line_str) in lines.into_iter().enumerate() {
            let line_char_count = line_str.chars().count();
            let line_end_idx = current_char_idx + line_char_count;

            let line_number_ui = div()
                .w(px(45.0))
                .flex_shrink_0()
                .text_color(rgb(0x6c7086))
                .flex()
                .justify_end()
                .pr(px(16.0))
                .child((line_idx + 1).to_string());

            let mut line_content = div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(24.0))
                .child(line_number_ui);

            if line_str.is_empty() {
                if self.cursor_offset == current_char_idx {
                    let cursor_ui = div()
                        .w(px(2.0))
                        .h(px(18.0))
                        .bg(rgb(0x89b4fa))
                        .ml(px(-1.0))
                        .mr(px(-1.0));
                    line_content = line_content.child(cursor_ui);
                } else if current_char_idx >= sel_start && current_char_idx < sel_end {
                    line_content =
                        line_content.child(div().w(px(8.0)).h(px(18.0)).bg(rgb(0x45475a)));
                } else {
                    line_content = line_content.child(" ");
                }
            } else {
                let mut split_points = vec![0, line_char_count];

                if self.cursor_offset > current_char_idx && self.cursor_offset < line_end_idx {
                    split_points.push(self.cursor_offset - current_char_idx);
                }
                if sel_start > current_char_idx && sel_start < line_end_idx {
                    split_points.push(sel_start - current_char_idx);
                }
                if sel_end > current_char_idx && sel_end < line_end_idx {
                    split_points.push(sel_end - current_char_idx);
                }

                split_points.sort();
                split_points.dedup();

                for window in split_points.windows(2) {
                    let start_char = window[0];
                    let end_char = window[1];
                    let global_start = current_char_idx + start_char;

                    if global_start == self.cursor_offset {
                        let cursor_ui = div()
                            .w(px(2.0))
                            .h(px(18.0))
                            .bg(rgb(0x89b4fa))
                            .ml(px(-1.0))
                            .mr(px(-1.0));
                        line_content = line_content.child(cursor_ui);
                    }

                    let byte_start = line_str
                        .char_indices()
                        .nth(start_char)
                        .map(|(b, _)| b)
                        .unwrap_or(line_str.len());
                    let byte_end = line_str
                        .char_indices()
                        .nth(end_char)
                        .map(|(b, _)| b)
                        .unwrap_or(line_str.len());
                    let segment_text = &line_str[byte_start..byte_end];

                    let is_selected = global_start >= sel_start && global_start < sel_end;
                    let mut text_ui = div().child(segment_text.to_string());

                    if is_selected {
                        text_ui = text_ui.bg(rgb(0x45475a));
                    }

                    line_content = line_content.child(text_ui);
                }

                if self.cursor_offset == line_end_idx {
                    let cursor_ui = div()
                        .w(px(2.0))
                        .h(px(18.0))
                        .bg(rgb(0x89b4fa))
                        .ml(px(-1.0))
                        .mr(px(-1.0));
                    line_content = line_content.child(cursor_ui);
                }
            }

            text_container = text_container.child(line_content);

            current_char_idx += line_char_count + 1;
        }

        div()
            .id("editor_main")
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            .key_context("Editor")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::handle_save))
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_click(cx.listener(|this, _event, window, _cx| {
                window.focus(&this.focus_handle);
            }))
            .child(div().p_2().bg(rgb(0x181825)).text_sm().child(format!(
                "{} {}",
                filename,
                if is_dirty { "*" } else { "" }
            )))
            .child(
                div()
                    .id("scrollable_text_area")
                    .flex_1()
                    .w_full()
                    .overflow_y_scroll()
                    .py_4()
                    .child(text_container),
            )
    }
}
