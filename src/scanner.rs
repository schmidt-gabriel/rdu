use rayon::prelude::*;
use std::path::Path;
use walkdir::WalkDir;

/// A node in the directory tree.
#[derive(Debug, Clone)]
pub struct DirNode {
    pub name: String,
    pub size: u64, // total size in bytes (recursive)
    pub items: u64, // total items (recursive)
    pub children: Vec<DirNode>,
    pub is_dir: bool,
}

impl DirNode {
    pub fn file(name: String, size: u64) -> Self {
        Self {
            name,
            size,
            items: 1,
            children: vec![],
            is_dir: false,
        }
    }
}

pub struct Scanner;

impl Scanner {
    /// Recursively scan a path and return the root DirNode.
    pub fn scan(root: &str) -> DirNode {
        let path = Path::new(root);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root.to_string());

        Self::scan_dir(name, root)
    }

    fn scan_dir(name: String, path: &str) -> DirNode {
        // Collect immediate children (depth=1) using walkdir
        let mut subdirs: Vec<String> = vec![];
        let mut own_size: u64 = 0;
        let mut file_children: Vec<DirNode> = vec![];

        // Only iterate one level deep for direct children
        for entry in WalkDir::new(path)
            .min_depth(1)
            .max_depth(1)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            let entry_path = entry.path().to_string_lossy().to_string();
            let entry_name = entry.file_name().to_string_lossy().to_string();

            if entry.file_type().is_dir() {
                subdirs.push(entry_path);
            } else {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                own_size += size;
                file_children.push(DirNode::file(entry_name, size));
            }
        }

        // Recurse into subdirectories in parallel via rayon
        let dir_children: Vec<DirNode> = subdirs
            .par_iter()
            .map(|subpath| {
                let subname = Path::new(subpath)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| subpath.clone());
                Self::scan_dir(subname, subpath)
            })
            .collect();

        let dir_size: u64 = dir_children.iter().map(|c| c.size).sum();
        let total_size = own_size + dir_size;

        // Merge files and dirs into children
        let mut children: Vec<DirNode> = dir_children;
        children.extend(file_children);
        // Sort largest first
        children.sort_by(|a, b| b.size.cmp(&a.size));

        let total_items = 1 + children.iter().map(|c| c.items).sum::<u64>();

        DirNode {
            name,
            size: total_size,
            items: total_items,
            children,
            is_dir: true,
        }
    }
}

/// Format a byte count into a human-readable string.
pub fn fmt_size(bytes: u64) -> String {
    use humansize::{format_size, BINARY};
    format_size(bytes, BINARY)
}
