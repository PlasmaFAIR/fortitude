use assert_cmd::prelude::*;
use insta_cmd::assert_cmd_snapshot;
use std::path::{Path, PathBuf};
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
        // Ignore specific os errors
        settings.add_filter(r"E000 Error opening file: .*", "E000 Error opening file: [OS_ERROR]");
        let _bound = settings.bind_to_scope();
    }
}

#[test]
fn check_file_doesnt_exist() -> anyhow::Result<()> {
    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("test/file/doesnt/exist.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test/file/doesnt/exist.f90:1:1: E000 Error opening file: [OS_ERROR]
    fortitude: 0 files scanned, 1 could not be read.
    Number of errors: 1

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn deprecated_category() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
program test
    implicit none
    integer :: i
    i = 1  ! Comment ending with backslash\
    select case (i)  ! Select without default
        case(1)
            print *, "one"
    end select
end program test
        "#,
    )?;
    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=bugprone")
                         .arg("--preview")
                         .arg(&test_file),
                         @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] C011 Trailing backslash
      |
    3 |     implicit none
    4 |     integer :: i
    5 |     i = 1  ! Comment ending with backslash\
      |                                           ^ C011
    6 |     select case (i)  ! Select without default
    7 |         case(1)
      |

    [TEMP_FILE] C001 Missing default case may not handle all values
       |
     4 |       integer :: i
     5 |       i = 1  ! Comment ending with backslash\
     6 | /     select case (i)  ! Select without default
     7 | |         case(1)
     8 | |             print *, "one"
     9 | |     end select
       | |______________^ C001
    10 |   end program test
       |
       = help: Add 'case default'

    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    warning: `bugprone` has been remapped to `C001`, `C011`.
    "#
    );
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
    fortitude failed
    Error: Failed to parse [TEMP_FILE]

    Caused by:
        TOML parse error at line 3, column 1
          |
        3 | unknown-key = 1
          | ^^^^^^^^^^^
        unknown field `unknown-key`, expected one of `files`, `fix`, `unsafe-fixes`, `show-fixes`, `fix-only`, `output-format`, `preview`, `progress-bar`, `ignore`, `select`, `extend-select`, `file-extensions`, `exclude`, `extend-exclude`, `force-exclude`, `respect-gitignore`, `line-length`, `per-file-ignores`
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
                         .arg("--select=S061,T001,T011,T021")
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

    [TEMP_FILE] T011 logical kind set with number literal '4'
      |
    2 | program test
    3 |   logical*4, parameter :: true = .true.
      |           ^ T011
    4 | end program
      |
      = help: Use the parameter 'int32' from 'iso_fortran_env'

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
                         .arg("--select=S071,P021,T003,T004")
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
                         .arg("--select=S071,P021,T003,T004")
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

#[test]
fn check_per_file_ignores() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let path = tempdir.path();
    let nested = path.join("nested");
    let double_nested = nested.join("double_nested");
    std::fs::create_dir(nested.as_path())?;
    std::fs::create_dir(double_nested.as_path())?;
    for file in ["foo", "bar", "baz"] {
        for (idx, dir) in [path, &nested, &double_nested].iter().enumerate() {
            let snippet = format!(
                r#"
module {file}{idx}
! missing implicit none
contains
  integer function f()
    f = 1
  end function f
end module {file}{idx}
"#
            );
            fs::write(dir.join(format!("{file}{idx}.f90")), snippet)?;
        }
    }

    let config_file = path.join(".fortitude.toml");
    let config = r#"
[check.per-file-ignores]
"bar*.f90" = ["implicit-typing"]
"#;
    fs::write(&config_file, config)?;
    apply_common_filters!();
    // Expect:
    // - Override per-file-ignores in the config file
    // - Files of foo, bar, and baz
    // - No files with index 2
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=typing")
                         .arg("--per-file-ignores=**/double_nested/*.f90:implicit-typing")
                         .current_dir(path),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    bar0.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module bar0
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    baz0.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module baz0
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    foo0.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module foo0
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    nested/bar1.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module bar1
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    nested/baz1.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module baz1
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    nested/foo1.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module foo1
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    fortitude: 9 files scanned.
    Number of errors: 6

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");

    Ok(())
}

