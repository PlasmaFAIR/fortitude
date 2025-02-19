#![allow(clippy::useless_format)]
/// A collection of all rules, and utilities to select a subset at runtime.
#[macro_use]
mod macros;
pub(crate) mod correctness;
pub(crate) mod error;
pub(crate) mod filesystem;
pub(crate) mod io;
pub(crate) mod modernization;
pub(crate) mod modules;
pub(crate) mod obsolescent;
pub(crate) mod precision;
pub(crate) mod readability;
pub(crate) mod style;
pub(crate) mod testing;
pub(crate) mod typing;
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
        
        (Modernization, "001") => (RuleGroup::Stable, Ast, Optional, modernization::double_precision::DoublePrecision),

        (Filesystem, "001") => (RuleGroup::Stable, Path, Default,filesystem::extensions::NonStandardFileExtension),

        (Style, "001") => (RuleGroup::Stable, Text, Default, style::line_length::LineTooLong),
        (Style, "021") => (RuleGroup::Stable, Ast, Default, style::exit_labels::MissingExitOrCycleLabel),
        (Style, "041") => (RuleGroup::Stable, Ast, Default, style::old_style_array_literal::OldStyleArrayLiteral),
        (Style, "051") => (RuleGroup::Stable, Ast, Default, style::relational_operators::DeprecatedRelationalOperator),
        (Style, "061") => (RuleGroup::Stable, Ast, Default, style::end_statements::UnnamedEndStatement),
        (Style, "071") => (RuleGroup::Stable, Ast, Default, style::double_colon_in_decl::MissingDoubleColon),
        (Style, "081") => (RuleGroup::Preview, Ast, Default, style::semicolons::SuperfluousSemicolon),
        (Style, "082") => (RuleGroup::Preview, Ast, Optional, style::semicolons::MultipleStatementsPerLine),
        (Style, "101") => (RuleGroup::Stable, Text, Default, style::whitespace::TrailingWhitespace),
        (Style, "102") => (RuleGroup::Stable, Ast, Optional, style::whitespace::IncorrectSpaceBeforeComment),

        (Typing, "001") => (RuleGroup::Stable, Ast, Default, typing::implicit_typing::ImplicitTyping),
        (Typing, "002") => (RuleGroup::Stable, Ast, Default, typing::implicit_typing::InterfaceImplicitTyping),
        (Typing, "003") => (RuleGroup::Stable, Ast, Default, typing::implicit_typing::SuperfluousImplicitNone),
        (Typing, "004") => (RuleGroup::Preview, Ast, Default, typing::implicit_typing::ImplicitExternalProcedures),
        (Typing, "011") => (RuleGroup::Stable, Ast, Default, typing::literal_kinds::LiteralKind),
        (Typing, "012") => (RuleGroup::Stable, Ast, Default, typing::literal_kinds::LiteralKindSuffix),
        (Typing, "021") => (RuleGroup::Stable, Ast, Default, typing::star_kinds::StarKind),
        (Typing, "031") => (RuleGroup::Stable, Ast, Default, typing::intent::MissingIntent),
        (Typing, "041") => (RuleGroup::Stable, Ast, Default, typing::assumed_size::AssumedSize),
        (Typing, "042") => (RuleGroup::Stable, Ast, Default, typing::assumed_size::AssumedSizeCharacterIntent),
        (Typing, "043") => (RuleGroup::Stable, Ast, Default, typing::assumed_size::DeprecatedAssumedSizeCharacter),
        (Typing, "051") => (RuleGroup::Stable, Ast, Default, typing::init_decls::InitialisationInDeclaration),
        (Typing, "061") => (RuleGroup::Stable, Ast, Default, typing::external::ExternalProcedure),
        (Typing, "071") => (RuleGroup::Preview, Ast, Default, typing::derived_default_init::MissingDefaultPointerInitalisation),

        (Obsolescent, "001") => (RuleGroup::Stable, Ast, Default, obsolescent::statement_functions::StatementFunction),
        (Obsolescent, "011") => (RuleGroup::Stable, Ast, Default, obsolescent::common_blocks::CommonBlock),
        (Obsolescent, "021") => (RuleGroup::Stable, Ast, Default, obsolescent::entry_statement::EntryStatement),
        (Obsolescent, "031") => (RuleGroup::Preview, Ast, Default, obsolescent::specific_names::SpecificName),
        (Obsolescent, "041") => (RuleGroup::Preview, Ast, Default, obsolescent::computed_goto::ComputedGoTo),
        (Obsolescent, "051") => (RuleGroup::Stable, Ast, Default, obsolescent::pause_statement::PauseStatement),

        (Precision, "001") => (RuleGroup::Stable, Ast, Optional, precision::kind_suffixes::NoRealSuffix),
        (Precision, "021") => (RuleGroup::Stable, Ast, Optional, precision::implicit_kinds::ImplicitRealKind),

        (Modules, "001") => (RuleGroup::Stable, Ast, Default, modules::external_functions::ProcedureNotInModule),
        (Modules, "011") => (RuleGroup::Stable, Ast, Default, modules::use_statements::UseAll),
        (Modules, "012") => (RuleGroup::Preview, Ast, Default, modules::use_statements::MissingIntrinsic),
        (Modules, "021") => (RuleGroup::Preview, Ast, Default, modules::accessibility_statements::MissingAccessibilityStatement),
        (Modules, "022") => (RuleGroup::Preview, Ast, Optional, modules::accessibility_statements::DefaultPublicAccessibility),
        (Modules, "031") => (RuleGroup::Preview, Ast, Optional, modules::include_statement::IncludeStatement),
        (Modules, "041") => (RuleGroup::Preview, Ast, Optional, modules::file_contents::MultipleModules),
        (Modules, "042") => (RuleGroup::Preview, Ast, Optional, modules::file_contents::ProgramWithModule),

        (Io, "001") => (RuleGroup::Preview, Ast, Optional, io::missing_specifier::MissingActionSpecifier),
        (Io, "011") => (RuleGroup::Preview, Ast, Optional, io::magic_io_unit::MagicIoUnit),
        (Io, "012") => (RuleGroup::Preview, Ast, Optional, io::magic_io_unit::NonPortableIoUnit),

        (Readability, "001") => (RuleGroup::Preview, Ast, Optional, readability::magic_numbers::MagicNumberInArraySize),

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
