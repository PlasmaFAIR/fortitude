pub mod assumed_size_character_syntax;
pub mod common_blocks;
pub mod computed_goto;
pub mod entry_statement;
pub mod pause_statement;
pub mod specific_names;
pub mod statement_functions;

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

    #[test_case(Rule::CommonBlock, Path::new("OB011.f90"))]
    #[test_case(Rule::EntryStatement, Path::new("OB021.f90"))]
    #[test_case(Rule::SpecificName, Path::new("OB031.f90"))]
    #[test_case(Rule::ComputedGoTo, Path::new("OB041.f90"))]
    #[test_case(Rule::PauseStatement, Path::new("OB051.f90"))]
    #[test_case(Rule::DeprecatedAssumedSizeCharacter, Path::new("OB061.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("obsolescent").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
