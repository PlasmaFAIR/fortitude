pub mod literal_kinds;
pub mod magic_io_unit;
pub mod star_kinds;

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

    #[test_case(Rule::NonPortableIoUnit, Path::new("PORT001.f90"))]
    #[test_case(Rule::LiteralKind, Path::new("PORT011.f90"))]
    #[test_case(Rule::LiteralKindSuffix, Path::new("PORT012.f90"))]
    #[test_case(Rule::StarKind, Path::new("PORT021.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("portability").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
