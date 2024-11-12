#![allow(clippy::useless_format)]
/// A collection of all rules, and utilities to select a subset at runtime.
pub(crate) mod error; // Public so we can use `IOError` in other places
mod filesystem;
#[macro_use]
mod macros;
mod modules;
mod precision;
mod style;
mod typing;
use crate::registry::{AsRule, Category, RuleCheckKind};

use strum_macros::{AsRefStr, EnumIter};

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
        (Error, "E000")  => (RuleGroup::Stable, Path, error::ioerror::IOError),
        (Error, "E001")  => (RuleGroup::Stable, AST, error::syntax_error::SyntaxError),

        (Filesystem, "F001")  => (RuleGroup::Stable, Path, filesystem::extensions::NonStandardFileExtension),

        (Style, "S001")  => (RuleGroup::Stable, Text, style::line_length::LineTooLong),
        (Style, "S021")  => (RuleGroup::Stable, AST, style::exit_labels::MissingExitOrCycleLabel),
        (Style, "S041")  => (RuleGroup::Stable, AST, style::old_style_array_literal::OldStyleArrayLiteral),
        (Style, "S051")  => (RuleGroup::Stable, AST, style::relational_operators::DeprecatedRelationalOperator),
        (Style, "S061")  => (RuleGroup::Stable, AST, style::end_statements::UnnamedEndStatement),
        (Style, "S101")  => (RuleGroup::Stable, Text, style::whitespace::TrailingWhitespace),

        (Typing, "T001")  => (RuleGroup::Stable, AST, typing::implicit_typing::ImplicitTyping),
        (Typing, "T002")  => (RuleGroup::Stable, AST, typing::implicit_typing::InterfaceImplicitTyping),
        (Typing, "T003")  => (RuleGroup::Stable, AST, typing::implicit_typing::SuperfluousImplicitNone),
        (Typing, "T011")  => (RuleGroup::Stable, AST, typing::literal_kinds::LiteralKind),
        (Typing, "T012")  => (RuleGroup::Stable, AST, typing::literal_kinds::LiteralKindSuffix),
        (Typing, "T021")  => (RuleGroup::Stable, AST, typing::star_kinds::StarKind),
        (Typing, "T031")  => (RuleGroup::Stable, AST, typing::intent::MissingIntent),
        (Typing, "T041")  => (RuleGroup::Stable, AST, typing::assumed_size::AssumedSize),
        (Typing, "T042")  => (RuleGroup::Stable, AST, typing::assumed_size::AssumedSizeCharacterIntent),
        (Typing, "T043")  => (RuleGroup::Stable, AST, typing::assumed_size::DeprecatedAssumedSizeCharacter),
        (Typing, "T051")  => (RuleGroup::Stable, AST, typing::init_decls::InitialisationInDeclaration),

        (Precision, "P001")  => (RuleGroup::Stable, AST, precision::kind_suffixes::NoRealSuffix),
        (Precision, "P011")  => (RuleGroup::Stable, AST, precision::double_precision::DoublePrecision),
        (Precision, "P021")  => (RuleGroup::Stable, AST, precision::implicit_kinds::ImplicitRealKind),

        (Modules, "M001")  => (RuleGroup::Stable, AST, modules::external_functions::ExternalFunction),
        (Modules, "M011")  => (RuleGroup::Stable, AST, modules::use_statements::UseAll),
    })
}