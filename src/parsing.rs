use tree_sitter::Node;
/// Utilities to simplify parsing tree-sitter structures.

/// Convert a node to text, collapsing any raised errors to None.
pub fn to_text<'a>(node: &'a Node<'a>, src: &'a str) -> Option<&'a str> {
    let result = node.utf8_text(src.as_bytes()).ok()?;
    Some(result)
}

/// Strip line breaks from a string of Fortran code.
pub fn strip_line_breaks(src: &str) -> String {
    src.replace("&", "").replace("\n", " ")
}

/// Given a variable declaration or function statement, return its type if it's an intrinsic type,
/// or None otherwise.
pub fn intrinsic_type(node: &Node) -> Option<String> {
    // TODO This is copied from literal kinds, need to make a parsing utils file
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "intrinsic_type" {
            let grandchild = child.child(0)?;
            return Some(grandchild.kind().to_string());
        }
    }
    None
}

/// Returns true if the type passed to it is number-like.
pub fn dtype_is_number(dtype: &str) -> bool {
    matches!(dtype, "integer" | "real" | "logical" | "complex")
}
