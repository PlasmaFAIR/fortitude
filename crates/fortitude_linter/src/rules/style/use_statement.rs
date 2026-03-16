use crate::AstRule;
use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::traits::TextRanged;
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::{LineRanges, SourceFile};
use ruff_text_size::TextRange;
use tree_sitter::Node;

/// ## What it does
/// Checks that `use` statements are sorted alphabetically within contiguous blocks.
/// Intrinsic modules (`use, intrinsic ::`) are always placed first.
///
/// ## Why is this bad?
/// Sorted imports are easier to scan, reduce cognitive load when reviewing code,
/// and help avoid merge conflicts when multiple developers add imports to the same block.
///
/// ## Example
/// ```f90
/// ! Not recommended
/// use module_c, only: fun_c
/// use, intrinsic :: iso_fortran_env, only: int32
/// use module_a, only: fun_a
/// use module_b, only: fun_b
///
/// ! Better
/// use, intrinsic :: iso_fortran_env, only: int32
/// use module_a, only: fun_a
/// use module_b, only: fun_b
/// use module_c, only: fun_c
/// ```
///
/// Blocks of `use` statements separated by blank lines are sorted independently.
#[derive(ViolationMetadata)]
pub(crate) struct UnsortedUses {}

impl AlwaysFixableViolation for UnsortedUses {
    #[derive_message_formats]
    fn message(&self) -> String {
        "`use` statements are not sorted".to_string()
    }

    fn fix_title(&self) -> String {
        "Sort `use` statements".to_string()
    }
}

impl AstRule for UnsortedUses {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let use_statements: Vec<UseStatementData> = node
            .children(&mut node.walk())
            .filter(|child| child.kind() == "use_statement")
            .map(|child| extract_use_statement_data(&child, src))
            .collect();

        if use_statements.len() <= 1 {
            return None;
        }
        // Group use statements into blocks separated by empty lines
        let blocks = group_use_statements_into_blocks(&use_statements);

        let mut diagnostics = Vec::new();

        for block in &blocks {
            if block.len() <= 1 {
                continue;
            }

            let mut sorted: Vec<&UseStatementData> = block.to_vec();
            sorted.sort_by(|a, b| compare_use_statements(a, b));

            let is_sorted = block
                .iter()
                .zip(sorted.iter())
                .all(|(orig, s)| orig.text == s.text);

            if is_sorted {
                continue;
            }

            let block_start = src
                .source_text()
                .line_start(block.first()?.text_range.start());
            let block_end = src
                .source_text()
                .full_line_end(block.last()?.text_range.end());

            let replacement = sorted.iter().map(|s| s.text.as_str()).collect::<String>();
            let edit = Edit::range_replacement(replacement, TextRange::new(block_start, block_end));
            let fix = Fix::safe_edit(edit);

            let first = block.first()?;
            let diag = Diagnostic::new(UnsortedUses {}, first.text_range).with_fix(fix);
            diagnostics.push(diag);
        }

        if diagnostics.is_empty() {
            None
        } else {
            Some(diagnostics)
        }
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["module", "submodule", "program", "subroutine", "function"]
    }
}

/// Groups indices of `use` statements into contiguous blocks.
fn group_use_statements_into_blocks<'a>(
    all_use_statements: &'a [UseStatementData],
) -> Vec<Vec<&'a UseStatementData>> {
    let mut last_row: Option<usize> = None;

    let use_statements: Vec<&UseStatementData> = all_use_statements
        .iter()
        .filter(|child| {
            let row = child.start_position_row;
            if Some(row) == last_row {
                false
            } else {
                last_row = Some(row);
                true
            }
        })
        .collect();

    if use_statements.is_empty() {
        return Vec::new();
    }
    let mut blocks: Vec<Vec<&'a UseStatementData>> = Vec::new();
    let mut current_block = vec![use_statements[0]];

    for i in 1..use_statements.len() {
        let prev = &use_statements[i - 1];
        let curr = &use_statements[i];

        if are_statements_adjacent(prev, curr) {
            current_block.push(curr);
        } else {
            blocks.push(current_block);
            current_block = vec![curr];
        }
    }

    blocks.push(current_block);
    blocks
}

/// Two use statements are considered adjacent if the second one starts
/// on the line immediately following the end of the first one.
fn are_statements_adjacent(stmt1: &UseStatementData, stmt2: &UseStatementData) -> bool {
    let line1 = stmt1.end_position_row;
    let line2 = stmt2.start_position_row;
    line2 == line1 + 1
}

struct UseStatementData {
    text_range: TextRange,
    start_position_row: usize,
    end_position_row: usize,
    text: String,
    module_name: String,
    is_intrinsic: bool,
}

fn extract_use_statement_data(node: &Node, src: &SourceFile) -> UseStatementData {
    let module_name = node
        .module_name(src.source_text())
        // Fortran is case-insensitive, normalize to lowercase for consistent sorting
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let is_intrinsic = node
        .children(&mut node.walk())
        .any(|child| child.to_text(src.source_text()) == Some("intrinsic"));

    let mut text_range = node.textrange();
    let mut start_position_row = node.start_position().row;

    // If there's a preceding block of comments, then keep those attached to
    // this statement
    if let Some(comments) = node.prev_attached_comment_block(src.source_text()) {
        text_range = text_range.sub_start(comments.textrange().len());
        start_position_row = comments.start_row();
    }

    let text = src.source_text().full_lines_str(text_range).to_string();

    UseStatementData {
        text_range,
        start_position_row,
        end_position_row: node.end_position().row,
        text,
        module_name,
        is_intrinsic,
    }
}

