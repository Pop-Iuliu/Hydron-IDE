use crate::fs;
use anyhow::Result;
use ropey::Rope;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum EditAction {
    Insert { offset: usize, text: String },
    Remove { offset: usize, text: String },
}

/// modelul central
pub struct Buffer {
    pub text: Rope,
    pub file_path: Option<PathBuf>,
    pub is_dirty: bool,
    pub undo_stack: Vec<EditAction>,
    pub redo_stack: Vec<EditAction>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            text: Rope::new(),
            file_path: None,
            is_dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// load from disk a buffer
    pub fn open(path: PathBuf) -> Result<Self> {
        let content = fs::read_file_content(&path)?;
        Ok(Self {
            text: Rope::from_str(&content),
            file_path: Some(path),
            is_dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    /// save the buffer (ctrl + S)
    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = &self.file_path {
            // rope -> string
            let content = self.text.to_string();
            fs::write_file_content(path, &content)?;
            self.is_dirty = false;
        }
        Ok(())
    }

    pub fn insert(&mut self, char_idx: usize, text: &str) {
        self.text.insert(char_idx, text);
        self.is_dirty = true;

        self.undo_stack.push(EditAction::Insert {
            offset: char_idx,
            text: text.to_string(),
        });
        self.redo_stack.clear();
    }

    /// delete on an interval [a, b]
    pub fn remove(&mut self, char_range: std::ops::Range<usize>) -> String {
        let offset = char_range.start;
        let removed_text = self.text.slice(char_range.clone()).to_string();
        self.text.remove(char_range);
        self.is_dirty = true;

        self.undo_stack.push(EditAction::Remove {
            offset,
            text: removed_text.clone(),
        });
        self.redo_stack.clear();

        removed_text
    }

    pub fn undo(&mut self) -> Option<usize> {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                EditAction::Insert { offset, text } => {
                    let len = text.chars().count();

                    self.text.remove(offset..offset + len);

                    self.redo_stack.push(EditAction::Insert { offset, text });
                    self.is_dirty = true;
                    return Some(offset);
                }
                EditAction::Remove { offset, text } => {
                    self.text.insert(offset, &text);

                    self.redo_stack.push(EditAction::Remove {
                        offset,
                        text: text.clone(),
                    });
                    self.is_dirty = true;
                    return Some(offset + text.chars().count());
                }
            }
        }
        None
    }
}
