use crate::scanner::{DirNode, Scanner};
use std::sync::mpsc::{self, Receiver};

/// The complete application state.
pub struct App {
    /// The path we were invoked with.
    pub root_path: String,

    /// The fully-built filesystem tree (None while scanning).
    pub tree: Option<DirNode>,

    /// Current node being viewed in the file list.
    pub current_node_path: Vec<usize>, // indices from root → current

    /// Selected row in the current file list.
    pub selected: usize,

    /// Receive completed scan results from the background thread.
    scan_rx: Option<Receiver<DirNode>>,

    /// True while the background scan thread is running.
    pub scanning: bool,

    /// Show the help overlay.
    pub show_help: bool,
}

impl App {
    pub fn new(root_path: String) -> Self {
        Self {
            root_path,
            tree: None,
            current_node_path: vec![],
            selected: 0,
            scan_rx: None,
            scanning: false,
            show_help: false,
        }
    }

    /// Spawn a background scan. Resets current state.
    pub fn start_scan(&mut self) {
        self.tree = None;
        self.current_node_path = vec![];
        self.selected = 0;
        self.scanning = true;

        let path = self.root_path.clone();
        let (tx, rx) = mpsc::channel();
        self.scan_rx = Some(rx);

        std::thread::spawn(move || {
            let node = Scanner::scan(&path);
            let _ = tx.send(node);
        });
    }

    /// Check if the background scan has finished; if so, store the result.
    pub fn poll_scan(&mut self) {
        if let Some(rx) = &self.scan_rx {
            if let Ok(node) = rx.try_recv() {
                self.tree = Some(node);
                self.scanning = false;
                self.scan_rx = None;
            }
        }
    }

    /// Returns a reference to the node currently displayed in the file list.
    pub fn current_node(&self) -> Option<&DirNode> {
        let root = self.tree.as_ref()?;
        let mut node = root;
        for &idx in &self.current_node_path {
            node = node.children.get(idx)?;
        }
        Some(node)
    }

    /// Sorted children of the current node (largest first).
    pub fn current_children(&self) -> Vec<&DirNode> {
        let Some(node) = self.current_node() else { return vec![] };
        let mut children: Vec<&DirNode> = node.children.iter().collect();
        children.sort_by(|a, b| b.size.cmp(&a.size));
        children
    }

    pub fn select_next(&mut self) {
        let count = self.current_children().len();
        if count == 0 { return; }
        self.selected = (self.selected + 1).min(count - 1);
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Drill into the selected directory.
    pub fn enter_selected(&mut self) {
        let children = self.current_children();
        let Some(child) = children.get(self.selected) else { return };

        // Only enter directories
        if child.children.is_empty() { return; }

        // Find the real (unsorted) index in the current node's children
        let Some(node) = self.current_node() else { return };
        let target_name = child.name.clone();
        if let Some(real_idx) = node.children.iter().position(|c| c.name == target_name) {
            self.current_node_path.push(real_idx);
            self.selected = 0;
        }
    }

    /// Navigate to the parent directory.
    pub fn go_up(&mut self) {
        if self.current_node_path.pop().is_some() {
            self.selected = 0;
        }
    }

    /// The display path of the current node.
    pub fn current_path_display(&self) -> String {
        let Some(root) = self.tree.as_ref() else { return self.root_path.clone() };
        let mut path = root.name.clone();
        let mut node = root;
        for &idx in &self.current_node_path {
            if let Some(child) = node.children.get(idx) {
                path = format!("{}/{}", path, child.name);
                node = child;
            }
        }
        path
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}
