/// Defines rules that govern the use of keywords.
use crate::ast::{FortitudeNode, symbol_table::SymbolTable};
use crate::diagnostics::{
    AlwaysFixableViolation, Diagnostic, Edit, Fix, FixAvailability, Violation,
};
use crate::stylist::ToCapitalisation;
use crate::traits::TextRanged;
use crate::{AstRule, CheckContext, kind_ids};
use fortitude_macros::ViolationMetadata;
use itertools::Itertools;
use ruff_macros::derive_message_formats;
use ruff_text_size::{TextRange, TextSize};
use std::str::FromStr;
use tree_sitter::Node;

// TODO Support for `endfile`/`end file`

#[derive(strum_macros::EnumString, strum_macros::Display)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
enum DoubleKeyword {
    DoublePrecision,
    DoubleComplex,
    SelectCase,
    SelectType,
    ElseIf,
    ElseWhere,
    EndAssociate,
    EndBlock,
    EndCritical,
    EndDo,
    EndEnum,
    EndForAll,
    EndFunction,
    EndIf,
    EndInterface,
    EndModule,
    EndProcedure,
    EndProgram,
    EndSelect,
    EndSubmodule,
    EndSubroutine,
    EndTeam,
    EndType,
    EndWhere,
    InOut,
    GoTo,
}

impl DoubleKeyword {
    /// Returns the number of characters that the first keyword is comprised of.
    fn offset(&self) -> usize {
        match self {
            DoubleKeyword::InOut | DoubleKeyword::GoTo => 2,
            DoubleKeyword::DoublePrecision
            | DoubleKeyword::DoubleComplex
            | DoubleKeyword::SelectCase
            | DoubleKeyword::SelectType => 6,
            DoubleKeyword::ElseIf | DoubleKeyword::ElseWhere => 4,
            DoubleKeyword::EndAssociate
            | DoubleKeyword::EndBlock
            | DoubleKeyword::EndCritical
            | DoubleKeyword::EndDo
            | DoubleKeyword::EndEnum
            | DoubleKeyword::EndForAll
            | DoubleKeyword::EndFunction
            | DoubleKeyword::EndIf
            | DoubleKeyword::EndInterface
            | DoubleKeyword::EndModule
            | DoubleKeyword::EndProcedure
            | DoubleKeyword::EndProgram
            | DoubleKeyword::EndSelect
            | DoubleKeyword::EndSubmodule
            | DoubleKeyword::EndSubroutine
            | DoubleKeyword::EndTeam
            | DoubleKeyword::EndType
            | DoubleKeyword::EndWhere => 3,
        }
    }

    /// Return keywords with a space between them.
    fn with_space(&self) -> String {
        let mut result = self.to_string();
        result.insert(self.offset(), ' ');
        result
    }
}

/// ## What it does
/// Checks for the use of keywords comprised of two words where the space is
/// omitted, such as `elseif` instead of `else if` and `endmodule` instead of
/// `endmodule`. The keywords `inout` and `goto` are exempt from this rule by
/// default, but may be included by setting the options
/// [`inout-with-space`](../settings.md#inout-with-space) and
/// [`goto-with-space`](../settings.md#goto-with-space).
///
/// ## Why is this bad?
/// Contracting two keywords into one can make code less readable. Enforcing
/// this rule can help maintain a consistent style.
#[derive(ViolationMetadata)]
pub struct KeywordsMissingSpace {
    keywords: DoubleKeyword,
}

impl AlwaysFixableViolation for KeywordsMissingSpace {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { keywords } = self;
        format!("Missing space in '{keywords}'")
    }

    fn fix_title(&self) -> String {
        let preferred = self.keywords.with_space();
        format!("Replace with '{preferred}'")
    }
}

impl AstRule for KeywordsMissingSpace {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        let first_child = if node.kind() == "inout" {
            *node
        } else {
            node.child(0)?
        };
        let text = first_child.to_text(context.source_text())?;
        let keywords = DoubleKeyword::from_str(text).ok()?;

        // Exit early if the keyword is permitted
        let keyword_settings = &context.settings().keyword_whitespace;
        if matches!(keywords, DoubleKeyword::InOut) && !keyword_settings.inout_with_space {
            return None;
        }
        if matches!(keywords, DoubleKeyword::GoTo) && !keyword_settings.goto_with_space {
            return None;
        }

        let space_pos = node.start_textsize() + TextSize::try_from(keywords.offset()).unwrap();
        let fix = Fix::safe_edit(Edit::insertion(" ".to_string(), space_pos));
        some_vec!(
            context
                .create_diagnostic(Self { keywords }, first_child)
                .with_fix(fix)
        )
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids![
            "elseif_clause",
            "elsewhere_clause",
            "end_associate_statement",
            "end_block_construct_statement",
            "end_coarray_critical_statement",
            "end_coarray_team_statement",
            "end_do_loop_statement",
            "end_enum_statement",
            "end_forall_statement",
            "end_function_statement",
            "end_if_statement",
            "end_interface_statement",
            "end_module_procedure_statement",
            "end_module_statement",
            "end_program_statement",
            "end_select_statement",
            "end_submodule_statement",
            "end_subroutine_statement",
            "end_type_statement",
            "end_where_statement",
            "inout" | kw,
            "intrinsic_type",    // double precision and double complex
            "keyword_statement", // goto
            "select_case_statement",
            "select_type_statement",
        ]
    }
}

