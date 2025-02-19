pub mod implicit_kinds;
pub mod kind_suffixes;
pub mod select_default;
pub mod trailing_backslash;

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

    #[test_case(Rule::MissingDefaultCase, Path::new("C001.f90"))]
    #[test_case(Rule::TrailingBackslash, Path::new("C011.F90"))]
    #[test_case(Rule::NoRealSuffix, Path::new("C021.f90"))]
    #[test_case(Rule::ImplicitRealKind, Path::new("C022.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("correctness").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
