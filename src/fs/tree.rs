use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
}

// Construieste file tree ul pe baza unui directory ~(find .)
// also, automat ignora ce e in .gitignore :)
pub fn build_file_tree(dir: &str) -> Vec<FileNode> {
    let mut nodes = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = path.is_dir();

            // !!!!!!! ATENTIE SA DAU REMOVE AICI (la un moment dat, nu acum :)
            if name.starts_with('.') || name == "target" {
                continue;
            }

            let children = if is_dir {
                Some(build_file_tree(path.to_str().unwrap()))
            } else {
                None
            };

            nodes.push(FileNode {
                path,
                name,
                is_dir,
                children,
            });
        }
    }

    // folderele apar primele, respectiv se afiseaza fisierele in ordine crescatoare lexicografica
    nodes.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    nodes
}