/// ## What it does
/// Checks for the use of `in out` instead of `inout` and `go to` instead of `goto`.
/// Either may be exempted from this rule by setting the options
/// [`inout-with-space`](../settings.md#inout-with-space) and
/// [`goto-with-space`](../settings.md#goto-with-space).
///
/// ## Why is this bad?
/// By convention, `inout` is normally preferred to `in out`. Both `go to` and
/// `goto` are valid, but Fortitude prefers the latter as `goto` is most common
/// in other languages, and neither `go` nor `to` have secondary purposes in
/// other keywords. Enforcing this rule can help maintain a consistent style.
#[derive(ViolationMetadata)]
pub struct KeywordHasWhitespace {
    keywords: DoubleKeyword,
}

impl Violation for KeywordHasWhitespace {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Sometimes;

    #[derive_message_formats]
    fn message(&self) -> String {
        let keywords = self.keywords.with_space();
        format!("Whitespace included in '{keywords}'")
    }

    fn fix_title(&self) -> Option<String> {
        let preferred = self.keywords.to_string();
        Some(format!("Replace with '{preferred}'"))
    }
}

impl AstRule for KeywordHasWhitespace {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if node.kind() == "in" && context.settings().keyword_whitespace.inout_with_space {
            return None;
        }
        if node.kind() == "keyword_statement"
            && context.settings().keyword_whitespace.goto_with_space
        {
            return None;
        }
        let (first, second, first_child, violation) = if node.kind() == "in" {
            // We don't want to match just `in`, so need to check the
            // next node, but not if someone has perversely put a
            // comment and/or line break inbetween
            let mut sibling = node.next_sibling()?;
            while matches!(sibling.kind(), "comment" | "&") {
                sibling = sibling.next_sibling()?;
            }
            if sibling.kind() != "out" {
                return None;
            }

            (
                "in",
                "out",
                *node,
                Self {
                    keywords: DoubleKeyword::InOut,
                },
            )
        } else {
            (
                "go",
                "to",
                node.child(0)?,
                Self {
                    keywords: DoubleKeyword::GoTo,
                },
            )
        };
        // Verify that the node is 'in' / 'go'
        if first_child.to_text(context.source_text())?.to_lowercase() != first {
            return None;
        }
        // Check if immediate sibling is 'out'/'to'
        let sibling = first_child.next_sibling()?;
        if sibling.to_text(context.source_text())?.to_lowercase() == second {
            let start = first_child.start_textsize();
            let end = sibling.end_textsize();
            let fix_start = first_child.end_textsize();
            let fix_end = sibling.start_textsize();
            let fix = Fix::safe_edit(Edit::deletion(fix_start, fix_end));
            return some_vec!(
                context
                    .create_diagnostic(violation, TextRange::new(start, end))
                    .with_fix(fix)
            );
        }
        // Can't fix this case, it may contain line continuations, comments, etc.
        some_vec!(context.create_diagnostic(violation, first_child))
    }

    fn entrypoints() -> Vec<u16> {
        kind_ids![
            "in" | kw,
            "keyword_statement", // goto
        ]
    }
}

pub mod settings {
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use std::fmt::{Display, Formatter};

    #[derive(Debug, Clone, Default, CacheKey)]
    pub struct Settings {
        pub inout_with_space: bool,
        pub goto_with_space: bool,
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.keyword-whitespace",
                fields = [self.inout_with_space, self.goto_with_space]
            }
            Ok(())
        }
    }
}

/// ## What it does
/// Checks that Fortran keywords use a consistent casing style. Flags any keyword
/// whose casing does not match the configured style (see
/// [`keyword-case`](../settings.md#keyword-case)).
///
/// ## Why is this bad?
/// Fortran is case-insensitive, so keyword casing is purely a stylistic choice.
/// However, inconsistent casing — mixing `IMPLICIT NONE`, `implicit none`, and
/// `Implicit None` in the same codebase — reduces readability and makes code
/// harder to scan. Enforcing a consistent style helps maintain a uniform
/// appearance across the codebase.
///
/// Modern Fortran style guides generally favour lowercase keywords. Older
/// codebases often use uppercase, inherited from fixed-form Fortran conventions.
///
/// ## Examples
///
/// With `keyword-case = "lowercase"`:
///
/// ### Incorrect
/// ```f90
/// IMPLICIT NONE
/// INTEGER, INTENT(IN) :: x
/// END SUBROUTINE foo
/// ```
///
/// ### Correct
/// ```f90
/// implicit none
/// integer, intent(in) :: x
/// end subroutine foo
/// ```
///
/// With `keyword-case = "uppercase"`:
///
/// ### Incorrect
/// ```f90
/// implicit none
/// integer, intent(in) :: x
/// end subroutine foo
/// ```
///
/// ### Correct
/// ```f90
/// IMPLICIT NONE
/// INTEGER, INTENT(IN) :: x
/// END SUBROUTINE foo
/// ```
#[derive(ViolationMetadata)]
pub struct IncorrectKeywordCase {
    actual: String,
    expected: String,
}

