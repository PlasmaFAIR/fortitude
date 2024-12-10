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
    fortitude: 0 files scanned, 1 could not be read.
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
    unknown field `unknown-key`, expected one of `files`, `ignore`, `select`, `extend-select`, `line-length`, `file-extensions`, `fix`, `no-fix`, `unsafe-fixes`, `no-unsafe-fixes`, `show-fixes`, `no-show-fixes`, `fix-only`, `no-fix-only`, `output-format`, `preview`, `no-preview`, `progress-bar`
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

    [TEMP_FILE] T021 'logical*4' uses non-standard syntax
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
      |          ^^ T021
    4 | end program
      |
      = help: Replace with 'logical(4)'

    [TEMP_FILE] T011 logical kind set with number literal '4', use 'iso_fortran_env' parameter
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
      |           ^ T011
    4 | end program
      |

    [TEMP_FILE] S061 [*] end statement should be named.
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      | ^^^^^^^^^^^ S061
      |
      = help: Write as 'end program test'.

    fortitude: 1 files scanned.
    Number of errors: 4

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option (1 hidden fix can be enabled with the `--unsafe-fixes` option).

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

    [TEMP_FILE] S061 [*] end statement should be named.
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      | ^^^^^^^^^^^ S061
      |
      = help: Write as 'end program test'.

    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option.

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

    [TEMP_FILE] S061 [*] end statement should be named.
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      | ^^^^^^^^^^^ S061
      |
      = help: Write as 'end program test'.

    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option.

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

    [TEMP_FILE] S061 [*] end statement should be named.
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      | ^^^^^^^^^^^ S061
      |
      = help: Write as 'end program test'.

    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option.

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

    [TEMP_FILE] S061 [*] end statement should be named.
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
    4 | end program
      | ^^^^^^^^^^^ S061
      |
      = help: Write as 'end program test'.

    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option.

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn apply_fixes() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
program foo
  implicit none
  real i
  i = 4.0
contains
  subroutine bar
    implicit none
  end subroutine bar
end program foo
"#,
    )?;
    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--preview")
                         .arg("--fix")
                         .arg(&test_file),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] T004 'implicit none' missing 'external'
      |
    2 | program foo
    3 |   implicit none
      |   ^^^^^^^^^^^^^ T004
    4 |   real :: i
    5 |   i = 4.0
      |
      = help: Add `(external)` to 'implicit none'

    [TEMP_FILE] P021 real has implicit kind
      |
    2 | program foo
    3 |   implicit none
    4 |   real :: i
      |   ^^^^ P021
    5 |   i = 4.0
    6 | contains
      |

    fortitude: 1 files scanned.
    Number of errors: 4 (2 fixed, 2 remaining)

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    No fixes available (1 hidden fix can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");

    let expected = r#"
program foo
  implicit none
  real :: i
  i = 4.0
contains
  subroutine bar
  end subroutine bar
end program foo
"#
    .to_string();

    let transformed = fs::read_to_string(&test_file)?;
    assert_eq!(transformed, expected);

    Ok(())
}

#[test]
fn apply_unsafe_fixes() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
program foo
  implicit none
  real i
  i = 4.0
contains
  subroutine bar
    implicit none
  end subroutine bar
end program foo
"#,
    )?;
    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--preview")
                         .arg("--fix")
                         .arg("--unsafe-fixes")
                         .arg(&test_file),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] P021 real has implicit kind
      |
    2 | program foo
    3 |   implicit none (type, external)
    4 |   real :: i
      |   ^^^^ P021
    5 |   i = 4.0
    6 | contains
      |

    fortitude: 1 files scanned.
    Number of errors: 4 (3 fixed, 1 remaining)

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");

    let expected = r#"
program foo
  implicit none (type, external)
  real :: i
  i = 4.0
contains
  subroutine bar
  end subroutine bar
end program foo
"#
    .to_string();

    let transformed = fs::read_to_string(&test_file)?;
    assert_eq!(transformed, expected);

    Ok(())
}

#[test]
fn check_multibyte_utf8() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        // NOTE: There should be trailing whitespace in the snippet, do not remove!
        r#"
