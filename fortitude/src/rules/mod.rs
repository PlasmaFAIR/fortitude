#![allow(clippy::useless_format)]
/// A collection of all rules, and utilities to select a subset at runtime.
#[macro_use]
mod macros;
pub(crate) mod correctness;
pub(crate) mod error;
pub(crate) mod modernization;
pub(crate) mod modules;
pub(crate) mod obsolescent;
pub(crate) mod portability;
pub(crate) mod style;
pub(crate) mod testing;
pub mod utilities;
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
        (Error, "000") => (RuleGroup::Stable, None, Default, error::ioerror::IoError),
        (Error, "001") => (RuleGroup::Stable, Ast, Default, error::syntax_error::SyntaxError),
        (Error, "011") => (RuleGroup::Stable, None, Default, error::allow_comments::InvalidRuleCodeOrName),

        (Correctness, "001") => (RuleGroup::Preview, Ast, Default, correctness::select_default::MissingDefaultCase),
        (Correctness, "011") => (RuleGroup::Preview, Ast, Default, correctness::trailing_backslash::TrailingBackslash),
        (Correctness, "021") => (RuleGroup::Stable, Ast, Optional, correctness::kind_suffixes::NoRealSuffix),
        (Correctness, "022") => (RuleGroup::Stable, Ast, Optional, correctness::implicit_kinds::ImplicitRealKind),
        (Correctness, "031") => (RuleGroup::Preview, Ast, Optional, correctness::magic_numbers::MagicNumberInArraySize),
        (Correctness, "032") => (RuleGroup::Preview, Ast, Optional, correctness::magic_numbers::MagicIoUnit),
        (Correctness, "043") => (RuleGroup::Preview, Ast, Optional, correctness::missing_io_specifier::MissingActionSpecifier),
        (Correctness, "051") => (RuleGroup::Stable, Ast, Default, correctness::implicit_typing::ImplicitTyping),
        (Correctness, "052") => (RuleGroup::Stable, Ast, Default, correctness::implicit_typing::InterfaceImplicitTyping),
        (Correctness, "053") => (RuleGroup::Preview, Ast, Default, correctness::implicit_typing::ImplicitExternalProcedures),
        (Correctness, "061") => (RuleGroup::Stable, Ast, Default, correctness::intent::MissingIntent),
        (Correctness, "071") => (RuleGroup::Stable, Ast, Default, correctness::assumed_size::AssumedSize),
        (Correctness, "072") => (RuleGroup::Stable, Ast, Default, correctness::assumed_size::AssumedSizeCharacterIntent),
        (Correctness, "081") => (RuleGroup::Stable, Ast, Default, correctness::init_decls::InitialisationInDeclaration),
        (Correctness, "091") => (RuleGroup::Stable, Ast, Default, correctness::external::ExternalProcedure),
        (Correctness, "101") => (RuleGroup::Preview, Ast, Default, correctness::derived_default_init::MissingDefaultPointerInitalisation),
        (Correctness, "111") => (RuleGroup::Stable, Ast, Default, correctness::external_functions::ProcedureNotInModule),
        (Correctness, "121") => (RuleGroup::Stable, Ast, Default, correctness::use_statements::UseAll),
        (Correctness, "122") => (RuleGroup::Preview, Ast, Default, correctness::use_statements::MissingIntrinsic),
        (Correctness, "131") => (RuleGroup::Preview, Ast, Default, correctness::accessibility_statements::MissingAccessibilityStatement),
        (Correctness, "132") => (RuleGroup::Preview, Ast, Optional, correctness::accessibility_statements::DefaultPublicAccessibility),
        (Correctness, "141") => (RuleGroup::Preview, Ast, Optional, correctness::include_statement::IncludeStatement),

        
        (Modernization, "001") => (RuleGroup::Stable, Ast, Optional, modernization::double_precision::DoublePrecision),
        
        (Portability, "001") => (RuleGroup::Preview, Ast, Optional, portability::magic_io_unit::NonPortableIoUnit),
        (Portability, "011") => (RuleGroup::Stable, Ast, Default, portability::literal_kinds::LiteralKind),
        (Portability, "012") => (RuleGroup::Stable, Ast, Default, portability::literal_kinds::LiteralKindSuffix),
        (Portability, "021") => (RuleGroup::Stable, Ast, Default, portability::star_kinds::StarKind),

        (Style, "001") => (RuleGroup::Stable, Text, Default, style::line_length::LineTooLong),
        (Style, "021") => (RuleGroup::Stable, Ast, Default, style::exit_labels::MissingExitOrCycleLabel),
        (Style, "041") => (RuleGroup::Stable, Ast, Default, style::old_style_array_literal::OldStyleArrayLiteral),
        (Style, "051") => (RuleGroup::Stable, Ast, Default, style::relational_operators::DeprecatedRelationalOperator),
        (Style, "061") => (RuleGroup::Stable, Ast, Default, style::end_statements::UnnamedEndStatement),
        (Style, "071") => (RuleGroup::Stable, Ast, Default, style::double_colon_in_decl::MissingDoubleColon),
        (Style, "081") => (RuleGroup::Preview, Ast, Default, style::semicolons::SuperfluousSemicolon),
        (Style, "082") => (RuleGroup::Preview, Ast, Optional, style::semicolons::MultipleStatementsPerLine),
        (Style, "091") => (RuleGroup::Stable, Path, Default, style::file_extensions::NonStandardFileExtension),
        // There are likely to be many whitespace rules at some point, reserve S1xx for them
        (Style, "101") => (RuleGroup::Stable, Text, Default, style::whitespace::TrailingWhitespace),
        (Style, "102") => (RuleGroup::Stable, Ast, Optional, style::whitespace::IncorrectSpaceBeforeComment),
        (Style, "201") => (RuleGroup::Stable, Ast, Default, style::implicit_none::SuperfluousImplicitNone),

        (Obsolescent, "001") => (RuleGroup::Stable, Ast, Default, obsolescent::statement_functions::StatementFunction),
        (Obsolescent, "011") => (RuleGroup::Stable, Ast, Default, obsolescent::common_blocks::CommonBlock),
        (Obsolescent, "021") => (RuleGroup::Stable, Ast, Default, obsolescent::entry_statement::EntryStatement),
        (Obsolescent, "031") => (RuleGroup::Preview, Ast, Default, obsolescent::specific_names::SpecificName),
        (Obsolescent, "041") => (RuleGroup::Preview, Ast, Default, obsolescent::computed_goto::ComputedGoTo),
        (Obsolescent, "051") => (RuleGroup::Stable, Ast, Default, obsolescent::pause_statement::PauseStatement),
        (Obsolescent, "061") => (RuleGroup::Stable, Ast, Default, obsolescent::assumed_size_character_syntax::DeprecatedAssumedSizeCharacter),


        (Modules, "041") => (RuleGroup::Preview, Ast, Optional, modules::file_contents::MultipleModules),
        (Modules, "042") => (RuleGroup::Preview, Ast, Optional, modules::file_contents::ProgramWithModule),


        // Rules for testing fortitude
        // Couldn't get a separate `Testing` category working for some reason
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9900") => (RuleGroup::Stable, None, Default, testing::test_rules::StableTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9901") => (RuleGroup::Stable, None, Default, testing::test_rules::StableTestRuleSafeFix),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9902") => (RuleGroup::Stable, None, Default, testing::test_rules::StableTestRuleUnsafeFix),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9903") => (RuleGroup::Stable, None, Default, testing::test_rules::StableTestRuleDisplayOnlyFix),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9911") => (RuleGroup::Preview, None, Default, testing::test_rules::PreviewTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9920") => (RuleGroup::Deprecated, None, Default, testing::test_rules::DeprecatedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9921") => (RuleGroup::Deprecated, None, Default, testing::test_rules::AnotherDeprecatedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9930") => (RuleGroup::Removed, None, Default, testing::test_rules::RemovedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9931") => (RuleGroup::Removed, None, Default, testing::test_rules::AnotherRemovedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9940") => (RuleGroup::Removed, None, Default, testing::test_rules::RedirectedFromTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9950") => (RuleGroup::Stable, None, Default, testing::test_rules::RedirectedToTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Error, "9960") => (RuleGroup::Removed, None, Default, testing::test_rules::RedirectedFromPrefixTestRule),
    })
}
