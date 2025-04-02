/// Defines rules that govern the use of keywords.
use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
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
/// [`inout_with_space`](../settings.md#inout-with-space) and
/// [`goto_with_space`](../settings.md#goto-with-space).
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
    fn check(settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let first_child = if node.kind() == "inout" {
            *node
        } else {
            node.child(0)?
        };
        let text = first_child.to_text(src.source_text())?;
        let keywords = DoubleKeyword::from_str(text).ok()?;

        // Exit early if the keyword is permitted
        if matches!(keywords, DoubleKeyword::InOut)
            && !settings.check.keyword_whitespace.inout_with_space
        {
            return None;
        }
        if matches!(keywords, DoubleKeyword::GoTo)
            && !settings.check.keyword_whitespace.goto_with_space
        {
            return None;
        }

        let space_pos = TextSize::try_from(node.start_byte() + keywords.offset()).unwrap();
        let fix = Fix::safe_edit(Edit::insertion(" ".to_string(), space_pos));
        some_vec!(Diagnostic::from_node(Self { keywords }, &first_child).with_fix(fix))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec![
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
            "inout",
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
/// [`inout_with_space`](../settings.md#inout-with-space) and
/// [`goto_with_space`](../settings.md#goto-with-space).
///
/// ## Why is this bad?
/// By convention, `inout` in normally preferred to `in out`. Both `go to` and
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
    fn check(settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        if node.kind() == "inout" && settings.check.keyword_whitespace.inout_with_space {
            return None;
        }
        if node.kind() == "keyword_statement" && settings.check.keyword_whitespace.goto_with_space {
            return None;
        }
        let (first, second, first_child, violation) = if node.kind() == "inout" {
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
        if first_child.to_text(src.source_text())?.to_lowercase() != first {
            return None;
        }
        // Check if immediate sibling is 'out'/'to'
        let sibling = first_child.next_sibling()?;
        if sibling.to_text(src.source_text())?.to_lowercase() == second {
            let start = TextSize::try_from(first_child.start_byte()).unwrap();
            let end = TextSize::try_from(sibling.end_byte()).unwrap();
            let fix_start = TextSize::try_from(first_child.end_byte()).unwrap();
            let fix_end = TextSize::try_from(sibling.start_byte()).unwrap();
            let fix = Fix::safe_edit(Edit::deletion(fix_start, fix_end));
            return some_vec!(Diagnostic::new(violation, TextRange::new(start, end)).with_fix(fix));
        }
        // Can't fix this case, it may contain line continuations, comments, etc.
        some_vec!(Diagnostic::from_node(violation, &first_child))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec![
            "inout",
            "keyword_statement", // goto
        ]
    }
}

pub(crate) mod settings {
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
