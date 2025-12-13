pub(crate) mod accessibility_statements;
pub(crate) mod assumed_size;
pub(crate) mod conditionals;
pub(crate) mod derived_default_init;
pub(crate) mod error_handling;
pub mod exit_labels;
pub(crate) mod external;
pub(crate) mod implicit_kinds;
pub(crate) mod implicit_typing;
pub(crate) mod init_decls;
pub(crate) mod intent;
pub(crate) mod kind_suffixes;
pub(crate) mod magic_numbers;
pub(crate) mod missing_io_specifier;
pub(crate) mod nonportable_shortcircuit_inquiry;
pub(crate) mod select_default;
pub(crate) mod split_escaped_quote;
pub(crate) mod trailing_backslash;
pub(crate) mod unreachable_statement;
pub(crate) mod use_statements;

#[cfg(test)]
mod tests {
    use std::convert::AsRef;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use anyhow::Result;
    use assert_cmd::prelude::*;
    use insta::assert_snapshot;
    use tempfile::TempDir;
    use test_case::test_case;

    use crate::apply_common_filters;
    use crate::registry::Rule;
    use crate::rules::correctness::exit_labels;
    use crate::settings::{CheckSettings, FortranStandard};
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
    #[test_case(Rule::PointerInitialisationInDeclaration, Path::new("C082.f90"))]
    #[test_case(Rule::ExternalProcedure, Path::new("C091.f90"))]
    #[test_case(Rule::ProcedureNotInModule, Path::new("C092.f90"))]
    #[test_case(Rule::MissingDefaultPointerInitalisation, Path::new("C101.f90"))]
    #[test_case(Rule::UseAll, Path::new("C121.f90"))]
    #[test_case(Rule::MissingIntrinsic, Path::new("C122.f90"))]
    #[test_case(Rule::MissingAccessibilityStatement, Path::new("C131.f90"))]
    #[test_case(Rule::DefaultPublicAccessibility, Path::new("C132.f90"))]
    #[test_case(Rule::MissingExitOrCycleLabel, Path::new("C141.f90"))]
    #[test_case(Rule::ExitOrCycleInUnlabelledLoop, Path::new("C142.f90"))]
    #[test_case(Rule::MissingEndLabel, Path::new("C143.f90"))]
    #[test_case(Rule::MisleadingInlineIfSemicolon, Path::new("C151.f90"))]
    #[test_case(Rule::MisleadingInlineIfContinuation, Path::new("C152.f90"))]
    #[test_case(Rule::NonportableShortcircuitInquiry, Path::new("C161.f90"))]
    #[test_case(Rule::SplitEscapedQuote, Path::new("C171.f90"))]
    #[test_case(Rule::UncheckedStat, Path::new("C181.f90"))]
    #[test_case(Rule::MultipleAllocationsWithStat, Path::new("C182.f90"))]
    #[test_case(Rule::StatWithoutMessage, Path::new("C183.f90"))]
    #[test_case(Rule::UnreachableStatement, Path::new("C191.f90"))]
    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.as_ref(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("correctness").join(path).as_path(),
            &CheckSettings::for_rule(rule_code),
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
            &CheckSettings::for_rule(rule_code),
        )?;
        assert!(
            diagnostics.is_empty(),
            "Test source has no warnings, but some were raised:\n{diagnostics}"
        );
        Ok(())
    }

    #[test_case(
        Rule::ImplicitExternalProcedures,
        Path::new("C003.f90"),
        FortranStandard::F2008
    )]
    #[test_case(
        Rule::AssumedSizeCharacterIntent,
        Path::new("C072.f90"),
        FortranStandard::F95
    )]
    #[test_case(
        Rule::MissingDefaultPointerInitalisation,
        Path::new("C101.f90"),
        FortranStandard::F95
    )]
    #[test_case(Rule::MissingIntrinsic, Path::new("C122.f90"), FortranStandard::F95)]
    fn rules_pass_for_standards_up_to_and_including(
        rule_code: Rule,
        path: &Path,
        std: FortranStandard,
    ) -> Result<()> {
        let mut settings = CheckSettings::for_rule(rule_code);
        settings.target_std = std;
        let diagnostics = test_path(Path::new("correctness").join(path).as_path(), &settings)?;
        assert!(
            diagnostics.is_empty(),
            "Test source has no warnings, but some were raised:\n{diagnostics}"
        );
        Ok(())
    }

    #[test]
    fn c061_fortran95() -> Result<()> {
        // Should exclude the pointer without intent warning
        let path = Path::new("C061.f90");
        let snapshot = format!("missing-intent-f95_{}", path.to_string_lossy());
        let mut settings = CheckSettings::for_rule(Rule::MissingIntent);
        settings.target_std = FortranStandard::F95;
        let diagnostics = test_path(Path::new("correctness").join(path).as_path(), &settings)?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }

    #[test]
    fn warn_c142_on_nested_loops_only() -> Result<()> {
        let rule_code = Rule::ExitOrCycleInUnlabelledLoop;
        let path = Path::new("C142.f90");
        let snapshot = format!(
            "{}_{}_nested_loops_only",
            rule_code.as_ref(),
            path.to_string_lossy()
        );
        let settings = CheckSettings {
            exit_unlabelled_loops: exit_labels::settings::Settings {
                allow_unnested_loops: true,
            },
            ..CheckSettings::for_rule(rule_code)
        };
        let diagnostics = test_path(Path::new("correctness").join(path).as_path(), &settings)?;
        apply_common_filters!();
        assert_snapshot!(snapshot, diagnostics);
        Ok(())
    }

    #[test]
    fn c151_fix_multiple_inline_if() -> Result<()> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().join("C151.f90");
        let code = r#"
        program test
            implicit none
            integer :: i
            if (i == 1) print *, "foo"; if (i == 2) print *, "bar"; if (i == 3) print *, "baz"
        end program test
        "#;
        fs::write(&path, code)?;
        Command::cargo_bin("fortitude")?
            .arg("check")
            .arg("--isolated")
            .arg("--fix")
            .arg("--preview")
            .arg("--select=C151")
            .arg(path.as_os_str())
            .status()?;
        let fixed = String::from_utf8(fs::read(path.as_os_str())?)?;
        let snapshot = "c151_fix_multiple_inline_if";
        apply_common_filters!();
        assert_snapshot!(snapshot, fixed);
        Ok(())
    }
}
