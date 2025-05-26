use crate::ast::{is_keyword_argument, FortitudeNode};
use ruff_source_file::SourceFile;
use tree_sitter::Node;

pub fn match_original_case(original: &str, new: &str) -> Option<String> {
    let first_ch = original.chars().next()?;

    if first_ch.is_lowercase() {
        Some(new.to_lowercase())
    } else {
        Some(new.to_uppercase())
    }
}

pub fn literal_as_io_unit<'a>(node: &'a Node, src: &SourceFile) -> Option<Node<'a>> {
    let unit = if let Some(unit) = node.child_with_name("unit_identifier") {
        unit.child(0)?
    } else {
        node.named_children(&mut node.walk())
            .find(|child| is_keyword_argument(child, "unit", src.source_text()))
            .map(|node| node.child_by_field_name("value"))??
    };

    if unit.kind() == "number_literal" {
        Some(unit)
    } else {
        None
    }
}
