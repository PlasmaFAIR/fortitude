use fortitude_sitter::Node;
use lazy_regex::regex_replace_all;

pub fn match_original_case(original: &str, new: &str) -> Option<String> {
    let first_ch = original.chars().next()?;

    if first_ch.is_lowercase() {
        Some(new.to_lowercase())
    } else {
        Some(new.to_uppercase())
    }
}

pub fn literal_as_io_unit<'a>(node: &'a Node) -> Option<Node<'a>> {
    let unit = if let Some(unit) = node.child_with_name("unit_identifier") {
        unit.child(0)?
    } else {
        node.kwarg_value("unit")?
    };

    if unit.kind() == "number_literal" {
        Some(unit)
    } else {
        None
    }
}

/// Get the text of a `number_literal`, removing any line continuations
/// (including any comments and blank lines) that split the literal, for
/// example:
///
/// ```f90
/// x = 1& ! comment
///   &0_8
/// ```
///
/// gives `10_8`
pub fn number_literal_text(node: &Node) -> String {
    regex_replace_all!(
        r"&[ \t]*(?:![^\r\n]*)?\r?\n(?:[ \t]*(?:![^\r\n]*)?\r?\n)*[ \t]*&?",
        node.text(),
        ""
    )
    .into_owned()
}
