//! Interface for generating fix edits from higher-level actions (e.g., "remove an argument").

use anyhow::{Result, anyhow};
use itertools::Itertools;
use lazy_regex::lazy_regex;
use ruff_diagnostics::Edit;
use ruff_source_file::{LineEnding, SourceFile};
use ruff_text_size::Ranged;
use tree_sitter::Node;

use crate::{
    ast::FortitudeNode,
    ast::types::VariableDeclaration,
    traits::{HasNode, TextRanged},
};

/// Return the [`Edit`] to delete the declaration of a variable
///
/// Removes the given variable name from a variable declaration statement,
/// taking care of:
/// - commas if there are multiple declarations in the same statement
/// - removing the whole statement if this is the only variable
pub(crate) fn remove_variable_decl(
    var: &Node,
    decl: &VariableDeclaration,
    src: &SourceFile,
) -> Result<Edit> {
    remove_from_comma_sep_stmt(
        var,
        decl.node(),
        &decl.names().iter().map(|name| *name.node()).collect_vec(),
        src,
    )
}

fn next_comma<'a>(item: Node<'a>) -> Result<Node<'a>> {
    let mut comma = item;
    while let Some(next) = comma.next_sibling() {
        if next.kind() == "," {
            return Ok(next);
        }
        comma = next;
    }
    Err(anyhow!("unable to find trailing comma for {item:?}"))
}

pub(crate) fn remove_from_comma_sep_stmt(
    item: &Node,
    stmt: &Node,
    children: &[impl TextRanged],
    src: &SourceFile,
) -> Result<Edit> {
    let (before, after): (Vec<_>, Vec<_>) = children
        .iter()
        .map(|name| name.textrange())
        .filter(|range| item.textrange() != *range)
        .partition(|range| range.start() < item.start_textsize());

    if !after.is_empty() {
        // Case 1: variable is _not_ the last node, so delete from the start of
        // the variable to the end of the subsequent comma
        let next = next_comma(*item)?;
        Ok(Edit::deletion(item.start_textsize(), next.end_textsize()))
    } else if let Some(previous) = before.iter().map(Ranged::end).max() {
        // Case 2: argument or keyword is the last node, so delete from the start of the
        // previous comma to the end of the argument.
        Ok(Edit::deletion(previous, item.end_textsize()))
    } else {
        // Case 3: variable is the only declaration
        Ok(stmt.edit_delete(src))
    }
}

pub(crate) fn add_attribute_to_var_decl(decl: &VariableDeclaration, attribute: &str) -> Edit {
    let start = decl
        .attributes()
        .last()
        .map(|attr| attr.node().end_textsize())
        .unwrap_or(decl.type_().node().end_textsize());

    let colon = if decl.has_colon() { "" } else { " ::" };
    Edit::insertion(format!(", {attribute}{colon}"), start)
}

