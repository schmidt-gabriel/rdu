use rayon::prelude::*;
use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::{Arc, Mutex};

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

        let root_dev = std::fs::symlink_metadata(path)
            .map(|m| m.dev())
            .unwrap_or(0);

        let visited = Arc::new(Mutex::new(HashSet::new()));

        Self::scan_dir(name, root, root_dev, visited)
    }

    fn scan_dir(
        name: String,
        path: &str,
        root_dev: u64,
        visited: Arc<Mutex<HashSet<(u64, u64)>>>,
    ) -> DirNode {
        let mut subdirs: Vec<String> = vec![];
        let mut own_size: u64 = 0;
        let mut file_children: Vec<DirNode> = vec![];

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let entry_name = entry.file_name().to_string_lossy().to_string();

                let Ok(meta) = std::fs::symlink_metadata(&entry_path) else { continue };

                // Skip if crossing mount point
                if meta.dev() != root_dev {
                    continue;
                }

                if meta.is_symlink() {
                    let size = meta.len();
                    own_size += size;
                    file_children.push(DirNode::file(entry_name, size));
                } else if meta.is_dir() {
                    subdirs.push(entry_path.to_string_lossy().to_string());
                } else {
                    // Use allocated blocks instead of logical length for sparse files
                    let mut size = meta.blocks() * 512;

                    // Deduplicate hard links
                    if meta.nlink() > 1 {
                        let mut visited_set = visited.lock().unwrap();
                        if !visited_set.insert((meta.dev(), meta.ino())) {
                            size = 0; // Already counted this file
                        }
                    }

                    own_size += size;
                    file_children.push(DirNode::file(entry_name, size));
                }
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
            Self::scan_dir(subname, subpath, root_dev, visited.clone())
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
