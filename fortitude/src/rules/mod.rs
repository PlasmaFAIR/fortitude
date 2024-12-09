#![allow(clippy::useless_format)]
/// A collection of all rules, and utilities to select a subset at runtime.
pub(crate) mod error;
pub(crate) mod filesystem;
#[macro_use]
mod macros;
pub(crate) mod modules;
pub(crate) mod obsolescent;
pub(crate) mod precision;
pub(crate) mod style;
pub(crate) mod testing;
pub(crate) mod typing;
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
        (Style, "071") => (RuleGroup::Stable, Ast, style::double_colon_in_decl::MissingDoubleColon),
        (Style, "101") => (RuleGroup::Stable, Text, style::whitespace::TrailingWhitespace),
        (Style, "102") => (RuleGroup::Stable, Ast, style::whitespace::IncorrectSpaceBeforeComment),

        (Typing, "001") => (RuleGroup::Stable, Ast, typing::implicit_typing::ImplicitTyping),
        (Typing, "002") => (RuleGroup::Stable, Ast, typing::implicit_typing::InterfaceImplicitTyping),
        (Typing, "003") => (RuleGroup::Stable, Ast, typing::implicit_typing::SuperfluousImplicitNone),
        (Typing, "004") => (RuleGroup::Preview, Ast, typing::implicit_typing::ImplicitExternalProcedures),
        (Typing, "011") => (RuleGroup::Stable, Ast, typing::literal_kinds::LiteralKind),
        (Typing, "012") => (RuleGroup::Stable, Ast, typing::literal_kinds::LiteralKindSuffix),
        (Typing, "021") => (RuleGroup::Stable, Ast, typing::star_kinds::StarKind),
        (Typing, "031") => (RuleGroup::Stable, Ast, typing::intent::MissingIntent),
        (Typing, "041") => (RuleGroup::Stable, Ast, typing::assumed_size::AssumedSize),
        (Typing, "042") => (RuleGroup::Stable, Ast, typing::assumed_size::AssumedSizeCharacterIntent),
        (Typing, "043") => (RuleGroup::Stable, Ast, typing::assumed_size::DeprecatedAssumedSizeCharacter),
        (Typing, "051") => (RuleGroup::Stable, Ast, typing::init_decls::InitialisationInDeclaration),
        (Typing, "061") => (RuleGroup::Stable, Ast, typing::external::ExternalProcedure),

        (Obsolescent, "001") => (RuleGroup::Stable, Ast, obsolescent::statement_functions::StatementFunction),
        (Obsolescent, "011") => (RuleGroup::Stable, Ast, obsolescent::common_blocks::CommonBlock),
        (Obsolescent, "021") => (RuleGroup::Stable, Ast, obsolescent::entry_statement::EntryStatement),

        (Precision, "001") => (RuleGroup::Stable, Ast, precision::kind_suffixes::NoRealSuffix),
        (Precision, "011") => (RuleGroup::Stable, Ast, precision::double_precision::DoublePrecision),
        (Precision, "021") => (RuleGroup::Stable, Ast, precision::implicit_kinds::ImplicitRealKind),

        (Modules, "001") => (RuleGroup::Stable, Ast, modules::external_functions::ProcedureNotInModule),
        (Modules, "011") => (RuleGroup::Stable, Ast, modules::use_statements::UseAll),
        (Modules, "021") => (RuleGroup::Preview, Ast, modules::accessibility_statements::MissingAccessibilityStatement),
        (Modules, "022") => (RuleGroup::Preview, Ast, modules::accessibility_statements::DefaultPublicAccessibility),

        // Rules for testing fortitude
        // Couldn't get a separate `Testing` category working for some reason
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9900") => (RuleGroup::Stable, Test, testing::test_rules::StableTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9901") => (RuleGroup::Stable, Test, testing::test_rules::StableTestRuleSafeFix),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9902") => (RuleGroup::Stable, Test, testing::test_rules::StableTestRuleUnsafeFix),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9903") => (RuleGroup::Stable, Test, testing::test_rules::StableTestRuleDisplayOnlyFix),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9904") => (RuleGroup::Preview, Test, testing::test_rules::PreviewTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9905") => (RuleGroup::Deprecated, Test, testing::test_rules::DeprecatedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9906") => (RuleGroup::Deprecated, Test, testing::test_rules::AnotherDeprecatedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9907") => (RuleGroup::Removed, Test, testing::test_rules::RemovedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9908") => (RuleGroup::Removed, Test, testing::test_rules::AnotherRemovedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9909") => (RuleGroup::Removed, Test, testing::test_rules::RedirectedFromTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9910") => (RuleGroup::Stable, Test, testing::test_rules::RedirectedToTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9911") => (RuleGroup::Removed, Test, testing::test_rules::RedirectedFromPrefixTestRule),
    })
}