/// Unindent and then indent each line, ignoring under-indented comments and
/// continuation lines.
///
/// This function will look at each non-empty line and determine the maximum
/// amount of whitespace that can be removed from all lines, and then add back
/// the given indentation. Fortran comments and continuation lines that are
/// "under-indented" will be ignored during this process:
///
/// ```
/// use fortitude_linter::fix::edits::redent;
/// use ruff_source_file::LineEnding;
///
/// assert_eq!(redent("
///     1st line
///       2nd line
///  !     comment
///     3rd line
/// ", "  ", LineEnding::Lf), "
///   1st line
///     2nd line
///  !     comment
///   3rd line
/// ");
/// ```
///
/// Adapted from `textwrap`
/// Copyright 2016 Martin Geisler
/// SPDX-License-Identifier: MIT
pub fn redent(s: &str, indentation: &str, line_ending: LineEnding) -> String {
    let mut prefix = "";
    let mut lines = s.lines();
    let comment_line = lazy_regex!(r"^\s*[&!]");

    // We first search for a non-empty line to find a prefix.
    for line in &mut lines {
        // Don't let comments or continuations set the prefix
        if comment_line.is_match(line) {
            continue;
        }

        let mut whitespace_idx = line.len();
        for (idx, ch) in line.char_indices() {
            if !ch.is_whitespace() {
                whitespace_idx = idx;
                break;
            }
        }

        // Check if the line had anything but whitespace
        if whitespace_idx < line.len() {
            prefix = &line[..whitespace_idx];
            break;
        }
    }

    // We then continue looking through the remaining lines to
    // possibly shorten the prefix.
    for line in &mut lines {
        // Don't let comments or continuations shorten the prefix
        if comment_line.is_match(line) {
            continue;
        }

        let mut whitespace_idx = line.len();
        for ((idx, a), b) in line.char_indices().zip(prefix.chars()) {
            if a != b {
                whitespace_idx = idx;
                break;
            }
        }

        // Check if the line had anything but whitespace and if we
        // have found a shorter prefix
        if whitespace_idx < line.len() && whitespace_idx < prefix.len() {
            prefix = &line[..whitespace_idx];
        }
    }

    // We now go over the lines a second time to build the result.
    let mut result = String::new();
    for line in s.lines() {
        if line.starts_with(prefix) && line.chars().any(|c| !c.is_whitespace()) {
            let (_, tail) = line.split_at(prefix.len());
            result.push_str(indentation);
            result.push_str(tail);
        } else if comment_line.is_match(line) {
            // Preserve under-indented comment and continuation lines
            result.push_str(line);
        }
        result.push_str(line_ending.as_str());
    }

    if result.ends_with('\n') && !s.ends_with('\n') {
        let new_len = result.len() - line_ending.len();
        result.truncate(new_len);
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::symbol_table::SymbolTable;

    use super::*;
    use anyhow::{Context, Result};
    use ruff_source_file::SourceFileBuilder;
    use ruff_text_size::TextSize;
    use tree_sitter::Parser;

    #[test]
    fn remove_from_variable_stmt() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
  integer :: x, Y(4), z = 5
  real, pointer :: a => null()
  integer :: d, &
     e &
     , f
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let symbol_table = SymbolTable::new(&root, code);
        let test_source = SourceFileBuilder::new("test.f90", code).finish();

        let x = symbol_table.get("x").unwrap();
        let y = symbol_table.get("y").unwrap();
        let z = symbol_table.get("Z").unwrap();
        let a = symbol_table.get("a").unwrap();
        let e = symbol_table.get("e").unwrap();

        let remove_x = remove_variable_decl(x.node(), x.decl_statement(), &test_source)?;
        assert_eq!(
            remove_x,
            Edit::deletion(TextSize::new(26), TextSize::new(28))
        );

        let remove_y = remove_variable_decl(y.node(), y.decl_statement(), &test_source)?;
        assert_eq!(
            remove_y,
            Edit::deletion(TextSize::new(29), TextSize::new(34))
        );

        let remove_z = remove_variable_decl(z.node(), z.decl_statement(), &test_source)?;
        assert_eq!(
            remove_z,
            Edit::deletion(TextSize::new(33), TextSize::new(40))
        );

        let remove_a = remove_variable_decl(a.node(), a.decl_statement(), &test_source)?;
        assert_eq!(
            remove_a,
            Edit::deletion(TextSize::new(41), TextSize::new(72))
        );

        let remove_e = remove_variable_decl(e.node(), e.decl_statement(), &test_source)?;
        assert_eq!(
            remove_e,
            Edit::deletion(TextSize::new(95), TextSize::new(105))
        );

        Ok(())
    }

    #[test]
    fn add_attr() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = r#"
program foo
  integer x
  integer :: y
  integer, save, allocatable, value :: z
end program foo
"#;
        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;

        let symbol_table = SymbolTable::new(&root, code);

        let x = symbol_table.get("x").unwrap();
        let y = symbol_table.get("y").unwrap();
        let z = symbol_table.get("z").unwrap();

        let add_x = add_attribute_to_var_decl(x.decl_statement(), "parameter");
        assert_eq!(
            add_x,
            Edit::insertion(", parameter ::".to_string(), TextSize::new(22))
        );

        let add_y = add_attribute_to_var_decl(y.decl_statement(), "parameter");
        assert_eq!(
            add_y,
            Edit::insertion(", parameter".to_string(), TextSize::new(34))
        );

        let add_z = add_attribute_to_var_decl(z.decl_statement(), "parameter");
        assert_eq!(
            add_z,
            Edit::insertion(", parameter".to_string(), TextSize::new(75))
        );

        Ok(())
    }

    #[test]
    fn redent_empty() {
        assert_eq!(redent("", "", LineEnding::Lf), "");
    }

    #[test]
    #[rustfmt::skip]
    fn redent_multi_line() {
        let x = [
            "    foo",
            "  bar",
            "    baz",
        ].join("\n");
        let y = [
            "   foo",
            " bar",
            "   baz"
        ].join("\n");
        assert_eq!(redent(&x, " ", LineEnding::Lf), y);
    }

    #[test]
    #[rustfmt::skip]
    fn redent_empty_line() {
        let x = [
            "    foo",
            "  bar",
            "   ",
            "    baz"
        ].join("\n");
        let y = [
            "   foo",
            " bar",
            "",
            "   baz"
        ].join("\n");
        assert_eq!(redent(&x, " ", LineEnding::Lf), y);
    }

    #[test]
    #[rustfmt::skip]
    fn redent_blank_line() {
        let x = [
            "      foo",
            "",
            "        bar",
            "          foo",
            "          bar",
            "          baz",
        ].join("\n");
        let y = [
            "foo",
            "",
            "  bar",
            "    foo",
            "    bar",
            "    baz",
        ].join("\n");
        assert_eq!(redent(&x, "", LineEnding::Lf), y);
    }

    #[test]
    #[rustfmt::skip]
    fn redent_whitespace_line() {
        let x = [
            "      foo",
            " ",
            "        bar",
            "          foo",
            "          bar",
            "          baz",
        ].join("\n");
        let y = [
            "  foo",
            "",
            "    bar",
            "      foo",
            "      bar",
            "      baz",
        ].join("\n");
        assert_eq!(redent(&x, "  ", LineEnding::Lf), y);
    }

    #[test]
    #[rustfmt::skip]
    fn redent_mixed_whitespace() {
        let x = [
            "\tfoo",
            "  bar",
        ].join("\n");
        let y = [
            "\tfoo",
            "  bar",
        ].join("\n");
        assert_eq!(redent(&x, "", LineEnding::Lf), y);
    }

    #[test]
    #[rustfmt::skip]
    fn redent_tabbed_whitespace() {
        let x = [
            "\t\tfoo",
            "\t\t\tbar",
        ].join("\n");
        let y = [
            "\tfoo",
            "\t\tbar",
        ].join("\n");
        assert_eq!(redent(&x, "\t", LineEnding::Lf), y);
    }

    #[test]
    #[rustfmt::skip]
    fn redent_mixed_tabbed_whitespace() {
        let x = [
            "\t  \tfoo",
            "\t  \t\tbar",
        ].join("\n");
        let y = [
            "  foo",
            "  \tbar",
        ].join("\n");
        assert_eq!(redent(&x, "  ", LineEnding::Lf), y);
    }

    #[test]
    #[rustfmt::skip]
    fn redent_mixed_tabbed_whitespace2() {
        let x = [
            "\t  \tfoo",
            "\t    \tbar",
        ].join("\n");
        let y = [
            "  \tfoo",
            "    \tbar",
        ].join("\n");
        assert_eq!(redent(&x, "  ", LineEnding::Lf), y);
    }

    #[test]
    #[rustfmt::skip]
    fn redent_preserve_no_terminating_newline() {
        let x = [
            "  foo",
            "    bar",
        ].join("\n");
        let y = [
            "foo",
            "  bar",
        ].join("\n");
        assert_eq!(redent(&x, "", LineEnding::Lf), y);
    }

    #[test]
    #[rustfmt::skip]
    fn redent_leave_shorter_comments() {
        let x = [
            "      foo",
            "        bar",
            "   !       foo",
            "     !     bar",
            "          baz",
        ].join("\n");
        let y = [
            "  foo",
            "    bar",
            "   !       foo",
            "     !     bar",
            "      baz",
        ].join("\n");
        assert_eq!(redent(&x, "  ", LineEnding::Lf), y);
    }
}
