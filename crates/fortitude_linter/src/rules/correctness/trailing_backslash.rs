use crate::ast::FortitudeNode;
/// Defines rules that govern line length.
use crate::settings::CheckSettings;
use crate::{AstRule, symbol_table::SymbolTables};
use lazy_regex::regex;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{ViolationMetadata, derive_message_formats};
use ruff_source_file::SourceFile;
use ruff_text_size::{TextRange, TextSize};
use tree_sitter::Node;

/// ## What does it do?
/// Checks if a backslash is the last character on a line
///
/// ## Why is this bad?
/// When compilers use the C preprocessor to pre-process Fortran files
/// the \ character is treated as a line continuation character by the C preprocessor,
/// potentially causing lines to be merged into one.
///
/// ## Example
/// When this Fortran program is passed through the C preprocessor,
/// ```f90
/// program t
///     implicit none
///     real :: A
///
///     ! Just a comment \
///     A = 2.0
///     print *, A
///  end
/// ```
/// it will end up with the variable assignment A placed onto the comment line,
/// ```f90
/// program t
///    implicit none
///    real :: A
///
///    ! Just a comment    A = 2.0
///
///    print *, A
/// end
/// ```
/// which causes the assignment to not be compiled.
///
#[derive(ViolationMetadata)]
pub(crate) struct TrailingBackslash {}

impl Violation for TrailingBackslash {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Trailing backslash".to_string()
    }
}

impl AstRule for TrailingBackslash {
    fn check(
        _settings: &CheckSettings,
        node: &Node,
        src: &SourceFile,
        _symbol_table: &SymbolTables,
    ) -> Option<Vec<Diagnostic>> {
        // Preprocessor might ignore trailing whitespace
        let trailing_backslash_re = regex!(r#".*(\\)\s*$"#);

        let comment = node.to_text(src.source_text())?;
        let captures = trailing_backslash_re.captures(comment)?;

        let trailing_backslash = captures.get(1)?;
        let start: TextSize = trailing_backslash.start().try_into().unwrap();
        let end: TextSize = trailing_backslash.end().try_into().unwrap();
        // Regex start/end are relative to start of comment node
        let comment_start: TextSize = node.start_byte().try_into().unwrap();
        let range = TextRange::new(comment_start + start, comment_start + end);
        some_vec!(Diagnostic::new(Self {}, range))
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["comment"]
    }
}
