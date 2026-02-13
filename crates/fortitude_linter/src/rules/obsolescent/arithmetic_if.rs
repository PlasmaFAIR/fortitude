use crate::ast::FortitudeNode;
use crate::settings::CheckSettings;
use crate::traits::TextRanged;
use crate::{AstRule, FromAstNode, SymbolTables};
use itertools::Itertools;
use log::debug;
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use strum_macros::{Display, EnumString};
use tree_sitter::Node;
use unicode_width::UnicodeWidthStr;

/// ## What it does
/// Checks for arithmetic `if` statements.
///
/// ## Why is this bad?
/// The arithmetic `if` statement is used to jump between one of three statement
/// labels depending on whether the condition is below, above, or equal to
/// zero. However, this is incompatible with the IEEE 754 standard on floating
/// point numbers (due to the comparison between `real`s), and the use of
/// statment labels can hinder optimisation, as well as making the code harder
/// to read and maintain.
///
/// ## Example
/// ```f90
///     IF(x(1)) 10, 20, 30
/// 10  PRINT *, 'first case'
///     GOTO 40
/// 20  PRINT *, 'second case'
///     GOTO 40
/// 30  PRINT *, 'third case'
/// 40  CONTINUE
/// ```
///
/// Use instead:
/// ```f90
/// if (x(1) < 0) then
///   print*, "first case"
/// else if (x(1) > 0) then
///   print*, "third case"
/// else
///   print*, "second case"
/// end if
/// ```
#[derive(ViolationMetadata)]
pub(crate) struct ArithmeticIf;

impl Violation for ArithmeticIf {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        "Obsolete arithmetic `if`".to_string()
    }

    fn fix_title(&self) -> Option<String> {
        Some("Use `if` statement or `if` construct".into())
    }
}

#[derive(Clone, Copy, Debug, Display, EnumString)]
enum LabelRefCmp {
    #[strum(serialize = "<")]
    LessThan,
    #[strum(serialize = "<=")]
    LessThanEqual,
    #[strum(serialize = "==")]
    Equal,
    #[strum(serialize = ">=")]
    GreaterThanEqual,
    #[strum(serialize = ">")]
    GreaterThan,
}

#[derive(Clone, Debug)]
enum ControlFlow {
    Continue,
    Cycle,
    Exit,
    GoTo(String),
    Return,
    Stop,
}

impl ControlFlow {
    fn maybe_from(value: &Node, src: &str) -> Option<Self> {
        debug!("--- {value:?}");
        if value.kind() != "keyword_statement" {
            return None;
        }
        match value.child(0)?.to_text(src)?.to_ascii_lowercase().as_str() {
            "continue" => Some(Self::Continue),
            "cycle" => Some(Self::Cycle),
            "exit" => Some(Self::Exit),
            "return" => Some(Self::Return),
            "stop" => Some(Self::Stop),
            "error" => Some(Self::Stop),
            keyword => Self::parse_goto(keyword, value, src),
        }
    }

    fn parse_goto(keyword: &str, value: &Node, src: &str) -> Option<Self> {
        if !matches!(keyword, "go" | "goto") {
            debug!("--- *** {keyword}");
            return None;
        }

        // We expect either `go to N` or `goto N`.
        // Don't bother with assigned or computed gotos for now
        let expected_ref_index = if keyword == "go" { 2 } else { 1 };
        if value.child_count() > expected_ref_index + 1 {
            debug!(
                "--- --- {} > {}",
                value.child_count(),
                expected_ref_index + 1
            );
            return None;
        }

        Some(Self::GoTo(
            value.child(expected_ref_index)?.to_text(src)?.to_string(),
        ))
    }
}

