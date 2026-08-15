use crate::buffer::Buffer;
use gpui::*;

actions!(editor, [Save]);

pub struct EditorView {
    pub buffer: Entity<Buffer>,
    focus_handle: FocusHandle,
    cursor_offset: usize,
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

        let mut cursor_line = 0;
        let mut cursor_col = 0;

        for (i, c) in content.char_indices() {
            if i == self.cursor_offset {
                break;
            }
            if c == '\n' {
                cursor_line += 1;
                cursor_col = 0;
            } else {
                cursor_col += 1;
            }
        }

        let mut text_container = div().flex().flex_col().w_full();

        let lines: Vec<&str> = content.split('\n').collect();

        for (line_idx, line_str) in lines.into_iter().enumerate() {
            let mut line_element = div().flex().flex_row().items_center().h(px(24.0));

            if line_idx == cursor_line {
                let byte_idx = line_str
                    .char_indices()
                    .nth(cursor_col)
                    .map(|(i, _)| i)
                    .unwrap_or(line_str.len());
                let (before, after) = line_str.split_at(byte_idx);

                let cursor_ui = div()
                    .w(px(2.0))
                    .h(px(18.0))
                    .bg(rgb(0x89b4fa))
                    .ml(px(-1.0))
                    .mr(px(-1.0));

                line_element = line_element
                    .child(before.to_string())
                    .child(cursor_ui)
                    .child(after.to_string());
            } else {
                let display_text = if line_str.is_empty() { " " } else { line_str };
                line_element = line_element.child(display_text.to_string());
            }

            text_container = text_container.child(line_element);
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
            .child(div().flex_1().p_4().child(text_container))
    }
}
