pub mod invalid_tab;
pub(crate) mod literal_kinds;
pub(crate) mod non_portable_io_unit;
pub(crate) mod star_kinds;

#[cfg(test)]
mod tests {
    use std::convert::AsRef;
    use std::path::Path;

    use anyhow::Result;
    use insta::assert_snapshot;
    use test_case::test_case;

    use crate::apply_common_filters;
    use crate::registry::Rule;
    use crate::rules::portability;
    use crate::settings::CheckSettings;
    use crate::test::test_path;

    #[test_case(Rule::NonPortableIoUnit, Path::new("PORT001.f90"))]
    #[test_case(Rule::LiteralKind, Path::new("PORT011.f90"))]
    #[test_case(Rule::LiteralKindSuffix, Path::new("PORT012.f90"))]
    #[test_case(Rule::StarKind, Path::new("PORT021.f90"))]
    #[test_case(Rule::InvalidTab, Path::new("PORT031.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("portability").join(path).as_path(),
            &CheckSettings::for_rule(rule_code),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }

    #[test]
    fn warn_port001_no_cray_units() -> Result<()> {
        let rule_code = Rule::NonPortableIoUnit;
        let path = Path::new("PORT001.f90");
        let snapshot = format!(
            "{}_{}_no_cray_units",
            rule_code.as_ref(),
            path.to_string_lossy()
        );

        let settings = CheckSettings {
            portability: portability::settings::Settings {
                allow_cray_file_units: true,
            },
            ..CheckSettings::for_rule(rule_code)
        };
        let diagnostics = test_path(Path::new("portability").join(path).as_path(), &settings)?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}

pub mod settings {
    use crate::display_settings;
    use ruff_macros::CacheKey;
    use std::fmt::{Display, Formatter};

    #[derive(Debug, Clone, Default, CacheKey)]
    pub struct Settings {
        pub allow_cray_file_units: bool,
    }

    impl Display for Settings {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            display_settings! {
                formatter = f,
                namespace = "check.portability",
                fields = [self.allow_cray_file_units]
            }
            Ok(())
        }
    }
}
