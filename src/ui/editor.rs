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
                if new_offset > 0 {
                    new_offset -= 1;
                }
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

        let mut display_content = content.clone();
        if self.cursor_offset <= display_content.chars().count() {
            let byte_idx = display_content
                .char_indices()
                .nth(self.cursor_offset)
                .map(|(i, _)| i)
                .unwrap_or(display_content.len());
            display_content.insert_str(byte_idx, "|");
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            .key_context("Editor")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::handle_save))
            .on_key_down(cx.listener(Self::handle_key_down))
            .child(div().p_2().bg(rgb(0x181825)).text_sm().child(format!(
                "{} {}",
                filename,
                if is_dirty { "*" } else { "" }
            )))
            .child(
                div().flex_1().p_4().child(display_content), // overlay the cursor gen
            )
    }
}
