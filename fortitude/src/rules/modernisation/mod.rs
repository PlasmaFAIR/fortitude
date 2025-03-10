pub mod double_precision;
pub mod include_statement;
pub mod old_style_array_literal;
pub mod relational_operators;

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

    #[test_case(Rule::DoublePrecision, Path::new("MOD001.f90"))]
    #[test_case(Rule::OldStyleArrayLiteral, Path::new("MOD011.f90"))]
    #[test_case(Rule::DeprecatedRelationalOperator, Path::new("MOD021.f90"))]
    #[test_case(Rule::IncludeStatement, Path::new("MOD031.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("modernisation").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
