use ruff_diagnostics::Edit;
use ruff_source_file::SourceFile;
use ruff_text_size::{TextRange, TextSize};
/// Contains methods to parse Fortran code into a tree-sitter Tree and utilities to simplify the
/// navigation of a Tree.
use tree_sitter::{Node, TreeCursor};

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
    exceptions: Vec<String>, // TODO Use kind ids instead
}

impl<'a> Iterator for DepthFirstIteratorExcept<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // ignore exception list if we're at a depth of 0
        if (self.cursor.depth() == 0
            || !self
                .exceptions
                .contains(&self.cursor.node().kind().to_string()))
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

    /// Get the current indentation level of the node.
    fn indentation(&self, source_file: &SourceFile) -> String;

    /// Return the edit required to remove this node.
    fn edit_delete(&self, source_file: &SourceFile) -> Edit;

    /// Creates an edit that inserts `content`, replacing the whole node
    fn edit_replacement(&self, original: &SourceFile, content: String) -> Edit;

    /// Get the [`TextSize`] offset where this node starts
    fn start_textsize(&self) -> TextSize;

    /// Get the [`TextSize`] offset where this node ends
    fn end_textsize(&self) -> TextSize;

    /// Get the [`TextRange`] of this node
    fn textrange(&self) -> TextRange;
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
        DepthFirstIteratorExcept {
            cursor: self.walk(),
            exceptions: exceptions.into_iter().map(|x| x.to_string()).collect(),
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

    fn indentation(&self, source_file: &SourceFile) -> String {
        let src = source_file.to_source_code();
        let start_byte = self.start_textsize();
        let start_index = src.line_index(start_byte);
        let start_line = src.line_start(start_index);
        let start_offset = start_byte - start_line;
        let line = src
            .slice(TextRange::new(start_line, start_line + start_offset))
            .to_string();
        line.chars().take_while(|&c| c.is_whitespace()).collect()
    }

    fn edit_delete(&self, source_file: &SourceFile) -> Edit {
        // If deletion results in an empty line (or multiple), remove it
        // TODO handle case where removal should also remove a preceding comma
        let src = source_file.to_source_code();
        let start_byte = self.start_textsize();
        let end_byte = self.end_textsize();
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

    fn edit_replacement(&self, original: &SourceFile, content: String) -> Edit {
        // The node might include the newline as part of the
        // end-of-statement child, so don't include trailing
        // whitespace in the replacement
        let text = self.to_text(original.source_text()).unwrap();
        let len = text.trim().len();
        let start = self.start_textsize();
        let end = start + TextSize::try_from(len).unwrap();

        Edit::replacement(content, start, end)
    }

    fn start_textsize(&self) -> TextSize {
        TextSize::try_from(self.start_byte()).unwrap()
    }

    fn end_textsize(&self) -> TextSize {
        TextSize::try_from(self.end_byte()).unwrap()
    }

    fn textrange(&self) -> TextRange {
        TextRange::new(self.start_textsize(), self.end_textsize())
    }
}

/// Strip line breaks from a string of Fortran code.
pub fn strip_line_breaks(src: &str) -> String {
    src.replace('&', "").replace('\n', " ").replace('\r', "")
}

/// Returns true if the type passed to it is number-like, and of a kind that can be modified using
/// kinds. 'double precision' and 'double complex' are not included.
pub fn dtype_is_plain_number(dtype: &str) -> bool {
    matches!(
        dtype.to_lowercase().as_str(),
        "integer" | "real" | "logical" | "complex"
    )
}

/// Returns `true` if `node` is a keyword argument with name `keyword`.
pub fn is_keyword_argument<S: AsRef<str>>(node: &Node, keyword: S, src: &str) -> bool {
    if node.kind() != "keyword_argument" {
        return false;
    }

    if let Some(kwarg) = node.child_by_field_name("name") {
        if let Some(name) = &kwarg.to_text(src) {
            return name.to_lowercase() == keyword.as_ref();
        }
    }
    false
}
