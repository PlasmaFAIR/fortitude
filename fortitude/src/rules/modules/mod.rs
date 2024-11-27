pub mod external_functions;
pub mod private_statement;
pub mod use_statements;

// TODO should be private by default, with explicit public interface
// TODO prefer 'end module {name}'
// TODO function is not used within a module and is not public

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

    #[test_case(Rule::ExternalFunction, Path::new("M001.f90"))]
    #[test_case(Rule::UseAll, Path::new("M011.f90"))]
    #[test_case(Rule::MissingPrivateStatement, Path::new("M021.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("modules").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
