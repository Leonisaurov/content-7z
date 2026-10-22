//! Directory tree built from the flat list of archive entries.

use crate::archive::entry::ArchiveEntry;

/// The kind of a node of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Dir,
}

/// A node of the tree.
#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub kind: NodeKind,
    pub size: u64,
    pub encrypted: bool,
    pub children: Vec<Node>,
}

impl Node {
    /// Creates an empty directory node.
    pub fn dir(name: &str) -> Self {
        Self {
            name: name.to_string(),
            kind: NodeKind::Dir,
            size: 0,
            encrypted: false,
            children: Vec::new(),
        }
    }

    /// Whether the node is a directory.
    pub fn is_dir(&self) -> bool {
        self.kind == NodeKind::Dir
    }

    /// Looks a direct child up by name.
    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|child| child.name == name)
    }
}

/// Builds the directory tree of an archive.
///
/// The tree is built in `O(n log n)`: the entries are sorted once and grouped
/// afterwards, instead of walking the tree for every single entry.
pub fn build_tree(entries: &[ArchiveEntry]) -> Node {
    let mut items: Vec<&ArchiveEntry> = entries
        .iter()
        .filter(|entry| !entry.components.is_empty())
        .collect();

    items.sort_by(|left, right| left.components.cmp(&right.components));

    let mut root = Node::dir("");
    root.children = build_level(&items, 0);
    root
}

/// Builds the children of one level of the tree.
fn build_level(items: &[&ArchiveEntry], depth: usize) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut index = 0usize;

    while index < items.len() {
        // The entries that end on a previous level are already handled.
        while index < items.len() && items[index].components.len() <= depth {
            index += 1;
        }
        if index >= items.len() {
            break;
        }

        let name = items[index].components[depth].clone();
        let start = index;
        while index < items.len()
            && items[index].components.len() > depth
            && items[index].components[depth] == name
        {
            index += 1;
        }
        let group = &items[start..index];

        // The entry that describes the node itself is the one without any
        // remaining component.
        let leaf = group
            .iter()
            .find(|entry| entry.components.len() == depth + 1);
        let has_children = group.iter().any(|entry| entry.components.len() > depth + 1);

        let children = if has_children {
            build_level(group, depth + 1)
        } else {
            Vec::new()
        };

        let is_dir = leaf.map(|entry| entry.is_dir).unwrap_or(false) || has_children;

        nodes.push(Node {
            name,
            kind: if is_dir {
                NodeKind::Dir
            } else {
                NodeKind::File
            },
            size: leaf.map(|entry| entry.size).unwrap_or(0),
            encrypted: leaf.map(|entry| entry.encrypted).unwrap_or(false),
            children,
        });
    }

    nodes
}

/// Sorts the children of a node: directories first, then by name.
pub fn sort_children(node: &mut Node) {
    node.children
        .sort_by(|left, right| match (left.is_dir(), right.is_dir()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        });

    for child in &mut node.children {
        sort_children(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, is_dir: bool) -> ArchiveEntry {
        ArchiveEntry::new(path, is_dir, 0, false).unwrap()
    }

    #[test]
    fn builds_nested_tree() {
        let entries = vec![
            entry("b.txt", false),
            entry("a/c.txt", false),
            entry("a/d/e.txt", false),
            entry("a", true),
        ];

        let tree = build_tree(&entries);
        assert_eq!(tree.children.len(), 2);

        let a = tree.child("a").unwrap();
        assert!(a.is_dir());
        assert_eq!(a.children.len(), 2);
        assert!(a.child("d").unwrap().is_dir());
        assert_eq!(a.child("d").unwrap().children.len(), 1);
    }

    #[test]
    fn creates_missing_directories() {
        let entries = vec![entry("a/b/c.txt", false)];
        let tree = build_tree(&entries);
        let a = tree.child("a").unwrap();
        assert!(a.is_dir());
        assert!(a.child("b").unwrap().is_dir());
    }

    #[test]
    fn sorts_directories_first() {
        let entries = vec![
            entry("z.txt", false),
            entry("m.txt", false),
            entry("b_dir", true),
        ];

        let mut tree = build_tree(&entries);
        sort_children(&mut tree);

        let names: Vec<&str> = tree
            .children
            .iter()
            .map(|child| child.name.as_str())
            .collect();
        assert_eq!(names, vec!["b_dir", "m.txt", "z.txt"]);
    }

    #[test]
    fn empty_archive_has_no_children() {
        let tree = build_tree(&[]);
        assert!(tree.children.is_empty());
    }
}
