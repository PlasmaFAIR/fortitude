pub mod common_blocks;
pub mod entry_statement;
pub mod statement_functions;

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

    #[test_case(Rule::StatementFunction, Path::new("OB001.f90"))]
    #[test_case(Rule::CommonBlock, Path::new("OB011.f90"))]
    #[test_case(Rule::EntryStatement, Path::new("OB021.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("obsolescent").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
