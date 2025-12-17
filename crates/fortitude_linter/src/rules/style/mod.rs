pub(crate) mod double_colon_in_decl;
pub(crate) mod end_statements;
pub(crate) mod file_contents;
pub(crate) mod file_extensions;
pub(crate) mod functions;
pub(crate) mod implicit_none;
pub mod inconsistent_dimension;
pub mod keywords;
pub(crate) mod line_length;
pub(crate) mod semicolons;
pub mod strings;
pub mod useless_return;
pub(crate) mod whitespace;

#[cfg(test)]
mod tests {
    use std::convert::AsRef;
    use std::path::Path;

    use anyhow::Result;
    use insta::assert_snapshot;
    use test_case::test_case;

    use crate::apply_common_filters;
    use crate::registry::Rule;
    use crate::rules::style::inconsistent_dimension::settings::PreferAttribute;
    use crate::rules::style::{inconsistent_dimension, keywords, strings};
    use crate::settings::CheckSettings;
    use crate::test::test_path;

    use super::strings::settings::Quote;

    #[test_case(Rule::LineTooLong, Path::new("S001.f90"))]
    #[test_case(Rule::MissingNewlineAtEndOfFile, Path::new("S002.f90"))]
    #[test_case(Rule::UnnamedEndStatement, Path::new("S061.f90"))]
    #[test_case(Rule::MissingDoubleColon, Path::new("S071.f90"))]
    #[test_case(Rule::SuperfluousSemicolon, Path::new("S081.f90"))]
    #[test_case(Rule::MultipleStatementsPerLine, Path::new("S082.f90"))]
    #[test_case(Rule::TrailingWhitespace, Path::new("S101.f90"))]
    #[test_case(Rule::IncorrectSpaceBeforeComment, Path::new("S102.f90"))]
    #[test_case(Rule::IncorrectSpaceAroundDoubleColon, Path::new("S103.f90"))]
    #[test_case(Rule::IncorrectSpaceBetweenBrackets, Path::new("S104.f90"))]
    #[test_case(Rule::SuperfluousImplicitNone, Path::new("S201.f90"))]
    #[test_case(Rule::MultipleModules, Path::new("S211.f90"))]
    #[test_case(Rule::ProgramWithModule, Path::new("S212.f90"))]
    #[test_case(Rule::FunctionMissingResult, Path::new("S221.f90"))]
    #[test_case(Rule::KeywordsMissingSpace, Path::new("S231.f90"))]
    #[test_case(Rule::KeywordHasWhitespace, Path::new("S231.f90"))]
    #[test_case(Rule::BadQuoteString, Path::new("S241.f90"))]
    #[test_case(Rule::AvoidableEscapedQuote, Path::new("S242.f90"))]
    #[test_case(Rule::UselessReturn, Path::new("S251.f90"))]
    #[test_case(Rule::SuperfluousElseReturn, Path::new("S252.f90"))]
    #[test_case(Rule::SuperfluousElseCycle, Path::new("S253.f90"))]
    #[test_case(Rule::SuperfluousElseExit, Path::new("S254.f90"))]
    #[test_case(Rule::SuperfluousElseStop, Path::new("S255.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("style").join(path).as_path(),
            &CheckSettings::for_rule(rule_code),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }

    #[test_case(Rule::SuperfluousImplicitNone, Path::new("S201_ok.f90"))]
    fn rules_pass(rule_code: Rule, path: &Path) -> Result<()> {
        let diagnostics = test_path(
            Path::new("style").join(path).as_path(),
            &CheckSettings::for_rule(rule_code),
        )?;
        assert!(
            diagnostics.is_empty(),
            "Test source has no warnings, but some were raised:\n{diagnostics}"
        );
        Ok(())
    }

    #[test_case(Rule::LineTooLong, Path::new("S001_line_length_20.f90"))]
    fn line_too_long_line_length_20(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());

        let settings = CheckSettings {
            line_length: 20,
            ..CheckSettings::for_rule(rule_code)
        };
        let diagnostics = test_path(Path::new("style").join(path).as_path(), &settings)?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }

    #[test_case(Rule::KeywordsMissingSpace, Path::new("S231.f90"))]
    #[test_case(Rule::KeywordHasWhitespace, Path::new("S231.f90"))]
    fn keyword_whitespace_include_inout_goto(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!(
            "{}_{}_include_inout_goto",
            rule_code.as_ref(),
            path.to_string_lossy()
        );

        #[allow(clippy::needless_update)]
        let settings = CheckSettings {
            keyword_whitespace: keywords::settings::Settings {
                inout_with_space: true,
                goto_with_space: true,
            },
            ..CheckSettings::for_rule(rule_code)
        };
        let diagnostics = test_path(Path::new("style").join(path).as_path(), &settings)?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }

    #[test_case(Rule::BadQuoteString, Path::new("S241.f90"))]
    fn bad_quote_string_single_quotes(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!(
            "{}_{}_include_inout_goto",
            rule_code.as_ref(),
            path.to_string_lossy()
        );

        let settings = CheckSettings {
            strings: strings::settings::Settings {
                quotes: Quote::Single,
            },
            ..CheckSettings::for_rule(rule_code)
        };
        let diagnostics = test_path(Path::new("style").join(path).as_path(), &settings)?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }

    #[test_case(
        Rule::InconsistentArrayDeclaration,
        Path::new("S261.f90"),
        PreferAttribute::Always
    )]
    #[test_case(
        Rule::InconsistentArrayDeclaration,
        Path::new("S261.f90"),
        PreferAttribute::Never
    )]
    #[test_case(
        Rule::InconsistentArrayDeclaration,
        Path::new("S261.f90"),
        PreferAttribute::Keep
    )]
    #[test_case(
        Rule::MixedScalarArrayDeclaration,
        Path::new("S262.f90"),
        PreferAttribute::Always
    )]
    #[test_case(
        Rule::MixedScalarArrayDeclaration,
        Path::new("S262.f90"),
        PreferAttribute::Never
    )]
    #[test_case(
        Rule::MixedScalarArrayDeclaration,
        Path::new("S262.f90"),
        PreferAttribute::Keep
    )]
    #[test_case(
        Rule::BadArrayDeclaration,
        Path::new("S263.f90"),
        PreferAttribute::Always
    )]
    #[test_case(
        Rule::BadArrayDeclaration,
        Path::new("S263.f90"),
        PreferAttribute::Never
    )]
    // This one should be a no-op
    #[test_case(
        Rule::BadArrayDeclaration,
        Path::new("S263.f90"),
        PreferAttribute::Keep
    )]
    fn inconsistent_dimensions(
        rule_code: Rule,
        path: &Path,
        prefer_attribute: PreferAttribute,
    ) -> Result<()> {
        let snapshot = format!(
            "{}_{}_{}",
            rule_code.as_ref(),
            path.to_string_lossy(),
            prefer_attribute
        );

        let settings = CheckSettings {
            inconsistent_dimension: inconsistent_dimension::settings::Settings { prefer_attribute },
            ..CheckSettings::for_rule(rule_code)
        };
        let diagnostics = test_path(Path::new("style").join(path).as_path(), &settings)?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
