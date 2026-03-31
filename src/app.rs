use crate::scanner::{DirNode, Scanner};
use ratatui::widgets::ListState;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    SizeDesc,
    SizeAsc,
    NameAsc,
    NameDesc,
}

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

    /// State of the list widget (keeps track of scroll offset).
    pub list_state: ListState,

    /// Receive completed scan results from the background thread.
    scan_rx: Option<Receiver<DirNode>>,

    /// True while the background scan thread is running.
    pub scanning: bool,

    /// Show the help overlay.
    pub show_help: bool,

    /// Current sort mode.
    pub sort_mode: SortMode,

    /// Set of paths currently marked for deletion.
    pub marked_items: HashMap<String, u64>, // path -> size

    /// Show the deletion confirmation overlay.
    pub show_delete_confirm: bool,

    /// Whether deletions are disabled.
    pub no_delete: bool,

    /// Is the search input currently active?
    pub is_searching: bool,

    /// Current search query.
    pub search_query: String,

    /// Paths (indices) of nodes matching the search query.
    pub search_results: Vec<Vec<usize>>,
}

impl App {
    pub fn new(root_path: String, no_delete: bool) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            root_path,
            tree: None,
            current_node_path: vec![],
            selected: 0,
            list_state,
            scan_rx: None,
            scanning: false,
            show_help: false,
            sort_mode: SortMode::SizeDesc,
            marked_items: HashMap::new(),
            show_delete_confirm: false,
            no_delete,
            is_searching: false,
            search_query: String::new(),
            search_results: Vec::new(),
        }
    }

    /// Spawn a background scan. Resets current state.
    pub fn start_scan(&mut self) {
        self.tree = None;
        self.current_node_path = vec![];
        self.selected = 0;
        self.list_state.select(Some(0));
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
            if let Ok(mut node) = rx.try_recv() {
                Self::sort_tree(&mut node, self.sort_mode);
                self.tree = Some(node);
                self.scanning = false;
                self.scan_rx = None;
                if !self.search_query.is_empty() {
                    self.perform_search();
                } else {
                    self.list_state.select(Some(self.selected));
                }
            }
        }
    }

    fn sort_tree(node: &mut DirNode, mode: SortMode) {
        match mode {
            SortMode::SizeDesc => node.children.sort_by(|a, b| b.size.cmp(&a.size)),
            SortMode::SizeAsc => node.children.sort_by(|a, b| a.size.cmp(&b.size)),
            SortMode::NameAsc => node.children.sort_by(|a, b| a.name.cmp(&b.name)),
            SortMode::NameDesc => node.children.sort_by(|a, b| b.name.cmp(&a.name)),
        }
        for child in &mut node.children {
            Self::sort_tree(child, mode);
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

    /// Children of the current node (already sorted by current sort_mode).
    pub fn current_children(&self) -> &[DirNode] {
        if let Some(node) = self.current_node() {
            &node.children
        } else {
            &[]
        }
    }

    /// Total items in the currently active view.
    pub fn current_item_count(&self) -> usize {
        if !self.search_query.is_empty() {
            self.search_results.len()
        } else {
            self.current_children().len()
        }
    }

    /// Get a node given its full path indices.
    pub fn get_node(&self, path: &[usize]) -> Option<&DirNode> {
        let mut node = self.tree.as_ref()?;
        for &idx in path {
            node = node.children.get(idx)?;
        }
        Some(node)
    }

    /// Retrieve the fully resolved string path for an index sequence.
    pub fn get_path_display(&self, path: &[usize]) -> String {
        let Some(root) = self.tree.as_ref() else { return self.root_path.clone() };
        let mut s = root.name.clone();
        let mut node = root;
        for &idx in path {
            if let Some(child) = node.children.get(idx) {
                if !s.ends_with('/') && !s.is_empty() {
                    s.push('/');
                }
                s.push_str(&child.name);
                node = child;
            } else {
                break;
            }
        }
        s
    }

    /// Returns a reference to the currently selected node in the active view.
    pub fn get_selected_node(&self) -> Option<&DirNode> {
        if !self.search_query.is_empty() {
            if let Some(path) = self.search_results.get(self.selected) {
                return self.get_node(path);
            }
        } else {
            return self.current_children().get(self.selected);
        }
        None
    }

    pub fn select_next(&mut self) {
        let count = self.current_item_count();
        if count == 0 { return; }
        self.selected = (self.selected + 1).min(count - 1);
        self.list_state.select(Some(self.selected));
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
        }
    }

    /// Drill into the selected directory.
    pub fn enter_selected(&mut self) {
        if !self.search_query.is_empty() {
            if let Some(path) = self.search_results.get(self.selected) {
                if let Some(node) = self.get_node(path) {
                    let mut new_path = path.clone();
                    if !node.is_dir {
                        new_path.pop(); // Go to parent directory if it's a file
                    }
                    self.current_node_path = new_path;
                    self.search_query.clear();
                    self.search_results.clear();
                    self.is_searching = false;
                    self.selected = 0;
                    self.list_state.select(Some(0));
                }
            }
            return;
        }

        let children = self.current_children();
        let Some(child) = children.get(self.selected) else { return };

        // Only enter directories
        if child.children.is_empty() { return; }

        self.current_node_path.push(self.selected);
        self.selected = 0;
        self.list_state.select(Some(0));
    }

    /// Navigate to the parent directory.
    pub fn go_up(&mut self) {
        if self.current_node_path.pop().is_some() {
            self.selected = 0;
            self.list_state.select(Some(0));
        }
    }

    /// The display path of the current node.
    pub fn current_path_display(&self) -> String {
        let Some(root) = self.tree.as_ref() else { return self.root_path.clone() };
        let mut path = root.name.clone();
        let mut node = root;
        for &idx in &self.current_node_path {
            if let Some(child) = node.children.get(idx) {
                if !path.ends_with('/') && !path.is_empty() {
                    path.push('/');
                }
                path.push_str(&child.name);
                node = child;
            }
        }
        path
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::SizeDesc => SortMode::SizeAsc,
            SortMode::SizeAsc => SortMode::NameAsc,
            SortMode::NameAsc => SortMode::NameDesc,
            SortMode::NameDesc => SortMode::SizeDesc,
        };
        if let Some(mut root) = self.tree.take() {
            Self::sort_tree(&mut root, self.sort_mode);
            self.tree = Some(root);
        }
        if !self.search_query.is_empty() {
            self.perform_search();
        } else {
            self.selected = 0;
            self.list_state.select(Some(0));
        }
    }

    pub fn handle_click(&mut self, row: u16, term_height: u16) {
        let list_start = 1;
        let list_end = term_height.saturating_sub(3); // Subtract bottom border and status bar

        if row >= list_start && row <= list_end {
            let offset = self.list_state.offset();
            let clicked_index = offset + (row - list_start) as usize;

            if clicked_index < self.current_item_count() {
                if self.selected == clicked_index {
                    self.enter_selected(); // Double-click equivalent to drill down
                } else {
                    self.selected = clicked_index;
                    self.list_state.select(Some(self.selected));
                }
            }
        }
    }

    pub fn get_path_of(&self, index: usize) -> Option<String> {
        if !self.search_query.is_empty() {
            let path = self.search_results.get(index)?;
            return Some(self.get_path_display(path));
        }

        let mut path = self.current_path_display();
        if let Some(child) = self.current_children().get(index) {
            if !path.ends_with('/') && !path.is_empty() {
                path.push('/');
            }
            path.push_str(&child.name);
            Some(path)
        } else {
            None
        }
    }

    pub fn toggle_mark(&mut self) {
        if self.no_delete {
            return;
        }

        if let Some(path) = self.get_path_of(self.selected) {
            if self.marked_items.contains_key(&path) {
                self.marked_items.remove(&path);
            } else if let Some(node) = self.get_selected_node() {
                self.marked_items.insert(path, node.size);
            }
        }
    }

    pub fn prompt_delete(&mut self) {
        if self.no_delete {
            return;
        }

        // If nothing is marked, implicitly mark the currently highlighted item
        if self.marked_items.is_empty() {
            if let Some(path) = self.get_path_of(self.selected) {
                if let Some(node) = self.get_selected_node() {
                    self.marked_items.insert(path, node.size);
                }
            }
        }
        if !self.marked_items.is_empty() {
            self.show_delete_confirm = true;
        }
    }

    pub fn delete_marked(&mut self) {
        for path in self.marked_items.keys() {
            let p = std::path::Path::new(path);
            if p.is_dir() {
                let _ = std::fs::remove_dir_all(p);
            } else {
                let _ = std::fs::remove_file(p);
            }
        }
        self.marked_items.clear();
        self.show_delete_confirm = false;
        self.start_scan();
    }

    /// Total size of all marked items.
    pub fn marked_size(&self) -> u64 {
        self.marked_items.values().sum()
    }

    pub fn perform_search(&mut self) {
        self.search_results.clear();
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            return;
        }
        if let Some(root) = &self.tree {
            let mut results = Vec::new();
            Self::search_tree_recursive(root, &query, &mut Vec::new(), &mut results);
            self.search_results = results;
        }
        self.selected = 0;
        self.list_state.select(Some(0));
    }

    fn search_tree_recursive(
        node: &DirNode,
        query: &str,
        current_path: &mut Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        for (i, child) in node.children.iter().enumerate() {
            current_path.push(i);
            if child.name.to_lowercase().contains(query) {
                results.push(current_path.clone());
            }
            Self::search_tree_recursive(child, query, current_path, results);
            current_path.pop();
        }
    }
}
