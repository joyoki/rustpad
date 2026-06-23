use std::path::PathBuf;

use eframe::egui;

/// File tree node type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
}

/// A node in the file tree.
#[derive(Debug, Clone)]
pub struct FileTreeNode {
    pub path: PathBuf,
    pub name: String,
    pub node_type: NodeType,
    pub expanded: bool,
    pub children: Vec<FileTreeNode>,
}

impl FileTreeNode {
    pub fn new(path: PathBuf, node_type: NodeType) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        Self {
            path,
            name,
            node_type,
            expanded: false,
            children: Vec::new(),
        }
    }

    /// Check if this is a directory.
    pub fn is_dir(&self) -> bool {
        self.node_type == NodeType::Directory
    }
}

/// File tree state.
#[derive(Debug)]
pub struct FileTree {
    pub root: Option<FileTreeNode>,
    pub selected: Option<PathBuf>,
    pub show_hidden: bool,
}

impl FileTree {
    pub fn new() -> Self {
        Self {
            root: None,
            selected: None,
            show_hidden: false,
        }
    }

    /// Load file tree from a root directory.
    pub fn load(&mut self, root_path: &PathBuf) {
        if root_path.exists() && root_path.is_dir() {
            self.root = Some(self.build_tree(root_path, 0));
        }
    }

    /// Recursively build the file tree.
    fn build_tree(&self, path: &PathBuf, depth: usize) -> FileTreeNode {
        let mut node = FileTreeNode::new(path.clone(), NodeType::Directory);

        if depth > 5 {
            return node;
        }

        if let Ok(entries) = std::fs::read_dir(path) {
            let mut entries: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    self.show_hidden || !name.starts_with('.')
                })
                .collect();

            entries.sort_by(|a, b| {
                let a_is_dir = a.path().is_dir();
                let b_is_dir = b.path().is_dir();
                if a_is_dir && !b_is_dir {
                    std::cmp::Ordering::Less
                } else if !a_is_dir && b_is_dir {
                    std::cmp::Ordering::Greater
                } else {
                    a.file_name().cmp(&b.file_name())
                }
            });

            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    node.children.push(self.build_tree(&path, depth + 1));
                } else {
                    node.children.push(FileTreeNode::new(path, NodeType::File));
                }
            }
        }

        node
    }

    /// Toggle expansion of a directory node.
    pub fn toggle_expansion(&mut self, path: &PathBuf) {
        if let Some(ref mut root) = self.root {
            Self::toggle_node_expansion(root, path);
        }
    }

    fn toggle_node_expansion(node: &mut FileTreeNode, path: &PathBuf) {
        if node.path == *path {
            node.expanded = !node.expanded;
            return;
        }
        for child in &mut node.children {
            Self::toggle_node_expansion(child, path);
        }
    }

    /// Select a file.
    pub fn select(&mut self, path: PathBuf) {
        self.selected = Some(path);
    }
}

impl Default for FileTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Show the file tree panel in the UI.
pub fn show(app: &mut crate::app::RustpadApp, ctx: &egui::Context) {
    if !app.show_sidebar {
        return;
    }

    egui::SidePanel::left("file_tree_panel")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("File Explorer");
            ui.separator();

            if let Some(workspace) = &app.workspace_root.clone() {
                if app.file_tree.root.is_none() {
                    app.file_tree.load(&workspace.clone());
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(ref root) = app.file_tree.root.clone() {
                        render_tree_node(ui, root, &mut app.file_tree, &mut app.tab_manager);
                    }
                });
            } else {
                ui.label("No workspace open");
                if ui.button("Open Folder...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        app.file_tree.load(&path);
                        app.workspace_root = Some(path);
                    }
                }
            }
        });
}

fn render_tree_node(
    ui: &mut egui::Ui,
    node: &FileTreeNode,
    tree: &mut FileTree,
    tab_manager: &mut crate::editor::TabManager,
) {
    let icon = if node.is_dir() {
        if node.expanded { "📂" } else { "📁" }
    } else {
        "📄"
    };

    let label = format!("{} {}", icon, node.name);
    let response = ui.selectable_label(tree.selected.as_ref() == Some(&node.path), &label);

    if response.clicked() {
        if node.is_dir() {
            tree.toggle_expansion(&node.path);
        } else {
            tree.select(node.path.clone());
            let _ = tab_manager.open_file(&node.path);
        }
    }

    if node.expanded {
        ui.indent(ui.id().with(&node.path), |ui| {
            for child in &node.children {
                render_tree_node(ui, child, tree, tab_manager);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_tree_new() {
        let tree = FileTree::new();
        assert!(tree.root.is_none());
        assert!(tree.selected.is_none());
    }

    #[test]
    fn test_file_tree_node() {
        let node = FileTreeNode::new(PathBuf::from("/test"), NodeType::Directory);
        assert!(node.is_dir());
        assert_eq!(node.name, "test");
    }
}
