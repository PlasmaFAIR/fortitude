use crate::settings::Settings;
use crate::AstRule;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, ViolationMetadata};
use ruff_source_file::SourceFile;
use ruff_text_size::{TextRange, TextSize};
use tree_sitter::Node;

/// ## What it does
/// Checks for any include statements
///
/// ## Why is this bad?
/// Include statements allow for pasting the contents of other files into
/// the current scope, which could be used for sharing COMMON blocks, procedures
/// or declaring variables. This can hide details from the programmer, increase
/// the maintenance burden and can be bug-prone. Avoided including files in
/// others and instead use modules.
///
/// ## References
/// - Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
///   Incorporating Fortran 2018_, Oxford University Press, Appendix A
///   'Deprecated Features'
/// - _Difference between INCLUDE and modules in Fortran_, 2013,
///   <https://stackoverflow.com/a/15668209>
#[derive(ViolationMetadata)]
pub(crate) struct IncludeStatement {}

impl Violation for IncludeStatement {
    #[derive_message_formats]
    fn message(&self) -> String {
        "Include statement is deprecated, use modules instead".to_string()
    }
}

impl AstRule for IncludeStatement {
    fn check(_settings: &Settings, node: &Node, _src: &SourceFile) -> Option<Vec<Diagnostic>> {
        // tree-sitter-fortran 0.5.1 includes the end newline as part
        // of the node, so we discard that here
        let start = TextSize::try_from(node.child(0)?.start_byte()).unwrap();
        let end = TextSize::try_from(node.child(1)?.end_byte()).unwrap();
        let range = TextRange::new(start, end);
        some_vec![Diagnostic::new(IncludeStatement {}, range)]
    }

    fn entrypoints() -> Vec<&'static str> {
        vec!["include_statement"]
    }
}
