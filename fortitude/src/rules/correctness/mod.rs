pub mod accessibility_statements;
pub mod assumed_size;
pub mod derived_default_init;
pub mod exit_labels;
pub mod external;
pub mod implicit_kinds;
pub mod implicit_typing;
pub mod init_decls;
pub mod intent;
pub mod kind_suffixes;
pub mod magic_numbers;
pub mod missing_io_specifier;
pub mod select_default;
pub mod trailing_backslash;
pub mod use_statements;

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

    #[test_case(Rule::ImplicitTyping, Path::new("C001.f90"))]
    #[test_case(Rule::InterfaceImplicitTyping, Path::new("C002.f90"))]
    #[test_case(Rule::ImplicitExternalProcedures, Path::new("C003.f90"))]
    #[test_case(Rule::MissingDefaultCase, Path::new("C011.f90"))]
    #[test_case(Rule::NoRealSuffix, Path::new("C021.f90"))]
    #[test_case(Rule::ImplicitRealKind, Path::new("C022.f90"))]
    #[test_case(Rule::MagicNumberInArraySize, Path::new("C031.f90"))]
    #[test_case(Rule::MagicIoUnit, Path::new("C032.f90"))]
    #[test_case(Rule::MissingActionSpecifier, Path::new("C041.f90"))]
    #[test_case(Rule::TrailingBackslash, Path::new("C051.F90"))]
    #[test_case(Rule::MissingIntent, Path::new("C061.f90"))]
    #[test_case(Rule::AssumedSize, Path::new("C071.f90"))]
    #[test_case(Rule::AssumedSizeCharacterIntent, Path::new("C072.f90"))]
    #[test_case(Rule::InitialisationInDeclaration, Path::new("C081.f90"))]
    #[test_case(Rule::ExternalProcedure, Path::new("C091.f90"))]
    #[test_case(Rule::ProcedureNotInModule, Path::new("C092.f90"))]
    #[test_case(Rule::MissingDefaultPointerInitalisation, Path::new("C101.f90"))]
    #[test_case(Rule::UseAll, Path::new("C121.f90"))]
    #[test_case(Rule::MissingIntrinsic, Path::new("C122.f90"))]
    #[test_case(Rule::MissingAccessibilityStatement, Path::new("C131.f90"))]
    #[test_case(Rule::DefaultPublicAccessibility, Path::new("C132.f90"))]
    #[test_case(Rule::MissingExitOrCycleLabel, Path::new("C141.f90"))]
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

    #[test_case(Rule::ImplicitTyping, Path::new("C001_ok.f90"))]
    #[test_case(Rule::InterfaceImplicitTyping, Path::new("C002_ok.f90"))]
    fn rules_pass(rule_code: Rule, path: &Path) -> Result<()> {
        let diagnostics = test_path(
            Path::new("correctness").join(path).as_path(),
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