impl AlwaysFixableViolation for IncorrectKeywordCase {
    #[derive_message_formats]
    fn message(&self) -> String {
        let Self { actual, expected } = self;
        format!("Keyword '{actual}' should be '{expected}'")
    }

    fn fix_title(&self) -> String {
        let Self {
            actual: _,
            expected,
        } = self;
        format!("Change to '{expected}'")
    }
}

impl AstRule for IncorrectKeywordCase {
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if node.child_count() > 0 {
            // Filter node that wrap other nodes, we want only leafs
            return None;
        }
        if node.parent()?.kind() == "identifier" {
            // This is actually a variable
            return None;
        }

        let text = node.to_text(context.source_text())?;

        let expected =
            text.to_capitalisation(context.settings().incorrect_keyword_case.keyword_case);
        if text == expected {
            return None;
        }

        // build fix
        let fix = Fix::safe_edit(Edit::replacement(
            expected.clone(),
            TextSize::from(node.start_byte() as u32),
            TextSize::from(node.end_byte() as u32),
        ));

        some_vec!(
            context
                .create_diagnostic(
                    Self {
                        actual: text.to_string(),
                        expected
                    },
                    node
                )
                .with_fix(fix)
        )
    }

    fn entrypoints() -> Vec<u16> {
        let language: tree_sitter::Language = tree_sitter_fortran::LANGUAGE.into();
        // TODO(peter): make this compiletime
        tree_sitter_fortran::KEYWORDS
            .iter()
            .map(|kw| language.id_for_node_kind(kw, false))
            // these two are named
            .chain(kind_ids!["none", "default"])
            .collect_vec()
    }
}

pub mod settings_keyword_case {
    use crate::{display_settings, stylist::Capitalisation};
    use ruff_macros::CacheKey;
    use std::fmt::{Display, Formatter};

    #[derive(Debug, Clone, Default, CacheKey)]
    pub struct Settings {
        pub keyword_case: Capitalisation,
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.keyword-case",
                fields = [self.keyword_case]
            }
            Ok(())
        }
    }
}

/// ## What it does
/// Checks for the use of keywords when naming variables, modules, functions,
/// etc.
///
/// ## Why is this bad?
/// The reuse of keywords as identifiers can be confusing to readers, and may cause
/// problems for some tools. Enforcing this rule can help maintain a consistent style.
///
/// ## Examples
///
/// ```f90
/// module program
///
///   implicit none (type, external)
///   private
///
/// contains
///
///   subroutine function(stop)
///     integer, intent(in) :: stop
///     print *, stop
///   end subroutine function
///
/// end module program
/// ```
#[derive(ViolationMetadata)]
pub struct KeywordReuse {
    keyword: String,
    is_block_label: bool,
}

impl Violation for KeywordReuse {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    #[derive_message_formats]
    fn message(&self) -> String {
        let keyword = &self.keyword;
        if self.is_block_label {
            format!("Keyword `{keyword}` used as a label")
        } else {
            format!("Keyword `{keyword}` used as an identifier")
        }
    }
}

/// Check the symbol table for any identifiers that are the same as Fortran
/// keywords.
pub(crate) fn check_keyword_reuse(
    context: &CheckContext,
    symbols: &SymbolTable,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    // Check the names of all variables declared in the current scope
    for (name, symbol) in symbols.iter() {
        if tree_sitter_fortran::KEYWORDS.contains(&name.as_str()) {
            diagnostics.push(context.create_diagnostic(
                KeywordReuse {
                    keyword: name.to_string(),
                    is_block_label: false,
                },
                symbol.name(),
            ));
        }
    }
    diagnostics.sort_by(|a, b| a.range().ordering(b.range()));
    diagnostics
}

impl AstRule for KeywordReuse {
    /// Check for keyword reuse for block labels, which are not currently
    /// findable via the symbol table.
    fn check(context: &CheckContext, node: &Node) -> Option<Vec<Diagnostic>> {
        if node.kind() != "block_label_start_expression" {
            return None;
        }
        let name_node = node.child(0)?;
        let name = name_node.to_text(context.source_text())?;
        if tree_sitter_fortran::KEYWORDS.contains(&name) {
            return some_vec![context.create_diagnostic(
                KeywordReuse {
                    keyword: name.to_string(),
                    is_block_label: true,
                },
                name_node,
            )];
        }
        None
    }

    /// Entry point only on `block_label_start_expression`, as other cases of
    /// keyword reuse should be caught by `check_keyword_reuse`.
    fn entrypoints() -> Vec<u16> {
        kind_ids!["block_label_start_expression"]
    }
}
