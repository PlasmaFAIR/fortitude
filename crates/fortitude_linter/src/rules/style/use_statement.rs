use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::symbol_table::SymbolTables;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode};
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
        // Find all use statements
        let use_statements: Vec<Node> = node
            .children(&mut node.walk())
            .filter(|child| child.kind() == "use_statement")
            .collect();

        if use_statements.is_empty() {
            return None;
        }

        // Group use statements into blocks separated by empty lines
        let blocks = group_use_statements_into_blocks(&use_statements);

        let mut diagnostics = Vec::new();

        for block in blocks {
            if let Some(diagnostic) = check_and_fix_block(&block, src) {
                diagnostics.push(diagnostic);
            }
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

fn group_use_statements_into_blocks<'a>(use_statements: &[Node<'a>]) -> Vec<Vec<Node<'a>>> {
    let mut blocks = Vec::new();
    let mut current_block = Vec::new();

    for (i, stmt) in use_statements.iter().enumerate() {
        current_block.push(*stmt);

        if let Some(next_stmt) = use_statements.get(i + 1) {
            // If the next statement is not on the immediately following line
            // (blank line, comment, or any other content acts as a block separator),
            // close the current block and start a new one.
            if !are_statements_adjacent(stmt, next_stmt) {
                blocks.push(current_block);
                current_block = Vec::new();
            }
        }
    }

    if !current_block.is_empty() {
        blocks.push(current_block);
    }

    blocks
}

/// Two use statements are considered adjacent if the second one starts
/// on the line immediately following the end of the first one.
fn are_statements_adjacent(stmt1: &Node, stmt2: &Node) -> bool {
    let line1 = stmt1.end_position().row;
    let line2 = stmt2.start_position().row;
    line2 == line1 + 1
}

#[derive(Clone)]
struct UseStatementData {
    text: String,
    module_name: String,
    is_intrinsic: bool,
}

fn extract_use_statement_data(node: &Node, src: &SourceFile) -> UseStatementData {
    let range = node.textrange();
    let text = src.source_text().full_lines_str(range).to_string();

    let module_name = node
        .module_name(src.source_text())
        // Fortran is case-insensitive, normalize to lowercase for consistent sorting
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let is_intrinsic = node
        .children(&mut node.walk())
        .any(|child| child.to_text(src.source_text()) == Some("intrinsic"));

    UseStatementData {
        text,
        module_name,
        is_intrinsic,
    }
}

fn check_and_fix_block(block: &[Node], src: &SourceFile) -> Option<Diagnostic> {
    if block.len() <= 1 {
        return None; // Single statements or empty blocks don't need sorting
    }

    // Extract module name, intrinsic status and full line text
    let statements_with_data: Vec<UseStatementData> = block
        .iter()
        .map(|node| extract_use_statement_data(node, src))
        .collect();

    // Sort statements
    let mut sorted = statements_with_data.clone();
    sorted.sort_by(|a, b| compare_use_statements(a, b));

    // Check if already sorted
    let is_sorted = statements_with_data
        .iter()
        .zip(sorted.iter())
        .all(|(orig, s)| orig.text == s.text);

    if is_sorted {
        return None;
    }

    let block_start = src
        .source_text()
        .line_start(block.first()?.textrange().start());
    let block_end = src
        .source_text()
        .full_line_end(block.last()?.textrange().end());

    // Concatenate the sorted use statements into a single replacement string
    let replacement = sorted.iter().map(|s| s.text.as_str()).collect::<String>();

    let edit = Edit::range_replacement(replacement, TextRange::new(block_start, block_end));
    let fix = Fix::safe_edit(edit);

    Some(Diagnostic::from_node(UnsortedUses {}, block.first().unwrap()).with_fix(fix))
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
