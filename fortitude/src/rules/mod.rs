#![allow(clippy::useless_format)]
/// A collection of all rules, and utilities to select a subset at runtime.
pub(crate) mod error; // Public so we can use `IoError` in other places
mod filesystem;
#[macro_use]
mod macros;
mod modules;
mod precision;
mod style;
mod typing;
use crate::registry::{AsRule, Category};

use std::fmt::Formatter;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct NoqaCode(&'static str, &'static str);

impl NoqaCode {
    /// Return the prefix for the [`NoqaCode`], e.g., `SIM` for `SIM101`.
    pub fn prefix(&self) -> &str {
        self.0
    }

    /// Return the suffix for the [`NoqaCode`], e.g., `101` for `SIM101`.
    pub fn suffix(&self) -> &str {
        self.1
    }
}

impl std::fmt::Debug for NoqaCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for NoqaCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}{}", self.0, self.1)
    }
}

impl PartialEq<&str> for NoqaCode {
    fn eq(&self, other: &&str) -> bool {
        match other.strip_prefix(self.0) {
            Some(suffix) => suffix == self.1,
            None => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RuleGroup {
    /// The rule is stable.
    Stable,
    /// The rule is unstable, and preview mode must be enabled for usage.
    Preview,
    /// The rule has been deprecated, warnings will be displayed during selection in stable
    /// and errors will be raised if used with preview mode enabled.
    Deprecated,
    /// The rule has been removed, errors will be displayed on use.
    Removed,
}

#[fortitude_macros::map_codes]
pub fn code_to_rule(category: Category, code: &str) -> Option<(RuleGroup, Rule)> {
    #[allow(clippy::enum_glob_use)]
    use Category::*;

    #[rustfmt::skip]
    Some(match (category, code) {
        (Error, "000") => (RuleGroup::Stable, Path, error::ioerror::IoError),
        (Error, "001") => (RuleGroup::Stable, Ast, error::syntax_error::SyntaxError),

        (Filesystem, "001") => (RuleGroup::Stable, Path, filesystem::extensions::NonStandardFileExtension),

        (Style, "001") => (RuleGroup::Stable, Text, style::line_length::LineTooLong),
        (Style, "021") => (RuleGroup::Stable, Ast, style::exit_labels::MissingExitOrCycleLabel),
        (Style, "041") => (RuleGroup::Stable, Ast, style::old_style_array_literal::OldStyleArrayLiteral),
        (Style, "051") => (RuleGroup::Stable, Ast, style::relational_operators::DeprecatedRelationalOperator),
        (Style, "061") => (RuleGroup::Stable, Ast, style::end_statements::UnnamedEndStatement),
        (Style, "101") => (RuleGroup::Stable, Text, style::whitespace::TrailingWhitespace),

        (Typing, "001") => (RuleGroup::Stable, Ast, typing::implicit_typing::ImplicitTyping),
        (Typing, "002") => (RuleGroup::Stable, Ast, typing::implicit_typing::InterfaceImplicitTyping),
        (Typing, "003") => (RuleGroup::Stable, Ast, typing::implicit_typing::SuperfluousImplicitNone),
        (Typing, "011") => (RuleGroup::Stable, Ast, typing::literal_kinds::LiteralKind),
        (Typing, "012") => (RuleGroup::Stable, Ast, typing::literal_kinds::LiteralKindSuffix),
        (Typing, "021") => (RuleGroup::Stable, Ast, typing::star_kinds::StarKind),
        (Typing, "031") => (RuleGroup::Stable, Ast, typing::intent::MissingIntent),
        (Typing, "041") => (RuleGroup::Stable, Ast, typing::assumed_size::AssumedSize),
        (Typing, "042") => (RuleGroup::Stable, Ast, typing::assumed_size::AssumedSizeCharacterIntent),
        (Typing, "043") => (RuleGroup::Stable, Ast, typing::assumed_size::DeprecatedAssumedSizeCharacter),
        (Typing, "051") => (RuleGroup::Stable, Ast, typing::init_decls::InitialisationInDeclaration),

        (Precision, "001") => (RuleGroup::Stable, Ast, precision::kind_suffixes::NoRealSuffix),
        (Precision, "011") => (RuleGroup::Stable, Ast, precision::double_precision::DoublePrecision),
        (Precision, "021") => (RuleGroup::Stable, Ast, precision::implicit_kinds::ImplicitRealKind),

        (Modules, "001") => (RuleGroup::Stable, Ast, modules::external_functions::ExternalFunction),
        (Modules, "011") => (RuleGroup::Stable, Ast, modules::use_statements::UseAll),
    })
}