#[test]
fn check_extend_per_file_ignores() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let path = tempdir.path();
    let nested = path.join("nested");
    let double_nested = nested.join("double_nested");
    std::fs::create_dir(nested.as_path())?;
    std::fs::create_dir(double_nested.as_path())?;
    for file in ["foo", "bar", "baz"] {
        for (idx, dir) in [path, &nested, &double_nested].iter().enumerate() {
            let snippet = format!(
                r#"
module {file}{idx}
! missing implicit none
contains
  integer function f()
    f = 1
  end function f
end module {file}{idx}
"#
            );
            fs::write(dir.join(format!("{file}{idx}.f90")), snippet)?;
        }
    }

    let config_file = path.join(".fortitude.toml");
    let config = r#"
[check.per-file-ignores]
"bar*.f90" = ["implicit-typing"]
"#;
    fs::write(&config_file, config)?;
    apply_common_filters!();
    // Expect:
    // - Don't overwrite config file
    // - File types of foo and baz but no bar
    // - No files with index 2
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=typing")
                         .arg("--extend-per-file-ignores=**/double_nested/*.f90:implicit-typing")
                         .current_dir(path),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    baz0.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module baz0
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    foo0.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module foo0
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    nested/baz1.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module baz1
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    nested/foo1.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module foo1
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    fortitude: 9 files scanned.
    Number of errors: 4

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");

    Ok(())
}

fn exclude_test_path<P: AsRef<Path>>(tempdir: P) -> PathBuf {
    let base_path = tempdir.as_ref().join("base");
    let foo_path = base_path.join("foo");
    let bar_path = foo_path.join("bar");
    // Simulate a Python env, which is in the default exclude list
    let venv_path = base_path.join(".venv/lib/site-packages/numpy");
    std::fs::create_dir_all(bar_path.as_path()).unwrap();
    std::fs::create_dir_all(venv_path.as_path()).unwrap();
    for dir in [&base_path, &foo_path, &bar_path, &venv_path] {
        let name = dir.file_name().unwrap().to_string_lossy();
        let snippet = format!(
            r#"
module {name}
! missing implicit none
contains
  integer function f()
    f = 1
  end function f
end module {name}
"#
        );
        fs::write(dir.join(format!("{name}.f90")), snippet).unwrap();
    }

    let config_file = base_path.join(".fortitude.toml");
    let config = r#"
[check]
exclude = [
    "foo.f90",
]
"#;
    fs::write(&config_file, config).unwrap();
    base_path
}

