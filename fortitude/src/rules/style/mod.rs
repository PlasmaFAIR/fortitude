pub mod double_colon_in_decl;
pub mod end_statements;
pub mod file_contents;
pub mod file_extensions;
pub mod implicit_none;
pub mod line_length;
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
    #[test_case(Rule::UnnamedEndStatement, Path::new("S061.f90"))]
    #[test_case(Rule::MissingDoubleColon, Path::new("S071.f90"))]
    #[test_case(Rule::SuperfluousSemicolon, Path::new("S081.f90"))]
    #[test_case(Rule::MultipleStatementsPerLine, Path::new("S082.f90"))]
    #[test_case(Rule::TrailingWhitespace, Path::new("S101.f90"))]
    #[test_case(Rule::IncorrectSpaceBeforeComment, Path::new("S102.f90"))]
    #[test_case(Rule::SuperfluousImplicitNone, Path::new("S201.f90"))]
    #[test_case(Rule::MultipleModules, Path::new("S211.f90"))]
    #[test_case(Rule::ProgramWithModule, Path::new("S212.f90"))]
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

    #[test_case(Rule::SuperfluousImplicitNone, Path::new("S201_ok.f90"))]
    fn rules_pass(rule_code: Rule, path: &Path) -> Result<()> {
        let diagnostics = test_path(
            Path::new("style").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
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
