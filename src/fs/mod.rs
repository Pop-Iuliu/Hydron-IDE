pub mod tree;

use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn read_file_content<P: AsRef<Path>>(path: P) -> Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

pub fn write_file_content<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    fs::write(path, content)?;
    Ok(())
}
