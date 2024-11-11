use anyhow::Context;
use lazy_static::lazy_static;
use ruff_diagnostics::Edit;
use ruff_source_file::SourceFile;
use ruff_text_size::{TextRange, TextSize};
/// Contains methods to parse Fortran code into a tree-sitter Tree and utilites to simplify the
/// navigation of a Tree.
use std::sync::Mutex;
use tree_sitter::{Language, Node, Parser, Tree, TreeCursor};

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

/// Access the language of the parser.
pub fn language() -> Language {
    PARSER.lock().unwrap().language().unwrap()
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

pub struct DepthFirstIteratorExcept<'a> {
    cursor: TreeCursor<'a>,
    exceptions: Vec<u16>,
}

impl<'a> Iterator for DepthFirstIteratorExcept<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // ignore exception list if we're at a depth of 0
        if (self.cursor.depth() == 0 || !self.exceptions.contains(&self.cursor.node().kind_id()))
            && self.cursor.goto_first_child()
        {
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

/// Adds some extra functionality to [`tree_sitter::Node`]
pub trait FortitudeNode<'tree> {
    /// Iterate over all nodes beneath the current node in a depth-first manner.
    fn descendants(&self) -> impl Iterator<Item = Node>;

    /// Iterate over all named nodes beneath the current node in a depth-first manner.
    fn named_descendants(&self) -> impl Iterator<Item = Node>;

    /// Iterate over all nodes beneath the current node in a depth-first manner, never going deeper
    /// than any node types in the exceptions list.
    fn descendants_except<'a, I>(&self, exceptions: I) -> impl Iterator<Item = Node>
    where
        I: IntoIterator<Item = &'a str>;

    /// Iterate over all nodes beneath the current node in a depth-first manner, never going deeper
    /// than any node types in the exceptions list.
    fn named_descendants_except<'a, I>(&self, exceptions: I) -> impl Iterator<Item = Node>
    where
        I: IntoIterator<Item = &'a str>;

    /// Iterate over all nodes above the current node.
    fn ancestors(&self) -> impl Iterator<Item = Node>;

    /// Get the first child with a given name. Returns None if not found.
    fn child_with_name(&self, name: &str) -> Option<Node>;

    /// Convert a node to text, collapsing any raised errors to None.
    fn to_text<'a>(&self, src: &'a str) -> Option<&'a str>;

    /// Given a variable declaration or function statement, return its type if it's an intrinsic type,
    /// or None otherwise.
    fn parse_intrinsic_type(&self) -> Option<String>;

    /// Return the edit required to remove this node.
    fn edit_delete(&self, source_file: &SourceFile) -> Edit;
}

impl FortitudeNode<'_> for Node<'_> {
    fn descendants(&self) -> impl Iterator<Item = Node> {
        DepthFirstIterator {
            cursor: self.walk(),
        }
    }

    fn named_descendants(&self) -> impl Iterator<Item = Node> {
        self.descendants().filter(|&x| x.is_named())
    }

    fn descendants_except<'tree, I>(&self, exceptions: I) -> impl Iterator<Item = Node>
    where
        I: IntoIterator<Item = &'tree str>,
    {
        let lang = language();
        let exception_ids: Vec<_> = exceptions
            .into_iter()
            .map(|x| lang.id_for_node_kind(x, true))
            .collect();
        DepthFirstIteratorExcept {
            cursor: self.walk(),
            exceptions: exception_ids,
        }
    }

    fn named_descendants_except<'a, I>(&self, exceptions: I) -> impl Iterator<Item = Node>
    where
        I: IntoIterator<Item = &'a str>,
    {
        self.descendants_except(exceptions)
            .filter(|&x| x.is_named())
    }

    fn ancestors(&self) -> impl Iterator<Item = Node> {
        AncestorsIterator { node: *self }
    }

    fn child_with_name(&self, name: &str) -> Option<Self> {
        self.named_children(&mut self.walk())
            .find(|x| x.kind() == name)
    }

    fn to_text<'a>(&self, src: &'a str) -> Option<&'a str> {
        self.utf8_text(src.as_bytes()).ok()
    }

    fn parse_intrinsic_type(&self) -> Option<String> {
        if let Some(child) = self.child_with_name("intrinsic_type") {
            let grandchild = child.child(0)?;
            return Some(grandchild.kind().to_string());
        }
        None
    }

    fn edit_delete(&self, source_file: &SourceFile) -> Edit {
        // If deletion results in an empty line (or multiple), remove it
        // TODO handle case where removal should also remove a preceding comma
        let src = source_file.to_source_code();
        let start_byte = TextSize::try_from(self.start_byte()).unwrap();
        let end_byte = TextSize::try_from(self.end_byte()).unwrap();
        let start_index = src.line_index(start_byte);
        let end_index = src.line_index(end_byte);
        let start_line = src.line_start(start_index);
        let end_line = src.line_end(end_index);
        let mut text = src.slice(TextRange::new(start_line, end_line)).to_string();
        let start_offset = start_byte - start_line;
        let end_offset = end_byte - start_line;
        text.replace_range(usize::from(start_offset)..usize::from(end_offset), "");
        if text.trim().is_empty() {
            Edit::range_deletion(TextRange::new(start_line, end_line))
        } else {
            Edit::range_deletion(TextRange::new(start_byte, end_byte))
        }
    }
}

/// Strip line breaks from a string of Fortran code.
pub fn strip_line_breaks(src: &str) -> String {
    src.replace('&', "").replace('\n', " ")
}

/// Returns true if the type passed to it is number-like, and of a kind that can be modified using
/// kinds. 'double precision' and 'double complex' are not included.
pub fn dtype_is_plain_number(dtype: &str) -> bool {
    matches!(
        dtype.to_lowercase().as_str(),
        "integer" | "real" | "logical" | "complex"
    )
}
