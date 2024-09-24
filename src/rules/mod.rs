/// A collection of all rules, and utilities to select a subset at runtime.
mod error;
mod filesystem;
#[macro_use]
mod macros;
mod modules;
mod precision;
mod style;
mod typing;
use crate::register_rules;

register_rules! {
    (Category::Error, "E001", AST, error::syntax_error::SyntaxError, SyntaxError),
    (Category::Filesystem, "F001", PATH, filesystem::extensions::NonStandardFileExtension, NonStandardFileExtension),
    (Category::Style, "S001", TEXT, style::line_length::LineTooLong, LineTooLong),
    (Category::Style, "S101", TEXT, style::whitespace::TrailingWhitespace, TrailingWhitespace),
    (Category::Typing, "T001", AST, typing::implicit_typing::ImplicitTyping, ImplicitTyping),
    (Category::Typing, "T002", AST, typing::implicit_typing::InterfaceImplicitTyping, InterfaceImplicitTyping),
    (Category::Typing, "T003", AST, typing::implicit_typing::SuperfluousImplicitNone, SuperfluousImplicitNone),
    (Category::Typing, "T011", AST, typing::literal_kinds::LiteralKind, LiteralKind),
    (Category::Typing, "T012", AST, typing::literal_kinds::LiteralKindSuffix, LiteralKindSuffix),
    (Category::Typing, "T021", AST, typing::star_kinds::StarKind, StarKind),
    (Category::Precision, "P001", AST, precision::kind_suffixes::NoRealSuffix, NoRealSuffix),
    (Category::Precision, "P011", AST, precision::double_precision::DoublePrecision, DoublePrecision),
    (Category::Modules, "M001", AST, modules::external_functions::ExternalFunction, ExternalFunction),
    (Category::Modules, "M011", AST, modules::use_statements::UseAll, UseAll)
}
