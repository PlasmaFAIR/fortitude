use anyhow::Context;
use lazy_static::lazy_static;
/// Contains methods to parse Fortran code into a tree-sitter Tree and utilites to simplify the
/// navigation of a Tree.
use std::sync::Mutex;
use tree_sitter::{Node, Parser, Tree, TreeCursor};

lazy_static! {
    static ref PARSER: Mutex<Parser> = {
        let parser = Mutex::new(Parser::new());
        parser
            .lock()
            .unwrap()
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .expect("Error loading Fortran grammar");
        parser
    };
}

/// Parse a Fortran string and return the root note to the AST.
pub fn parse<S: AsRef<str>>(source: S) -> anyhow::Result<Tree> {
    PARSER
        .lock()
        .unwrap()
        .parse(source.as_ref(), None)
        .context("Failed to parse")
}

pub struct DepthFirstIterator<'a> {
    cursor: TreeCursor<'a>,
}

impl<'a> Iterator for DepthFirstIterator<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.goto_first_child() {
            return Some(self.cursor.node());
        }
        if self.cursor.goto_next_sibling() {
            return Some(self.cursor.node());
        }
        while self.cursor.goto_parent() {
            if self.cursor.goto_next_sibling() {
                return Some(self.cursor.node());
            }
        }
        None
    }
}

/// Iterate over all nodes beneath the current node in a depth-first manner.
pub fn descendants<'a>(node: &'a Node) -> impl Iterator<Item = Node<'a>> {
    DepthFirstIterator {
        cursor: node.walk(),
    }
}

/// Iterate over all named nodes beneath the current node in a depth-first manner.
pub fn named_descendants<'a>(node: &'a Node) -> impl Iterator<Item = Node<'a>> {
    descendants(node).filter(|&x| x.is_named())
}

pub struct AncestorsIterator<'a> {
    node: Node<'a>,
}

impl<'a> Iterator for AncestorsIterator<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.node = self.node.parent()?;
        Some(self.node)
    }
}

// Iterate over all nodes above the current node.
pub fn ancestors<'a>(node: &'a Node) -> impl Iterator<Item = Node<'a>> {
    AncestorsIterator { node: *node }
}

/// Get the first child with a given name. Returns None if not found.
pub fn child_with_name<'a>(node: &'a Node, name: &'a str) -> Option<Node<'a>> {
    node.named_children(&mut node.walk())
        .find(|x| x.kind() == name)
}

// Convert a node to text, collapsing any raised errors to None.
pub fn to_text<'a>(node: &Node, src: &'a str) -> Option<&'a str> {
    node.utf8_text(src.as_bytes()).ok()
}

/// Strip line breaks from a string of Fortran code.
pub fn strip_line_breaks(src: &str) -> String {
    src.replace('&', "").replace('\n', " ")
}

/// Given a variable declaration or function statement, return its type if it's an intrinsic type,
/// or None otherwise.
pub fn parse_intrinsic_type(node: &Node) -> Option<String> {
    if let Some(child) = child_with_name(node, "intrinsic_type") {
        let grandchild = child.child(0)?;
        return Some(grandchild.kind().to_string());
    }
    None
}

/// Returns true if the type passed to it is number-like, and of a kind that can be modified using
/// kinds. 'double precision' and 'double complex' are not included.
pub fn dtype_is_plain_number(dtype: &str) -> bool {
    matches!(
        dtype.to_lowercase().as_str(),
        "integer" | "real" | "logical" | "complex"
    )
}
