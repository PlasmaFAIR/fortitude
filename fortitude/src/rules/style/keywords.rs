/// Defines rules that govern the use of keywords.
use crate::ast::FortitudeNode;
use crate::settings::Settings;
use crate::{AstRule, FromAstNode};
use ruff_diagnostics::{AlwaysFixableViolation, Diagnostic, Edit, Fix};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use ruff_text_size::TextSize;
use std::str::FromStr;
use tree_sitter::Node;

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
    EndFile,
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
            | DoubleKeyword::EndFile
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

    /// Return the ideal form of the combined keywords.
    fn preferred(&self) -> String {
        let mut result = self.to_string();
        result.insert(self.offset(), ' ');
        result
    }
}

// implement display for DoubleKeyword.

/// ## What it does
/// Checks for the use of keywords comprised of two words where the space is
/// omitted, such as `elseif` instead of `else if` and `endmodule` instead of
/// `endmodule`. The keywords `inout` and `goto` are exempt from this rule by
/// default, but may be included by supplying the relevant options
///
/// TODO list options
///
/// ## Why is this bad?
/// Contracting two keywords into one can make code less readable
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
        let preferred = self.keywords.preferred();
        format!("Replace with '{preferred}'")
    }
}

impl AstRule for KeywordsMissingSpace {
    fn check(_settings: &Settings, node: &Node, src: &SourceFile) -> Option<Vec<Diagnostic>> {
        let first_child = node.child(0)?;
        let text = first_child.to_text(src.source_text())?;
        let keywords = DoubleKeyword::from_str(text).ok()?;

        // Exit early if the keyword is permitted
        // TODO add options to also split these
        if matches!(keywords, DoubleKeyword::InOut | DoubleKeyword::GoTo) {
            return None;
        }

        let space_pos = TextSize::try_from(node.start_byte() + keywords.offset()).unwrap();
        let fix = Fix::safe_edit(Edit::insertion(" ".to_string(), space_pos));
        some_vec!(Diagnostic::from_node(Self { keywords }, &first_child).with_fix(fix))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["elseif_clause", "elsewhere_clause"]
    }
}
