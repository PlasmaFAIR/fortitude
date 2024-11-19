pub mod double_precision;
pub mod implicit_kinds;
pub mod kind_suffixes;

#[cfg(test)]
mod tests {
    use std::convert::AsRef;
    use std::path::Path;

    use anyhow::Result;
    use insta::assert_snapshot;
    use test_case::test_case;

    use crate::registry::Rule;
    use crate::settings::Settings;
    use crate::test::test_path;

    #[test_case(Rule::NoRealSuffix, Path::new("P001.f90"))]
    #[test_case(Rule::DoublePrecision, Path::new("P011.f90"))]
    #[test_case(Rule::ImplicitRealKind, Path::new("P021.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("precision").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