program test
  use some_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_long_module_name, only : integer_working_precision
  implicit none
  integer(integer_working_precision), parameter, dimension(1) :: a = [1]
  write (*, '("╔════════════════════════════════════════════╗")')
  write (*, '("║  UTF-8 LOGO BOX                            ║")')
  write (*, '("╚════════════════════════════════════════════╝")')
  !-- transform into g/cm³   
  dens = dens * ( 0.001d0 / (1.0d-30*bohr**3.0d0))
  !-- transform³ into³ g/cm³   
  dens = dens * ( 0.001d0 / (1.0d-30*bohr**3.0d0))
end program test
"#,
    )?;
    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=trailing-whitespace,line-too-long")
                         .arg("--line-length=60")
                         .arg(&test_file),
                         @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] S001 line length of 222, exceeds maximum 60
      |
    2 | ...
    3 | ...ly_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_long_module_name, only : integer_working_precision
      |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ S001
    4 | ...
    5 | ...n(1) :: a = [1]
      |

    [TEMP_FILE] S001 line length of 72, exceeds maximum 60
      |
    3 |   use some_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_really_rea...
    4 |   implicit none
    5 |   integer(integer_working_precision), parameter, dimension(1) :: a = [1]
      |                                                             ^^^^^^^^^^^^ S001
    6 |   write (*, '("╔════════════════════════════════════════════╗"...
    7 |   write (*, '("║  UTF-8 LOGO BOX                            ║")')
      |

    [TEMP_FILE] S001 line length of 65, exceeds maximum 60
      |
    4 |   implicit none
    5 |   integer(integer_working_precision), parameter, dimension(1) :: a = [1]
    6 |   write (*, '("╔════════════════════════════════════════════╗"...
      |                                                             ^^^^^ S001
    7 |   write (*, '("║  UTF-8 LOGO BOX                            ║")')
    8 |   write (*, '("╚════════════════════════════════════════════╝"...
      |

    [TEMP_FILE] S001 line length of 65, exceeds maximum 60
      |
    5 |   integer(integer_working_precision), parameter, dimension(1) :: a = [1]
    6 |   write (*, '("╔════════════════════════════════════════════╗"...
    7 |   write (*, '("║  UTF-8 LOGO BOX                            ║")')
      |                                                             ^^^^^ S001
    8 |   write (*, '("╚════════════════════════════════════════════╝"...
    9 |   !-- transform into g/cm³   
      |

    [TEMP_FILE] S001 line length of 65, exceeds maximum 60
       |
     6 |   write (*, '("╔════════════════════════════════════════════╗"...
     7 |   write (*, '("║  UTF-8 LOGO BOX                            ║")')
     8 |   write (*, '("╚════════════════════════════════════════════╝"...
       |                                                             ^^^^^ S001
     9 |   !-- transform into g/cm³   
    10 |   dens = dens * ( 0.001d0 / (1.0d-30*bohr**3.0d0))
       |

    [TEMP_FILE] S101 [*] trailing whitespace
       |
     7 |   write (*, '("║  UTF-8 LOGO BOX                            ║")')
     8 |   write (*, '("╚════════════════════════════════════════════╝"...
     9 |   !-- transform into g/cm³   
       |                           ^^^ S101
    10 |   dens = dens * ( 0.001d0 / (1.0d-30*bohr**3.0d0))
    11 |   !-- transform³ into³ g/cm³   
       |
       = help: Remove trailing whitespace

    [TEMP_FILE] S101 [*] trailing whitespace
       |
     9 |   !-- transform into g/cm³   
    10 |   dens = dens * ( 0.001d0 / (1.0d-30*bohr**3.0d0))
    11 |   !-- transform³ into³ g/cm³   
       |                             ^^^ S101
    12 |   dens = dens * ( 0.001d0 / (1.0d-30*bohr**3.0d0))
    13 | end program test
       |
       = help: Remove trailing whitespace

    fortitude: 1 files scanned.
    Number of errors: 7

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 2 fixable with the `--fix` option.

    ----- stderr -----
    "#);

    Ok(())
}
