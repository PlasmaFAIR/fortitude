pub(crate) mod allow_comments;

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

    #[test_case(Rule::InvalidRuleCodeOrName, Path::new("FORT001.f90"))]
    #[test_case(Rule::UnusedAllowComment, Path::new("FORT002.f90"))]
    #[test_case(Rule::RedirectedAllowComment, Path::new("FORT003.f90"))]
    #[test_case(Rule::DuplicatedAllowComment, Path::new("FORT004.f90"))]
    #[test_case(Rule::DisabledAllowComment, Path::new("FORT005.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("fortitude").join(path).as_path(),
            &[
                // We enable all these related rules here as we should
                // only enabled exactly one of them
                Rule::InvalidRuleCodeOrName,
                Rule::UnusedAllowComment,
                Rule::RedirectedAllowComment,
                Rule::DuplicatedAllowComment,
                Rule::DisabledAllowComment,
                // and we add this one so we can have something that
                // gets used
                Rule::ImplicitTyping,
            ],
            &Settings::default(),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
