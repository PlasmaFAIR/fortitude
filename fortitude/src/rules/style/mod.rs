pub mod double_colon_in_decl;
pub mod end_statements;
pub mod exit_labels;
pub mod line_length;
pub mod old_style_array_literal;
pub mod relational_operators;
pub mod semicolons;
pub mod whitespace;

#[cfg(test)]
mod tests {
    use std::convert::AsRef;
    use std::path::Path;

    use anyhow::Result;
    use insta::assert_snapshot;
    use test_case::test_case;

    use crate::apply_common_filters;
    use crate::registry::Rule;
    use crate::settings::{CheckSettings, Settings};
    use crate::test::test_path;

    #[test_case(Rule::LineTooLong, Path::new("S001.f90"))]
    #[test_case(Rule::MissingExitOrCycleLabel, Path::new("S021.f90"))]
    #[test_case(Rule::OldStyleArrayLiteral, Path::new("S041.f90"))]
    #[test_case(Rule::DeprecatedRelationalOperator, Path::new("S051.f90"))]
    #[test_case(Rule::UnnamedEndStatement, Path::new("S061.f90"))]
    #[test_case(Rule::MissingDoubleColon, Path::new("S071.f90"))]
    #[test_case(Rule::SuperfluousSemicolon, Path::new("S081.f90"))]
    #[test_case(Rule::MultipleStatementsPerLine, Path::new("S082.f90"))]
    #[test_case(Rule::TrailingWhitespace, Path::new("S101.f90"))]
    #[test_case(Rule::IncorrectSpaceBeforeComment, Path::new("S102.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("style").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }

    #[test_case(Rule::LineTooLong, Path::new("S001_line_length_20.f90"))]
    fn line_too_long_line_length_20(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());

        let default = Settings::default();
        #[allow(clippy::needless_update)]
        let settings = Settings {
            check: CheckSettings {
                line_length: 20,
                ..default.check
            },
            ..default
        };
        let diagnostics = test_path(
            Path::new("style").join(path).as_path(),
            &[rule_code],
            &settings,
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
