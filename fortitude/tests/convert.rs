use assert_cmd::prelude::*;
use insta_cmd::assert_cmd_snapshot;
use similar_asserts::assert_eq;
use std::{fs, process::Command};
use tempfile::TempDir;

const BIN_NAME: &str = "fortitude";

macro_rules! apply_common_filters {
    {} => {
        let mut settings = insta::Settings::clone_current();
        // Macos Temp Folder
        settings.add_filter(r"/var/folders/\S+?/T/\S+", "[TEMP_FILE]");
        // Linux Temp Folder
        settings.add_filter(r"/tmp/\.tmp\S+", "[TEMP_FILE]");
        // Windows Temp folder
        settings.add_filter(r"\b[A-Z]:\\.*\\Local\\Temp\\\S+", "[TEMP_FILE]");
        // Convert windows paths to Unix Paths.
        settings.add_filter(r"\\\\?([\w\d.])", "/$1");
        let _bound = settings.bind_to_scope();
    }
}

#[test]
fn convert() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f");
    fs::write(
        &test_file,
        r#"
c Example fixed form program
      PROGRAM TEST
      PRINT*,'start
     +        stop'
* another comment
      END
"#,
    )?;

    apply_common_filters!();

    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("convert-fixed")
                         .arg(&test_file),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let expected = r#"
! Example fixed form program
      PROGRAM TEST
      PRINT*,'start &
     &        stop'
! another comment
      END
"#;

    let result = fs::read_to_string(test_file)?;

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn convert_directory() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f");
    fs::write(
        &test_file,
        r#"
c Example fixed form program
      PROGRAM TEST
      PRINT*,'start
     +        stop'
* another comment
      END
"#,
    )?;

    apply_common_filters!();

    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("convert-fixed")
                         .current_dir(&tempdir),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let expected = r#"
! Example fixed form program
      PROGRAM TEST
      PRINT*,'start &
     &        stop'
! another comment
      END
"#;

    let result = fs::read_to_string(test_file)?;

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn convert_syntax_error() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f");
    let original = r#"
c Example fixed form program
      PROGRAM TEST
      PRINT*,5 +
* another comment
      END
"#;
    fs::write(&test_file, original)?;

    apply_common_filters!();

    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("convert-fixed")
                         .arg(&test_file),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    warning: file '[TEMP_FILE] has syntax errors, skipping
    ");

    let result = fs::read_to_string(test_file)?;

    assert_eq!(result, original);

    Ok(())
}
