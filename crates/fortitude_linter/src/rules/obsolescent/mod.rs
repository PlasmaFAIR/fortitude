pub(crate) mod common_blocks;
pub(crate) mod computed_goto;
pub(crate) mod deprecated_character_syntax;
pub(crate) mod entry_statement;
pub(crate) mod mpi;
pub(crate) mod openmp;
pub(crate) mod pause_statement;
pub(crate) mod specific_names;
pub(crate) mod statement_functions;

#[cfg(test)]
mod tests {
    use std::convert::AsRef;
    use std::path::Path;

    use anyhow::Result;
    use insta::assert_snapshot;
    use test_case::test_case;

    use crate::apply_common_filters;
    use crate::registry::Rule;
    use crate::settings::CheckSettings;
    use crate::test::test_path;

    #[test_case(Rule::CommonBlock, Path::new("OB011.f90"))]
    #[test_case(Rule::EntryStatement, Path::new("OB021.f90"))]
    #[test_case(Rule::SpecificName, Path::new("OB031.f90"))]
    #[test_case(Rule::ComputedGoTo, Path::new("OB041.f90"))]
    #[test_case(Rule::PauseStatement, Path::new("OB051.f90"))]
    #[test_case(Rule::DeprecatedCharacterSyntax, Path::new("OB061.f90"))]
    #[test_case(Rule::DeprecatedMPIInclude, Path::new("OB201.f90"))]
    #[test_case(Rule::DeprecatedOmpInclude, Path::new("OB211.f90"))]

    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("obsolescent").join(path).as_path(),
            &CheckSettings::for_rule(rule_code),
        )?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }
}