/// We can fix if:
/// 1. labels follow the `if` (no going backwards
/// 2. the previous non-comment statement to each label is one of:
///    - the `if` statement itself
///    - `goto`
///    - `return`
/// 3. if the previous non-comment statement is a `goto` then:
///    - it must follow the last label
///
/// This give us structures like:
///
/// if (condition) A, B, C
/// <only comments>
/// A <statement>
///   ...
///   (goto D | return)
/// B <statement>
///   ...
///   (goto D | return)
/// C <statement>
///   ...
/// [D <statement>]
///
/// A, B, C don't have to appear in that order!
///
/// If A, B, C are all distinct, and either A or B blocks end in `goto
/// D`, then this should get translated to:
///
/// if (condition < 0) then
///    A ...
/// else if (condition > 0) then
///    C ...
/// else
///    B ...
/// end if
///
/// If A, B are identical, then `condition` should be translated to `<=` and we don't need B
/// If B, C are identical, then `condition` should be translated to `>=` and we don't need B
///
/// If either A or B end in `return`, then we must be a lot more careful,
/// because we may not know where C ends.
/// It must end with the close of the current block -- do we have a way of getting that?
fn fix_arithmetic_if(node: &Node, source: &SourceFile) -> Option<Fix> {
    let src = source.source_text();

    // First, we get the three label references
    let mut cursor = node.walk();
    let mut refs = node
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "statement_label_reference");

    let less_than = refs.next()?.to_text(src)?;
    let equal = refs.next()?.to_text(src)?;
    let greater_than = refs.next()?.to_text(src)?;

    // Check if the `equal` branch is the same as one of the others.
    // Probably don't need to worry about the other two being identical?
    let less_than_equal = less_than == equal;
    let greater_than_equal = greater_than == equal;

    // Then we find the labels they reference
    let mut refs = Vec::new();
    if less_than_equal {
        refs.push((less_than, LabelRefCmp::LessThanEqual));
    } else {
        refs.push((less_than, LabelRefCmp::LessThan));
    }

    if !less_than_equal && !greater_than_equal {
        refs.push((equal, LabelRefCmp::Equal));
    }

    if greater_than_equal {
        refs.push((greater_than, LabelRefCmp::GreaterThanEqual));
    } else {
        refs.push((greater_than, LabelRefCmp::GreaterThan));
    }

    // and get them in the order that they appear
    let mut labels = refs
        .iter()
        .filter_map(|label| Some((node.next_statement_label(label.0, src)?, label.1)))
        .sorted_by_key(|label| label.0.start_byte());
    // expect at least two sections!
    let first = labels.next()?;
    let second = labels.next()?;
    // might be None!
    let third = labels.next();

    let last = if let Some(third) = third {
        third
    } else {
        second
    };

    debug!("less_than = {less_than:?}\nequal = {equal:?}\ngreater_than = {greater_than:?}\n");
    debug!("first = {first:?}\nsecond = {second:?}\nthird = {third:?}\n");

    // First labelled section must be immediately following the `if` statement
    debug!("{:?} | {:?}\n", first.0.prev_non_comment_sibling(), node);
    if first.0.prev_non_comment_sibling() != Some(*node) {
        return None;
    }

    // Second and third sections must be immediately following either `goto` or `return`
    let end_first = ControlFlow::maybe_from(&second.0.prev_non_comment_sibling()?, src);
    let end_second = if let Some(third) = third {
        // If the second block doesn't end in a control flow, then exit early.
        // But we need the `Some` because we might not have `end_second`
        ControlFlow::maybe_from(&third.0.prev_non_comment_sibling()?, src)
    } else {
        None
    };

    debug!("end_first = {end_first:?}\nend_second = {end_second:?}\n");

    // If they're both gotos, then they better point to the same place
    if let Some(ControlFlow::GoTo(ref first_ref)) = end_first
        && let Some(ControlFlow::GoTo(ref second_ref)) = end_second
        && (first_ref != second_ref)
    {
        debug!("{first_ref} != {second_ref}");
        return None;
    }

    let final_close_node = match end_first {
        Some(ControlFlow::GoTo(ref_)) => last.0.next_statement_label_sibling(ref_, src),
        _ => None,
    }?;

    debug!("close_node = {final_close_node:?}\n");

    // Now we should have everything we need to rewrite the whole block

    // TODO: this is too crude, need to just lop off brackets
    let condition = node
        .child_with_name("parenthesized_expression")?
        .named_child(0)?
        .to_text(src)?;

    // TODO: indentation needs Stylist
    // TODO: need to indent all lines in block?
    let width = first.0.to_text(src)?.width() + 2;

    let first_edit = Edit::replacement(
        format!("if ({condition} {} 0) then\n{:width$}", first.1, " "),
        node.start_textsize(),
        first.0.end_textsize(),
    );

    let mut edits = Vec::new();

    let width = second.0.to_text(src)?.width() + 2;
    edits.push(Edit::replacement(
        format!("else if ({condition} {} 0) then\n{:width$}", second.1, " "),
        second.0.prev_non_comment_sibling()?.start_textsize(),
        second.0.end_textsize(),
    ));

    if let Some(third) = third {
        let width = third.0.to_text(src)?.width() + 2;
        edits.push(Edit::replacement(
            format!("else if ({condition} {} 0) then\n{:width$}", third.1, " "),
            third.0.prev_non_comment_sibling()?.start_textsize(),
            third.0.end_textsize(),
        ));
    }

    // TODO: indentation
    edits.push(Edit::replacement(
        "end if\n".to_string(),
        final_close_node.start_textsize(),
        final_close_node.end_textsize(),
    ));

    // TODO: Also delete the next statement if it's `continue`

    Some(Fix::unsafe_edits(first_edit, edits))
}

impl AstRule for ArithmeticIf {
    fn check<'a>(
        _settings: &CheckSettings,
        node: &'a Node,
        src: &'a SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        let mut diagnostic = Diagnostic::from_node(ArithmeticIf {}, node);
        if let Some(fix) = fix_arithmetic_if(node, src) {
            diagnostic.set_fix(fix);
        }
        some_vec![diagnostic]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["arithmetic_if_statement"]
    }
}
