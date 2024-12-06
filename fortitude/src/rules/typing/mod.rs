pub mod assumed_size;
pub mod external;
pub mod implicit_typing;
pub mod init_decls;
pub mod intent;
pub mod literal_kinds;
pub mod star_kinds;

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

    #[test_case(Rule::ImplicitTyping, Path::new("T001.f90"))]
    #[test_case(Rule::InterfaceImplicitTyping, Path::new("T002.f90"))]
    #[test_case(Rule::SuperfluousImplicitNone, Path::new("T003.f90"))]
    #[test_case(Rule::ImplicitExternalProcedures, Path::new("T004.f90"))]
    #[test_case(Rule::LiteralKind, Path::new("T011.f90"))]
    #[test_case(Rule::LiteralKindSuffix, Path::new("T012.f90"))]
    #[test_case(Rule::StarKind, Path::new("T021.f90"))]
    #[test_case(Rule::MissingIntent, Path::new("T031.f90"))]
    #[test_case(Rule::AssumedSize, Path::new("T041.f90"))]
    #[test_case(Rule::AssumedSizeCharacterIntent, Path::new("T042.f90"))]
    #[test_case(Rule::DeprecatedAssumedSizeCharacter, Path::new("T043.f90"))]
    #[test_case(Rule::InitialisationInDeclaration, Path::new("T051.f90"))]
    #[test_case(Rule::ExternalProcedure, Path::new("T061.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("typing").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }

    #[test_case(Rule::ImplicitTyping, Path::new("T001_ok.f90"))]
    #[test_case(Rule::InterfaceImplicitTyping, Path::new("T002_ok.f90"))]
    #[test_case(Rule::SuperfluousImplicitNone, Path::new("T003_ok.f90"))]
    fn rules_pass(rule_code: Rule, path: &Path) -> Result<()> {
        let diagnostics = test_path(
            Path::new("typing").join(path).as_path(),
            &[rule_code],
            &Settings::default(),
        )?;
        assert!(
            diagnostics.is_empty(),
            "Test source has no warnings, but some were raised:\n{diagnostics}"
        );
        Ok(())
    }
}
