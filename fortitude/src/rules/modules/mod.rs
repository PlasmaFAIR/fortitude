pub mod accessibility_statements;
pub mod external_functions;
pub mod file_contents;
pub mod include_statement;
pub mod use_statements;

#[cfg(test)]
mod tests {
    use std::convert::AsRef;
    use std::path::Path;

    use anyhow::Result;
    use insta::assert_snapshot;
    use test_case::test_case;

    use crate::apply_common_filters;
    use crate::registry::Rule;
    use crate::settings::Settings;
    use crate::test::test_path;

    #[test_case(Rule::ProcedureNotInModule, Path::new("M001.f90"))]
    #[test_case(Rule::UseAll, Path::new("M011.f90"))]
    #[test_case(Rule::MissingIntrinsic, Path::new("M012.f90"))]
    #[test_case(Rule::MissingAccessibilityStatement, Path::new("M021.f90"))]
    #[test_case(Rule::DefaultPublicAccessibility, Path::new("M022.f90"))]
    #[test_case(Rule::IncludeStatement, Path::new("M031.f90"))]
    #[test_case(Rule::MultipleModules, Path::new("M041.f90"))]
    #[test_case(Rule::ProgramWithModule, Path::new("M042.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("modules").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
