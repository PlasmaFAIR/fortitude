use assert_cmd::prelude::*;
use insta_cmd::assert_cmd_snapshot;
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
fn check_file_doesnt_exist() -> anyhow::Result<()> {
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("test/file/doesnt/exist.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test/file/doesnt/exist.f90:1:1: E000 Error opening file: No such file or directory (os error 2)

    fortitude: 1 files scanned.
    Number of errors: 1

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn unknown_name_in_config() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let config_file = tempdir.path().join("fpm.toml");
    fs::write(
        &config_file,
        r#"
[extra.fortitude.check]
unknown-key = 1
"#,
    )?;
    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .args(["--config-file", config_file.as_os_str().to_string_lossy().as_ref()])
                         .arg("check")
                         .arg("no-file.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: TOML parse error at line 2, column 1
      |
    2 | unknown-key = 1
      | ^^^^^^^^^^^
    unknown field `unknown-key`, expected one of `files`, `ignore`, `select`, `extend-select`, `line-length`, `file-extensions`, `output-format`
    ");
    Ok(())
}

#[test]
fn check_all() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
program test
  logical*4, parameter :: true = .true.
end program
"#,
    )?;

    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg(test_file),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] T001 program missing 'implicit none'
      |
    2 | program test
      | ^^^^^^^^^^^^ T001
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      |

    [TEMP_FILE] T021 logical*4 is non-standard, use logical(4)
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
      |          ^^ T021
    4 | end program
      |

    [TEMP_FILE] T011 logical kind set with number literal '4', use 'iso_fortran_env' parameter
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
      |           ^ T011
    4 | end program
      |

    [TEMP_FILE] S061 end statement should read 'end program test'
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      | ^^^^^^^^^^^ S061
      |


    fortitude: 1 files scanned.
    Number of errors: 4

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_select_cli() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
program test
  logical*4, parameter :: true = .true.
end program
"#,
    )?;

    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg(test_file)
                         .arg("--select=T001,style"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] T001 program missing 'implicit none'
      |
    2 | program test
      | ^^^^^^^^^^^^ T001
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      |

    [TEMP_FILE] S061 end statement should read 'end program test'
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      | ^^^^^^^^^^^ S061
      |


    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_select_file() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
program test
  logical*4, parameter :: true = .true.
end program
"#,
    )?;

    let config_file = tempdir.path().join("fortitude.toml");
    fs::write(
        &config_file,
        r#"
[check]
select = ["T001", "style"]
"#,
    )?;

    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .args(["--config-file", config_file.as_os_str().to_string_lossy().as_ref()])
                         .arg("check")
                         .arg(test_file),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] T001 program missing 'implicit none'
      |
    2 | program test
      | ^^^^^^^^^^^^ T001
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      |

    [TEMP_FILE] S061 end statement should read 'end program test'
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      | ^^^^^^^^^^^ S061
      |


    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_extend_select_file() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
program test
  logical*4, parameter :: true = .true.
end program
"#,
    )?;

    let config_file = tempdir.path().join("fortitude.toml");
    fs::write(
        &config_file,
        r#"
[check]
select = ["T001"]
"#,
    )?;

    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .args(["--config-file", config_file.as_os_str().to_string_lossy().as_ref()])
                         .arg("check")
                         .arg(test_file)
                         .arg("--extend-select")
                         .arg("style"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] T001 program missing 'implicit none'
      |
    2 | program test
      | ^^^^^^^^^^^^ T001
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      |

    [TEMP_FILE] S061 end statement should read 'end program test'
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      | ^^^^^^^^^^^ S061
      |


    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_select_file_fpm_toml() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
program test
  logical*4, parameter :: true = .true.
end program
"#,
    )?;

    let config_file = tempdir.path().join("fpm.toml");
    fs::write(
        &config_file,
        r#"
[extra.fortitude.check]
select = ["T001", "style"]
"#,
    )?;

    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .args(["--config-file", config_file.as_os_str().to_string_lossy().as_ref()])
                         .arg("check")
                         .arg(test_file),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] T001 program missing 'implicit none'
      |
    2 | program test
      | ^^^^^^^^^^^^ T001
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      |

    [TEMP_FILE] S061 end statement should read 'end program test'
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      | ^^^^^^^^^^^ S061
      |


    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}