// Intrinsic modules (e.g. `use, intrinsic :: iso_fortran_env`) always come first,
// followed by regular modules sorted alphabetically by name.
fn compare_use_statements(a: &UseStatementData, b: &UseStatementData) -> std::cmp::Ordering {
    match (a.is_intrinsic, b.is_intrinsic) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.module_name.cmp(&b.module_name),
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use ruff_source_file::SourceFileBuilder;
    use tree_sitter::Parser;

    use crate::rules::style::use_statement::{
        UseStatementData, extract_use_statement_data, group_use_statements_into_blocks,
    };

    #[test]
    fn test_group_use_statements_into_blocks() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        // Block 1: alpha, beta
        // blank line separator
        // Block 2: charlie, delta, echo
        // Block 4: only foxtrot is kept — golf is on the same line and must be ignored
        let code = {
            r#"
        program foo
          use alpha_module
          use beta_module

          use charlie_module
          use delta_module
          ! a comment acts as a separator
          use echo_module

          use foxtrot_module; use golf_module
        end program foo
    "#
        };

        let tree = parser.parse(code, None).context("Failed to parse")?;
        let src = SourceFileBuilder::new("test.f90", code).finish();

        let program_node = tree.root_node().child(0).context("Missing program node")?;
        assert_eq!(program_node.kind(), "program");

        let use_statements: Vec<UseStatementData> = program_node
            .children(&mut program_node.walk())
            .filter(|child| child.kind() == "use_statement")
            .map(|child| extract_use_statement_data(&child, &src))
            .collect();
        let blocks = group_use_statements_into_blocks(&use_statements);
        let block_names = |block: &Vec<&UseStatementData>| -> Vec<String> {
            block.iter().map(|s| s.module_name.clone()).collect()
        };

        assert_eq!(blocks.len(), 3, "expected 3 blocks");
        assert_eq!(block_names(&blocks[0]), vec!["alpha_module", "beta_module"]);
        assert_eq!(
            block_names(&blocks[1]),
            vec!["charlie_module", "delta_module", "echo_module"]
        );
        assert_eq!(block_names(&blocks[2]), vec!["foxtrot_module"]); // golf_module ignored: same line

        Ok(())
    }
    #[test]
    fn test_extract_use_statement_data() -> Result<()> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_fortran::LANGUAGE.into())
            .context("Error loading Fortran grammar")?;

        let code = {
            r#"
        program foo
          use iso_fortran_env, only: real64
          use, intrinsic :: iso_c_binding, only: c_int
          use My_Module
          use foxtrot_module; use golf_module
          use multiline_module, only: fun_1, &
                                      fun_2, & !! 123_comments
                                      fun_3
        end program foo
    "#
        };

        let tree = parser.parse(code, None).context("Failed to parse")?;
        let root = tree.root_node().child(0).context("Missing child")?;
        let src = SourceFileBuilder::new("test.f90", code).finish();

        let all_use_statements: Vec<UseStatementData> = root
            .children(&mut root.walk())
            .filter(|child| child.kind() == "use_statement")
            .map(|child| extract_use_statement_data(&child, &src))
            .collect();

        assert_eq!(all_use_statements.len(), 6);

        // Test regular use statement
        let regular = &all_use_statements[0];
        assert!(!regular.is_intrinsic);
        assert_eq!(regular.module_name, "iso_fortran_env");
        assert_eq!(regular.start_position_row, 2);
        assert_eq!(regular.end_position_row, 2);
        assert!(regular.text.contains("iso_fortran_env"));
        assert!(!regular.text_range.is_empty());

        // Test intrinsic use statement
        let intrinsic = &all_use_statements[1];
        assert!(intrinsic.is_intrinsic);
        assert_eq!(intrinsic.module_name, "iso_c_binding");
        assert_eq!(intrinsic.start_position_row, 3);
        assert_eq!(intrinsic.end_position_row, 3);
        assert!(intrinsic.text.contains("iso_c_binding"));
        assert!(!intrinsic.text_range.is_empty());

        // Test mixed case use statement
        let mixed_case = &all_use_statements[2];
        assert!(!mixed_case.is_intrinsic);
        assert_eq!(mixed_case.module_name, "my_module");
        assert_eq!(mixed_case.start_position_row, 4);
        assert_eq!(mixed_case.end_position_row, 4);
        assert!(mixed_case.text.contains("My_Module"));
        assert!(!mixed_case.text_range.is_empty());

        // Test foxtrot_module (first on same line)
        let foxtrot = &all_use_statements[3];
        assert!(!foxtrot.is_intrinsic);
        assert_eq!(foxtrot.module_name, "foxtrot_module");
        assert_eq!(foxtrot.start_position_row, 5);
        assert_eq!(foxtrot.end_position_row, 5);
        assert!(foxtrot.text.contains("foxtrot_module"));
        assert!(!foxtrot.text_range.is_empty());

        // Test golf_module (second on same line)
        let golf = &all_use_statements[4];
        assert!(!golf.is_intrinsic);
        assert_eq!(golf.module_name, "golf_module");
        assert_eq!(golf.start_position_row, 5);
        assert_eq!(golf.end_position_row, 5);
        assert!(golf.text.contains("golf_module"));
        assert!(!golf.text_range.is_empty());

        // Test multiline_module
        let multiline = &all_use_statements[5];
        assert!(!multiline.is_intrinsic);
        assert_eq!(multiline.module_name, "multiline_module");
        assert_eq!(multiline.start_position_row, 6);
        assert_eq!(multiline.end_position_row, 8);
        assert!(multiline.text.contains("fun_1"));
        assert!(multiline.text.contains("fun_2"));
        assert!(multiline.text.contains("fun_3"));
        assert!(multiline.text.contains("123_comments"));

        assert!(!golf.text_range.is_empty());
        Ok(())
    }
}
