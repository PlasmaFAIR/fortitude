#![allow(clippy::useless_format)]
/// A collection of all rules, and utilities to select a subset at runtime.
#[macro_use]
mod macros;
pub mod correctness;
pub mod error;
pub mod fortitude;
pub mod modernisation;
pub mod obsolescent;
pub mod portability;
pub mod style;
#[cfg(any(feature = "test-rules", test))]
pub mod testing;
pub mod utilities;

use crate::registry::Category;

use std::fmt::Formatter;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct NoqaCode(&'static str, &'static str);

impl NoqaCode {
    /// Return the prefix for the [`NoqaCode`], e.g., `SIM` for `SIM101`.
    pub fn prefix(&self) -> &'static str {
        self.0
    }

    /// Return the suffix for the [`NoqaCode`], e.g., `101` for `SIM101`.
    pub fn suffix(&self) -> &'static str {
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

/// Use `kind_ids!` in `AstRule::entrypoints` to convert a `Vec` of string
/// literals into a `Vec` of compiletime kind IDs. For unnamed keywords, append
/// ``| kw`` to the literal.
///
/// # Example
/// ```
/// use fortitude_linter::kind_ids;
/// use fortitude_macros::{kind, kw};
///
/// fn main() {
///     let ids = kind_ids![
///         "function",
///         "end" | kw
///     ];
///     let expected = vec![kind!("function"), kw!("end")];
///     assert_eq!(ids, expected);
/// }
/// ```
#[macro_export]
macro_rules! kind_ids {
    ($($kinds:literal $(| $modifier:tt)?),* $(,)?) => {
        vec![
            $(
                kind_ids!(@kind $kinds $(| $modifier)?),
            )*
        ]
    };
    (@kind $kind:literal) => {
        fortitude_macros::kind!($kind)
    };
    (@kind $kind:literal | kw) => {
        fortitude_macros::kw!($kind)
    };
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
        // error
        (Error, "000") => (RuleGroup::Stable, None, Default, error::ioerror::IoError),
        (Error, "001") => (RuleGroup::Stable, Ast, Default, error::syntax_error::SyntaxError),
        (Error, "011") => (RuleGroup::Stable, None, Default, error::invalid_character::InvalidCharacter),

        // correctness
        (Correctness, "001") => (RuleGroup::Stable, Ast, Default, correctness::implicit_typing::ImplicitTyping),
        (Correctness, "002") => (RuleGroup::Stable, Ast, Default, correctness::implicit_typing::InterfaceImplicitTyping),
        (Correctness, "003") => (RuleGroup::Stable, Ast, Default, correctness::implicit_typing::ImplicitExternalProcedures),
        (Correctness, "011") => (RuleGroup::Stable, Ast, Default, correctness::select_default::MissingDefaultCase),
        (Correctness, "021") => (RuleGroup::Stable, Ast, Optional, correctness::kind_suffixes::NoRealSuffix),
        (Correctness, "022") => (RuleGroup::Stable, Ast, Optional, correctness::implicit_kinds::ImplicitRealKind),
        (Correctness, "031") => (RuleGroup::Stable, Ast, Optional, correctness::magic_numbers::MagicNumberInArraySize),
        (Correctness, "032") => (RuleGroup::Stable, Ast, Optional, correctness::magic_numbers::MagicIoUnit),
        (Correctness, "043") => (RuleGroup::Stable, Ast, Optional, correctness::missing_io_specifier::MissingActionSpecifier),
        (Correctness, "051") => (RuleGroup::Stable, Ast, Default, correctness::trailing_backslash::TrailingBackslash),
        (Correctness, "061") => (RuleGroup::Stable, Ast, Default, correctness::intent::MissingIntent),
        (Correctness, "071") => (RuleGroup::Stable, Ast, Default, correctness::assumed_size::AssumedSize),
        (Correctness, "072") => (RuleGroup::Stable, Ast, Default, correctness::assumed_size::AssumedSizeCharacterIntent),
        (Correctness, "081") => (RuleGroup::Stable, Ast, Default, correctness::init_decls::InitialisationInDeclaration),
        (Correctness, "082") => (RuleGroup::Stable, Ast, Default, correctness::init_decls::PointerInitialisationInDeclaration),
        (Correctness, "091") => (RuleGroup::Stable, Ast, Default, correctness::external::ExternalProcedure),
        (Correctness, "092") => (RuleGroup::Stable, Ast, Default, correctness::external::ProcedureNotInModule),
        (Correctness, "101") => (RuleGroup::Stable, Ast, Default, correctness::derived_default_init::MissingDefaultPointerInitalisation),
        (Correctness, "121") => (RuleGroup::Stable, Ast, Default, correctness::use_statements::UseAll),
        (Correctness, "122") => (RuleGroup::Stable, Ast, Default, correctness::use_statements::MissingIntrinsic),
        (Correctness, "131") => (RuleGroup::Stable, Ast, Default, correctness::accessibility_statements::MissingAccessibilityStatement),
        (Correctness, "132") => (RuleGroup::Stable, Ast, Optional, correctness::accessibility_statements::DefaultPublicAccessibility),
        (Correctness, "141") => (RuleGroup::Stable, Ast, Default, correctness::exit_labels::MissingExitOrCycleLabel),
        (Correctness, "142") => (RuleGroup::Stable, Ast, Optional, correctness::exit_labels::ExitOrCycleInUnlabelledLoop),
        (Correctness, "143") => (RuleGroup::Stable, Ast, Default, correctness::exit_labels::MissingEndLabel),
        (Correctness, "151") => (RuleGroup::Stable, Ast, Default, correctness::conditionals::MisleadingInlineIfSemicolon),
        (Correctness, "152") => (RuleGroup::Stable, Ast, Default, correctness::conditionals::MisleadingInlineIfContinuation),
        (Correctness, "161") => (RuleGroup::Stable, Ast, Default, correctness::nonportable_shortcircuit_inquiry::NonportableShortcircuitInquiry),
        (Correctness, "171") => (RuleGroup::Stable, None, Optional, correctness::split_escaped_quote::SplitEscapedQuote),
        (Correctness, "181") => (RuleGroup::Stable, Ast, Default, correctness::error_handling::UncheckedStat),
        (Correctness, "182") => (RuleGroup::Stable, Ast, Default, correctness::error_handling::MultipleAllocationsWithStat),
        (Correctness, "183") => (RuleGroup::Stable, Ast, Optional, correctness::error_handling::StatWithoutMessage),
        (Correctness, "191") => (RuleGroup::Stable, Ast, Default, correctness::unreachable_statement::UnreachableStatement),
        (Correctness, "201") => (RuleGroup::Preview, None, Optional, correctness::shadowed_variable::ShadowedVariable),

        // modernisation
        (Modernisation, "001") => (RuleGroup::Stable, Ast, Optional, modernisation::double_precision::DoublePrecision),
        (Modernisation, "002") => (RuleGroup::Stable, Ast, Optional, modernisation::double_precision::DoublePrecisionLiteral),
        (Modernisation, "011") => (RuleGroup::Stable, Ast, Default, modernisation::old_style_array_literal::OldStyleArrayLiteral),
        (Modernisation, "021") => (RuleGroup::Stable, Ast, Default, modernisation::relational_operators::DeprecatedRelationalOperator),
        (Modernisation, "031") => (RuleGroup::Stable, Ast, Optional, modernisation::include_statement::IncludeStatement),
        (Modernisation, "041") => (RuleGroup::Stable, Ast, Default, modernisation::out_of_line_attribute::OutOfLineAttribute),
        (Modernisation, "051") => (RuleGroup::Preview, Ast, Default, modernisation::save::SuperfluousSave),
        (Modernisation, "201") => (RuleGroup::Stable, Ast, Optional, modernisation::mpi::OldMPIModule),

        // portability
        (Portability, "001") => (RuleGroup::Stable, Ast, Optional, portability::non_portable_io_unit::NonPortableIoUnit),
        (Portability, "011") => (RuleGroup::Stable, Ast, Default, portability::literal_kinds::LiteralKind),
        (Portability, "012") => (RuleGroup::Stable, Ast, Default, portability::literal_kinds::LiteralKindSuffix),
        (Portability, "021") => (RuleGroup::Stable, Ast, Default, portability::star_kinds::StarKind),
        (Portability, "031") => (RuleGroup::Stable, None, Default, portability::invalid_tab::InvalidTab),
        (Portability, "041") => (RuleGroup::Preview, Ast, Optional, portability::return_in_program::ReturnInProgram),
        (Portability, "051") => (RuleGroup::Preview, Ast, Optional, portability::unary_following_arithmetic::UnaryFollowingArithmetic),

        // style
        (Style, "001") => (RuleGroup::Stable, None, Default, style::line_length::LineTooLong),
        (Style, "002") => (RuleGroup::Stable, None, Default, style::whitespace::MissingNewlineAtEndOfFile),
        (Style, "061") => (RuleGroup::Stable, Ast, Default, style::end_statements::UnnamedEndStatement),
        (Style, "071") => (RuleGroup::Stable, Ast, Default, style::double_colon_in_decl::MissingDoubleColon),
        (Style, "081") => (RuleGroup::Stable, Ast, Default, style::semicolons::SuperfluousSemicolon),
        (Style, "082") => (RuleGroup::Stable, Ast, Optional, style::semicolons::MultipleStatementsPerLine),
        (Style, "091") => (RuleGroup::Stable, None, Default, style::file_extensions::NonStandardFileExtension),
        // There are likely to be many whitespace rules at some point, reserve S1xx for them
        (Style, "101") => (RuleGroup::Stable, None, Default, style::whitespace::TrailingWhitespace),
        (Style, "102") => (RuleGroup::Stable, Ast, Optional, style::whitespace::IncorrectSpaceBeforeComment),
        (Style, "103") => (RuleGroup::Stable, Ast, Optional, style::whitespace::IncorrectSpaceAroundDoubleColon),
        (Style, "104") => (RuleGroup::Stable, Ast, Optional, style::whitespace::IncorrectSpaceBetweenBrackets),
        (Style, "201") => (RuleGroup::Stable, Ast, Optional, style::implicit_none::SuperfluousImplicitNone),
        (Style, "211") => (RuleGroup::Stable, Ast, Optional, style::file_contents::MultipleModules),
        (Style, "212") => (RuleGroup::Stable, Ast, Optional, style::file_contents::ProgramWithModule),
        (Style, "221") => (RuleGroup::Stable, Ast, Optional, style::functions::FunctionMissingResult),
        (Style, "231") => (RuleGroup::Stable, Ast, Default, style::keywords::KeywordsMissingSpace),
        (Style, "232") => (RuleGroup::Stable, Ast, Default, style::keywords::KeywordHasWhitespace),
        (Style, "233") => (RuleGroup::Preview, Ast, Default, style::keywords::IncorrectKeywordCase),
        (Style, "241") => (RuleGroup::Stable, Ast, Default, style::strings::BadQuoteString),
        (Style, "242") => (RuleGroup::Stable, Ast, Optional, style::strings::AvoidableEscapedQuote),
        (Style, "251") => (RuleGroup::Stable, Ast, Optional, style::useless_return::UselessReturn),
        (Style, "252") => (RuleGroup::Stable, None, Optional, style::useless_return::SuperfluousElseReturn),
        (Style, "253") => (RuleGroup::Stable, None, Optional, style::useless_return::SuperfluousElseCycle),
        (Style, "254") => (RuleGroup::Stable, None, Optional, style::useless_return::SuperfluousElseExit),
        (Style, "255") => (RuleGroup::Stable, None, Optional, style::useless_return::SuperfluousElseStop),
        (Style, "261") => (RuleGroup::Stable, None, Default, style::inconsistent_dimension::InconsistentArrayDeclaration),
        (Style, "262") => (RuleGroup::Stable, None, Optional, style::inconsistent_dimension::MixedScalarArrayDeclaration),
        (Style, "263") => (RuleGroup::Stable, None, Optional, style::inconsistent_dimension::BadArrayDeclaration),
        (Style, "271") => (RuleGroup::Preview, Ast, Default, style::use_statement::UnsortedUses),
        (Style, "291") => (RuleGroup::Preview, Ast, Default, style::literals::BareDecimal),
        (Style, "301") => (RuleGroup::Preview, Ast, Optional, style::superfluous_while_true::SuperfluousWhileTrue),
        (Style, "311") => (RuleGroup::Preview, Ast, Default, style::keywords::KeywordReuse),
        (Style, "901") => (RuleGroup::Preview, Ast, Optional, style::complexity::TooComplex),
        (Style, "902") => (RuleGroup::Preview, Ast, Optional, style::complexity::TooManyArguments),

        // obsolescent
        (Obsolescent, "001") => (RuleGroup::Removed, Ast, Default, obsolescent::statement_functions::StatementFunction),
        (Obsolescent, "011") => (RuleGroup::Stable, Ast, Default, obsolescent::common_blocks::CommonBlock),
        (Obsolescent, "012") => (RuleGroup::Preview, Ast, Default, obsolescent::equivalence_statement::EquivalenceStatement),
        (Obsolescent, "013") => (RuleGroup::Preview, Ast, Default, obsolescent::block_data_construct::BlockDataConstruct),
        (Obsolescent, "021") => (RuleGroup::Stable, Ast, Default, obsolescent::entry_statement::EntryStatement),
        (Obsolescent, "031") => (RuleGroup::Stable, Ast, Default, obsolescent::specific_names::SpecificName),
        (Obsolescent, "041") => (RuleGroup::Stable, Ast, Default, obsolescent::computed_goto::ComputedGoTo),
        (Obsolescent, "051") => (RuleGroup::Stable, Ast, Default, obsolescent::pause_statement::PauseStatement),
        (Obsolescent, "061") => (RuleGroup::Stable, Ast, Default, obsolescent::deprecated_character_syntax::DeprecatedCharacterSyntax),
        (Obsolescent, "071") => (RuleGroup::Preview, Ast, Default, obsolescent::forall_statement::ForallStatement),
        (Obsolescent, "081") => (RuleGroup::Preview, Ast, Default, obsolescent::arithmetic_if::ArithmeticIf),
        (Obsolescent, "091") => (RuleGroup::Preview, Ast, Default, obsolescent::non_block_do::LabelledDoLoop),
        (Obsolescent, "092") => (RuleGroup::Preview, Ast, Default, obsolescent::non_block_do::SharedDoTermination),
        (Obsolescent, "093") => (RuleGroup::Preview, Ast, Default, obsolescent::non_block_do::BadDoTermination),
        (Obsolescent, "094") => (RuleGroup::Preview, Ast, Default, obsolescent::non_block_do::GotoEndDo),
        (Obsolescent, "201") => (RuleGroup::Stable, Ast, Default, obsolescent::mpi::DeprecatedMPIInclude),
        (Obsolescent, "211") => (RuleGroup::Stable, Ast, Default, obsolescent::openmp::DeprecatedOmpInclude),

        // fortitude
        (Fortitude, "001") => (RuleGroup::Stable, None, Default, fortitude::allow_comments::InvalidRuleCodeOrName),
        (Fortitude, "002") => (RuleGroup::Stable, None, Default, fortitude::allow_comments::UnusedAllowComment),
        (Fortitude, "003") => (RuleGroup::Stable, None, Default, fortitude::allow_comments::RedirectedAllowComment),
        (Fortitude, "004") => (RuleGroup::Stable, None, Default, fortitude::allow_comments::DuplicatedAllowComment),
        (Fortitude, "005") => (RuleGroup::Stable, None, Default, fortitude::allow_comments::DisabledAllowComment),

        // Rules for testing fortitude
        // Couldn't get a separate `Testing` category working for some reason
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9900") => (RuleGroup::Stable, None, Default, testing::test_rules::StableTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9901") => (RuleGroup::Stable, None, Default, testing::test_rules::StableTestRuleSafeFix),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9902") => (RuleGroup::Stable, None, Default, testing::test_rules::StableTestRuleUnsafeFix),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9903") => (RuleGroup::Stable, None, Default, testing::test_rules::StableTestRuleDisplayOnlyFix),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9911") => (RuleGroup::Preview, None, Default, testing::test_rules::PreviewTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9920") => (RuleGroup::Deprecated, None, Default, testing::test_rules::DeprecatedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9921") => (RuleGroup::Deprecated, None, Default, testing::test_rules::AnotherDeprecatedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9930") => (RuleGroup::Removed, None, Default, testing::test_rules::RemovedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9931") => (RuleGroup::Removed, None, Default, testing::test_rules::AnotherRemovedTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9940") => (RuleGroup::Removed, None, Default, testing::test_rules::RedirectedFromTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9950") => (RuleGroup::Stable, None, Default, testing::test_rules::RedirectedToTestRule),
        #[cfg(any(feature = "test-rules", test))]
        (Fortitude, "9960") => (RuleGroup::Removed, None, Default, testing::test_rules::RedirectedFromPrefixTestRule),
    })
}
