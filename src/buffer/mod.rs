use crate::fs;
use anyhow::Result;
use ropey::Rope;
use std::path::PathBuf;

/// modelul central
pub struct Buffer {
    pub text: Rope,
    pub file_path: Option<PathBuf>,
    pub is_dirty: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            text: Rope::new(),
            file_path: None,
            is_dirty: false,
        }
    }

    /// load from disk a buffer
    pub fn open(path: PathBuf) -> Result<Self> {
        let content = fs::read_file_content(&path)?;
        Ok(Self {
            text: Rope::from_str(&content),
            file_path: Some(path),
            is_dirty: false,
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
    }

    /// delete on an interval [a, b]
    pub fn remove(&mut self, char_range: std::ops::Range<usize>) {
        self.text.remove(char_range);
        self.is_dirty = true;
    }
}