#[test]
fn check_exclude() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    apply_common_filters!();
    // Expect:
    // - Override 'foo.f90' in config file, see 'base.f90' and 'foo.f90' but not 'bar.f90'
    // - Don't see anything in venv
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=typing")
                         .arg("--exclude=bar")
                         .current_dir(exclude_test_path(tempdir.path())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    base.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module base
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    foo/foo.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module foo
      | ^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    fortitude: 2 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_extend_exclude() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    apply_common_filters!();
    // Expect:
    // - Don't overwrite 'foo.f90' in config file, see only base.f90
    // - Don't see anything in venv
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=typing")
                         .arg("--extend-exclude=bar")
                         .current_dir(exclude_test_path(tempdir.path())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    base.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module base
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    fortitude: 1 files scanned.
    Number of errors: 1

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");

    Ok(())
}

#[test]
fn check_no_force_exclude() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    apply_common_filters!();
    // Expect:
    // - See error in foo.f90 despite it being in the exclude list
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=typing")
                         .arg("foo/foo.f90")
                         .current_dir(exclude_test_path(tempdir.path())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    foo/foo.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module foo
      | ^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    fortitude: 1 files scanned.
    Number of errors: 1

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_force_exclude() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    apply_common_filters!();
    // Expect:
    // - Don't see error in foo.f90 despite it being asked for
    // - Also shouldn't see numpy.f90
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=typing")
                         .arg("--force-exclude")
                         .arg("foo/foo.f90")
                         .arg(".venv/lib/site-packages/numpy/numpy.f90")
                         .current_dir(exclude_test_path(tempdir.path())),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----
    fortitude: 0 files scanned.
    All checks passed!


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_exclude_builtin() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    apply_common_filters!();
    // Expect:
    // - See error in venv despite it being excluded by default
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=typing")
                         .arg(".venv/lib/site-packages/")
                         .current_dir(exclude_test_path(tempdir.path())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    .venv/lib/site-packages/numpy/numpy.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module numpy
      | ^^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    fortitude: 1 files scanned.
    Number of errors: 1

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_force_exclude_builtin() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    apply_common_filters!();
    // Expect:
    // - Don't see error in venv even though it was asked for
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=typing")
                         .arg("--force-exclude")
                         .arg(".venv/lib/site-packages/")
                         .current_dir(exclude_test_path(tempdir.path())),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----
    fortitude: 0 files scanned.
    All checks passed!


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_per_line_ignores() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
! allow(T001, unnamed-end-statement, literal-kind)
program test
  ! allow(star-kind)
  logical*4, parameter :: true = .true.
  ! allow(trailing-whitespace)
  logical*4, parameter :: false = .false.  
end program test
"#,
    )?;

    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg(test_file)
                         .args(["--select=T001,S061,T011,T021,S101"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] T021 'logical*4' uses non-standard syntax
      |
    5 |   logical*4, parameter :: true = .true.
    6 |   ! allow(trailing-whitespace)
    7 |   logical*4, parameter :: false = .false.  
      |          ^^ T021
    8 | end program test
      |
      = help: Replace with 'logical(4)'

    fortitude: 1 files scanned.
    Number of errors: 1

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    No fixes available (1 hidden fix can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");

    Ok(())
}

#[test]
fn check_invalid_per_line_ignores() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
! allow(badbad, notgood)
program test
end program test
"#,
    )?;

    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg(test_file)
                         .arg("--select=E011"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    [TEMP_FILE] E011 Unknown rule selector: `badbad`
      |
    2 | ! allow(badbad, notgood)
      |         ^^^^^^ E011
    3 | program test
    4 | end program test
      |

    [TEMP_FILE] E011 Unknown rule selector: `notgood`
      |
    2 | ! allow(badbad, notgood)
      |                 ^^^^^^^ E011
    3 | program test
    4 | end program test
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
fn apply_fixes_with_allow_comment() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(
        &test_file,
        r#"
! allow(superfluous-implicit-none)
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
                         .arg("--select=superfluous-implicit-none")
                         .arg("--fix")
                         .arg(&test_file),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----
    fortitude: 1 files scanned.
    All checks passed!


    ----- stderr -----
    ");

    let expected = r#"
! allow(superfluous-implicit-none)
program foo
  implicit none
  real i
  i = 4.0
contains
  subroutine bar
    implicit none
  end subroutine bar
end program foo
"#
    .to_string();

    let transformed = fs::read_to_string(&test_file)?;
    assert_eq!(transformed, expected);

    Ok(())
}

#[test]
fn check_toml_settings() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let config_file = tempdir.path().join("fortitude.toml");
    let fortran_file = tempdir.path().join("myfile.ff");
    fs::write(
        &config_file,
        r#"
[check]
file-extensions = ["ff"]
line-length = 10
"#,
    )?;
    fs::write(
        &fortran_file,
        r#"
program myprogram
end program myprogram
"#,
    )?;
    apply_common_filters!();
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=S001,F001,T001")
                         .current_dir(tempdir.path()),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    myfile.ff:1:1: F001 file extension should be '.f90' or '.F90'
    myfile.ff:2:1: T001 program missing 'implicit none'
      |
    2 | program myprogram
      | ^^^^^^^^^^^^^^^^^ T001
    3 | end program myprogram
      |

    myfile.ff:2:11: S001 line length of 17, exceeds maximum 10
      |
    2 | program myprogram
      |           ^^^^^^^ S001
    3 | end program myprogram
      |

    myfile.ff:3:11: S001 line length of 21, exceeds maximum 10
      |
    2 | program myprogram
    3 | end program myprogram
      |           ^^^^^^^^^^^ S001
      |

    fortitude: 1 files scanned.
    Number of errors: 4

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

fn gitignore_test_path<P: AsRef<Path>>(tempdir: P) -> PathBuf {
    let base_path = tempdir.as_ref().join("base");
    let include_dir = base_path.join("include");
    let exclude_dir_1 = base_path.join("exclude");
    let exclude_dir_2 = include_dir.join("exclude");
    std::fs::create_dir_all(exclude_dir_1.as_path()).unwrap();
    std::fs::create_dir_all(exclude_dir_2.as_path()).unwrap();
    for dir in [&base_path, &include_dir, &exclude_dir_1, &exclude_dir_2] {
        let name = dir.file_name().unwrap().to_string_lossy();
        let snippet = format!(
            r#"
module {name}
! missing implicit none
contains
  integer function f()
    f = 1
  end function f
end module {name}
"#
        );
        fs::write(dir.join("include.f90"), &snippet).unwrap();
        fs::write(dir.join("exclude.f90"), &snippet).unwrap();
    }

    // Simulate a git repo. Don't need anything inside the .git folder
    let git_path = base_path.join(".git");
    std::fs::create_dir_all(git_path.as_path()).unwrap();
    let gitignore_file = base_path.join(".gitignore");
    let config = r#"
exclude
exclude.f90
"#;
    fs::write(&gitignore_file, config).unwrap();
    base_path
}

#[test]
fn check_gitignore() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    apply_common_filters!();
    // Expect:
    // - See file include.f90 in the base path and include/include.f90
    // - Don't see file exclude.f90 in the base path or files in exclude directories
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=typing")
                         .current_dir(gitignore_test_path(tempdir.path())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    include.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module base
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    include/include.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module include
      | ^^^^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    fortitude: 2 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_no_respect_gitignore() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    apply_common_filters!();
    // Expect to see all 8 files, even though exclude.f90 and exclude/ are in the .gitignore
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .arg("--select=typing")
                         .arg("--no-respect-gitignore")
                         .current_dir(gitignore_test_path(tempdir.path())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    exclude.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module base
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    exclude/exclude.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module exclude
      | ^^^^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    exclude/include.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module exclude
      | ^^^^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    include.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module base
      | ^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    include/exclude.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module include
      | ^^^^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    include/exclude/exclude.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module exclude
      | ^^^^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    include/exclude/include.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module exclude
      | ^^^^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    include/include.f90:2:1: T001 module missing 'implicit none'
      |
    2 | module include
      | ^^^^^^^^^^^^^^ T001
    3 | ! missing implicit none
    4 | contains
      |

    fortitude: 8 files scanned.
    Number of errors: 8

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn preview_enabled_prefix() -> anyhow::Result<()> {
    // All the E99XX test rules should be triggered
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .args(["--select=E99", "--output-format=concise", "--preview"])
                         .arg("-"), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    -:1:1: E9900 Hey this is a stable test rule.
    -:1:1: E9901 [*] Hey this is a stable test rule with a safe fix.
    -:1:1: E9902 Hey this is a stable test rule with an unsafe fix.
    -:1:1: E9903 Hey this is a stable test rule with a display only fix.
    -:1:1: E9911 Hey this is a preview test rule.
    -:1:1: E9950 Hey this is a test rule that was redirected from another.
    fortitude: 1 files scanned.
    Number of errors: 6

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option (1 hidden fix can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn preview_disabled_direct() -> anyhow::Result<()> {
    // All the E99XX test rules should be triggered
    assert_cmd_snapshot!(Command::cargo_bin(BIN_NAME)?
                         .arg("check")
                         .args(["--select=E9911", "--output-format=concise"])
                         .arg("-"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    fortitude: 1 files scanned.
    All checks passed!


    ----- stderr -----
    warning: Selection `E9911` has no effect because preview is not enabled.
    ");
    Ok(())
}
