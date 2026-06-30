use anyhow::{Context, Result};
use insta::internals::SettingsBindDropGuard;
use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use std::path::{Path, PathBuf};
use std::{fs, process::Command};
use tempfile::TempDir;
use textwrap::dedent;

const BIN_NAME: &str = "fortitude";
const STDIN_BASE_OPTIONS: &[&str] = &["--output-format", "concise"];

fn fortitude_cmd() -> Command {
    Command::new(get_cargo_bin(BIN_NAME))
}

/// Creates a regex filter for replacing temporary directory paths in snapshots
pub(crate) fn tempdir_filter(path: impl AsRef<str>) -> String {
    format!(r"{}[\\/]?", regex::escape(path.as_ref()))
}

/// Builder for `fortitude check` commands
struct FortitudeCheck {
    _temp_dir: TempDir,
    _settings_scope: SettingsBindDropGuard,
    project_dir: PathBuf,
}

impl FortitudeCheck {
    /// Creates a new test fixture with an empty temporary directory.
    ///
    /// This sets up:
    /// - A temporary directory that's automatically cleaned up
    /// - Insta snapshot filters for cross-platform path compatibility
    /// - Environment isolation for consistent test behavior
    pub(crate) fn new() -> Result<Self> {
        Self::with_settings(|_, settings| settings)
    }

    /// Generate a [`Command`] for the `fortitude check` command with some
    /// default options.
    ///
    /// The command is set up with:
    /// - The correct fortitude binary path
    /// - Working directory set to the test directory
    /// - Clean environment variables for consistent behavior
    /// - The `check` subcommand
    ///
    /// You can chain additional arguments and options as needed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let output = fixture
    ///     .check_command()
    ///     .args(["--select", "E"])
    ///     .arg(".")
    ///     .output()?;
    /// ```
    fn check_command(&self) -> Command {
        let mut cmd = self.check_command_plain();
        cmd.args(STDIN_BASE_OPTIONS);
        cmd
    }

    /// Like [`check_command`], but without the other arguments
    fn check_command_plain(&self) -> Command {
        let mut cmd = fortitude_cmd();
        cmd.current_dir(&self.project_dir);

        // Unset all environment variables because they can affect test behavior.
        cmd.env_clear();

        // Use the `check` subcommand
        cmd.arg("check");
        cmd
    }

    pub(crate) fn with_settings(
        setup_settings: impl FnOnce(&Path, insta::Settings) -> insta::Settings,
    ) -> Result<Self> {
        let temp_dir = TempDir::new()?;

        // Canonicalize the tempdir path because macOS uses symlinks for tempdirs
        // and that doesn't play well with our snapshot filtering.
        // Simplify with dunce because otherwise we get UNC paths on Windows.
        let project_dir = dunce::simplified(
            &temp_dir
                .path()
                .canonicalize()
                .context("Failed to canonicalize project path")?,
        )
        .to_path_buf();

        let mut settings = setup_settings(&project_dir, insta::Settings::clone_current());

        settings.add_filter(&tempdir_filter(project_dir.to_str().unwrap()), "[TMP]/");
        settings.add_filter(
            &tempdir_filter(Self::crates_root().to_str().unwrap()),
            "CRATE_ROOT/",
        );
        settings.add_filter(r#"\\([\w&&[^nr"]]|\s|\.)"#, "/$1");
        settings.add_filter(r"(Panicked at) [^:]+:\d+:\d+", "$1 <location>");
        settings.add_filter(fortitude_linter::VERSION, "[VERSION]");
        settings.add_filter(
            r"E000 Error opening file: .*",
            "E000 Error opening file: [OS_ERROR]",
        );

        let settings_scope = settings.bind_to_scope();

        Ok(Self {
            project_dir,
            _temp_dir: temp_dir,
            _settings_scope: settings_scope,
        })
    }

    /// Returns the path to the test directory root.
    pub(crate) fn root(&self) -> &Path {
        &self.project_dir
    }

    /// Returns the path to the directory above the crate root.
    pub(crate) fn crates_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
    }

    /// Creates a test fixture with a single file.
    ///
    /// # Arguments
    ///
    /// * `path` - The relative path for the file
    /// * `content` - The content to write to the file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let fixture = FortitudeCheck::with_file("fortitude.toml", "select = ['E']")?;
    /// ```
    pub(crate) fn with_file(path: impl AsRef<Path>, content: &str) -> Result<Self> {
        let fixture = Self::new()?;
        fixture.write_file(path, content)?;
        Ok(fixture)
    }

    pub(crate) fn with_files<'a>(
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> anyhow::Result<Self> {
        let case = Self::new()?;
        case.write_files(files)?;
        Ok(case)
    }

    /// Ensures that the parent directory of a path exists.
    fn ensure_parent_directory(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory `{}`", parent.display()))?;
        }
        Ok(())
    }

    /// Writes a file to the test directory.
    ///
    /// Parent directories are created automatically if they don't exist.
    /// Content is dedented to remove common leading whitespace for cleaner test code.
    ///
    /// # Arguments
    ///
    /// * `path` - The relative path for the file
    /// * `content` - The content to write to the file
    pub(crate) fn write_file(&self, path: impl AsRef<Path>, content: &str) -> Result<()> {
        let path = path.as_ref();
        let file_path = self.project_dir.join(path);

        Self::ensure_parent_directory(&file_path)?;

        let content = dedent(content);
        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write file `{}`", file_path.display()))?;

        Ok(())
    }

    pub(crate) fn write_files<'a>(
        &self,
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<()> {
        for file in files {
            self.write_file(file.0, file.1)?;
        }
        Ok(())
    }
}

#[test]
fn check_file_doesnt_exist() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::new()?;
    assert_cmd_snapshot!(cmd
                         .check_command().arg("test/file/doesnt/exist.f90"),
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
    let cmd = FortitudeCheck::with_file(
        "test.f90",
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

    assert_cmd_snapshot!(cmd
                         .check_command()
                         .args(["--select=bugprone", "--preview"])
                         .arg("test.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:5:43: error[C051] Trailing backslash
    test.f90:6:5: error[C011] Missing default case may not handle all values
    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    warning: The selector `bugprone` refers to a deprecated rule category.
    warning: `B001` has been remapped to `C011`.
    warning: `B011` has been remapped to `C051`.
    "
    );
    Ok(())
}

#[test]
fn unknown_name_in_config() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_settings(|_, mut settings| {
        settings.add_filter(
            r"(unknown field `unknown-key`, expected one of).*",
            "$1 [OPTIONS]",
        );
        settings
    })?;

    cmd.write_file(
        "fpm.toml",
        r#"
[extra.fortitude.check]
unknown-key = 1
"#,
    )?;

    assert_cmd_snapshot!(cmd
                         .check_command()
                         .args(["--config", "fpm.toml"])
                         .arg("no_file.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    fortitude failed
    Error: Failed to load configuration `[TMP]/fpm.toml`

    Caused by:
        0: Failed to parse [TMP]/fpm.toml
        1: TOML parse error at line 3, column 1
             |
           3 | unknown-key = 1
             | ^^^^^^^^^^^
           unknown field `unknown-key`, expected one of [OPTIONS]
    ");
    Ok(())
}

#[test]
fn check_select_cli() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program test
  logical*4, parameter :: true = .true.
end program
"#,
    )?;

    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("test.f90")
                         .args(["--select=C001,style"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:2:1: C001 program uses implicit typing
    test.f90:4:1: S061 [*] end statement should be named.
    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option (1 hidden fix can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_select_file() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_files([
        (
            "test.f90",
            r#"
program test
  logical*4, parameter :: true = .true.
end program
"#,
        ),
        (
            "fortitude.toml",
            r#"
[check]
select = ["C001", "style"]
"#,
        ),
    ])?;

    assert_cmd_snapshot!(cmd
                         .check_command()
                         .args(["--config", "fortitude.toml"])
                         .arg("test.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:2:1: C001 program uses implicit typing
    test.f90:4:1: S061 [*] end statement should be named.
    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option (1 hidden fix can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_extend_select_file() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_files([
        (
            "test.f90",
            r#"
program test
  logical*4, parameter :: true = .true.
end program
"#,
        ),
        (
            "fortitude.toml",
            r#"
[check]
select = ["C001"]
"#,
        ),
    ])?;

    assert_cmd_snapshot!(cmd
                         .check_command()
                         .args(["--config", "fortitude.toml"])
                         .arg("test.f90")
                         .args(["--extend-select", "style"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:2:1: C001 program uses implicit typing
    test.f90:4:1: S061 [*] end statement should be named.
    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option (1 hidden fix can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_select_file_fpm_toml() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_files([
        (
            "test.f90",
            r#"
program test
  logical*4, parameter :: true = .true.
end program
"#,
        ),
        (
            "fpm.toml",
            r#"
[extra.fortitude.check]
select = ["C001", "style"]
"#,
        ),
    ])?;

    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("test.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:2:1: C001 program uses implicit typing
    test.f90:4:1: S061 [*] end statement should be named.
    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option (1 hidden fix can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_select_file_pyproject_toml() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_files([
        (
            "test.f90",
            r#"
program test
  logical*4, parameter :: true = .true.
end program
"#,
        ),
        (
            "pyproject.toml",
            r#"
[tool.fortitude.check]
select = ["C001", "style"]
"#,
        ),
    ])?;

    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("test.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:2:1: C001 program uses implicit typing
    test.f90:4:1: S061 [*] end statement should be named.
    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option (1 hidden fix can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn apply_fixes() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
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

    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("test.f90")
                         .args(["--select=S071,C022,S201,C003", "--preview", "--fix"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:3:3: error[C003] 'implicit none' missing 'external'
    test.f90:4:3: error[C022] real has implicit kind
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

    let transformed = fs::read_to_string(cmd.root().join("test.f90"))?;
    assert_eq!(transformed, expected);

    Ok(())
}

#[test]
fn apply_unsafe_fixes() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
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
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("test.f90")
                         .args(["--select=S071,C022,S201,C003", "--preview", "--fix", "--unsafe-fixes"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:4:3: error[C022] real has implicit kind
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

    let transformed = fs::read_to_string(cmd.root().join("test.f90"))?;
    assert_eq!(transformed, expected);

    Ok(())
}

#[test]
fn apply_all_fixes() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program foo
  implicit none
endprogram
"#,
    )?;
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("--select=S061,C003")
                         .arg("--unsafe-fixes")
                         .arg("--fix")
                         .arg("test.f90"),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----
    fortitude: 1 files scanned.
    Number of errors: 2 (2 fixed, 0 remaining)

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");

    let expected = r#"
program foo
  implicit none (type, external)
end program foo
"#
    .to_string();

    let transformed = fs::read_to_string(cmd.root().join("test.f90"))?;
    assert_eq!(transformed, expected);

    Ok(())
}

#[test]
fn apply_fixable_fixes() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program foo
  implicit none
endprogram
"#,
    )?;
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("--select=S061,C003")
                         .arg("--unsafe-fixes")
                         .arg("--fix")
                         .arg("--fixable=S061")
                         .arg("test.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:3:3: C003 'implicit none' missing 'external'
    fortitude: 1 files scanned.
    Number of errors: 2 (1 fixed, 1 remaining)

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");

    let expected = r#"
program foo
  implicit none
end program foo
"#
    .to_string();

    let transformed = fs::read_to_string(cmd.root().join("test.f90"))?;
    assert_eq!(transformed, expected);

    Ok(())
}

#[test]
fn skip_unfixable_fixes() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program foo
  implicit none
endprogram
"#,
    )?;
    assert_cmd_snapshot!(cmd.check_command()
                         .arg("--select=S061,C003")
                         .arg("--unsafe-fixes")
                         .arg("--fix")
                         .arg("--unfixable=C003")
                         .arg("test.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:3:3: C003 'implicit none' missing 'external'
    fortitude: 1 files scanned.
    Number of errors: 2 (1 fixed, 1 remaining)

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");

    let expected = r#"
program foo
  implicit none
end program foo
"#
    .to_string();

    let transformed = fs::read_to_string(cmd.root().join("test.f90"))?;
    assert_eq!(transformed, expected);

    Ok(())
}

#[test]
fn check_extend_fixable() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_files([
        (
            "test.f90",
            r#"
program test
  implicit none
end program
"#,
        ),
        (
            "fortitude.toml",
            r#"
[check]
fixable = ["C003"]
"#,
        ),
    ])?;

    assert_cmd_snapshot!(cmd.check_command()
                         .arg("--select=S061,C003")
                         .arg("--unsafe-fixes")
                         .arg("--fix")
                         .arg("--extend-fixable=S061")
                         .args(["--config", "fortitude.toml"])
                         .arg("test.f90"),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----
    fortitude: 1 files scanned.
    Number of errors: 2 (2 fixed, 0 remaining)

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_overwrite_fixable() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_files([
        (
            "test.f90",
            r#"
program test
  implicit none
end program
"#,
        ),
        (
            "fortitude.toml",
            r#"
[check]
fixable = ["C003"]
"#,
        ),
    ])?;

    assert_cmd_snapshot!(cmd.check_command()
                         .arg("--select=S061,C003")
                         .arg("--unsafe-fixes")
                         .arg("--fix")
                         .arg("--fixable=S061")
                         .args(["--config", "fortitude.toml"])
                         .arg("test.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:3:3: C003 'implicit none' missing 'external'
    fortitude: 1 files scanned.
    Number of errors: 2 (1 fixed, 1 remaining)

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_overwrite_unfixable() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_files([
        (
            "test.f90",
            r#"
program test
  implicit none
end program
"#,
        ),
        (
            "fortitude.toml",
            r#"
[check]
fixable = ["S061"]
"#,
        ),
    ])?;

    assert_cmd_snapshot!(cmd.check_command()
                         .arg("--select=S061,C003")
                         .arg("--unsafe-fixes")
                         .arg("--fix")
                         .arg("--unfixable=S061")
                         .arg("--extend-fixable=C003")
                         .args(["--config", "fortitude.toml"])
                         .arg("test.f90"),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:4:1: S061 end statement should be named.
    fortitude: 1 files scanned.
    Number of errors: 2 (1 fixed, 1 remaining)

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

/// When checking a file with syntax errors, any AST violations after the syntax
/// error are discarded.  This is to prevent the linter from raising false
/// positives due to an inaccurate AST. In this case, the syntax error should
/// cause the linter to ignore the second superfluous semi-colon violation, but
/// not the subsequent line length violation.
#[test]
fn check_syntax_errors() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program foo
  implicit none (type, external)
  integer :: i
  integer :: j
  i = 2;
  j = i ^ 2  ! This is a syntax error
  print *, j;
  print *, i + i + i + i + i + i + i + i + i + i + i
end program foo
"#,
    )?;
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("test.f90")
                         .args(["--select=syntax-error,superfluous-semicolon,line-too-long", "--line-length=50", "--preview"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:6:8: error[S081] unnecessary semicolon
    test.f90:7:9: error[E001] Syntax error
    test.f90:9:51: error[S001] line length of 52, exceeds maximum 50
    fortitude: 1 files scanned.
    Number of errors: 3

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    warning: Syntax errors detected in file: [TMP]/test.f90. Discarding subsequent violations from the AST and all fixes.
    ",);
    Ok(())
}

/// The above behaviour can be overridden by ignoring syntax errors.
#[test]
fn check_ignore_syntax_errors() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program foo
  implicit none (type, external)
  integer :: i
  integer :: j
  i = 2;
  j = i ^ 2  ! This is a syntax error
  print *, j;
end program foo
"#,
    )?;
    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args(["--select=superfluous-semicolon", "--preview"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:6:8: error[S081] unnecessary semicolon
    test.f90:8:13: error[S081] unnecessary semicolon
    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    warning: Syntax errors detected in file: [TMP]/test.f90. Discarding all fixes. Some violations from the AST may be unreliable.
    ",);
    Ok(())
}

/// Syntax errors can also be ignored with allow comments
#[test]
fn check_allow_syntax_errors() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program foo
  implicit none (type, external)
  integer :: i
  integer :: j
  i = 2;
  ! allow(syntax-error)
  j = i ^ 2  ! This is a syntax error
  print *, j;
end program foo
"#,
    )?;
    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args(["--select=syntax-error,superfluous-semicolon", "--preview"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:6:8: error[S081] unnecessary semicolon
    test.f90:9:13: error[S081] unnecessary semicolon
    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    warning: Syntax errors detected in file: [TMP]/test.f90. Discarding subsequent violations from the AST and all fixes.
    ",);
    Ok(())
}

/// Files with syntax errors should never be fixed under any circumstances.
#[test]
fn check_fix_with_syntax_errors() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program foo
  implicit none (type, external)
  integer :: i
  integer :: j
  i = 2
  j = i ^ 2  ! This is a syntax error
  print *, j;  ! superfluous-semicolon
end program foo
"#,
    )?;
    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args(["--select=superfluous-semicolon", "--preview", "--fix"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:8:13: error[S081] unnecessary semicolon
    fortitude: 1 files scanned.
    Number of errors: 1

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    warning: Syntax errors detected in file: [TMP]/test.f90. Discarding all fixes. Some violations from the AST may be unreliable.
    warning: Syntax errors detected in file: [TMP]/test.f90. No fixes will be applied.
    ",);
    Ok(())
}

#[test]
fn check_multibyte_utf8() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
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
    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args(["--select=trailing-whitespace,line-too-long", "--line-length=60"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:3:61: S001 line length of 222, exceeds maximum 60
    test.f90:5:61: S001 line length of 72, exceeds maximum 60
    test.f90:6:61: S001 line length of 65, exceeds maximum 60
    test.f90:7:61: S001 line length of 65, exceeds maximum 60
    test.f90:8:61: S001 line length of 65, exceeds maximum 60
    test.f90:9:27: S101 [*] trailing whitespace
    test.f90:11:29: S101 [*] trailing whitespace
    fortitude: 1 files scanned.
    Number of errors: 7

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 2 fixable with the `--fix` option.

    ----- stderr -----
    ");

    Ok(())
}

#[test]
fn check_per_file_ignores() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        ".fortitude.toml",
        r#"
[check.per-file-ignores]
"bar*.f90" = ["implicit-typing"]
"#,
    )?;

    let path = cmd.root();
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

    // Expect:
    // - Override per-file-ignores in the config file
    // - Files of foo, bar, and baz
    // - No files with index 2
    assert_cmd_snapshot!(cmd.check_command()
                         .arg("--select=implicit-typing")
                         .arg("--per-file-ignores=**/double_nested/*.f90:implicit-typing")
                         .current_dir(path),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    bar0.f90:2:1: C001 module uses implicit typing
    baz0.f90:2:1: C001 module uses implicit typing
    foo0.f90:2:1: C001 module uses implicit typing
    nested/bar1.f90:2:1: C001 module uses implicit typing
    nested/baz1.f90:2:1: C001 module uses implicit typing
    nested/foo1.f90:2:1: C001 module uses implicit typing
    fortitude: 9 files scanned.
    Number of errors: 6

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    No fixes available (6 hidden fixes can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");

    Ok(())
}

#[test]
fn check_extend_per_file_ignores() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        ".fortitude.toml",
        r#"
[check.per-file-ignores]
"bar*.f90" = ["implicit-typing"]
"#,
    )?;

    let path = cmd.root();
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

    // Expect:
    // - Don't overwrite config file
    // - File types of foo and baz but no bar
    // - No files with index 2
    assert_cmd_snapshot!(cmd.check_command()
                         .arg("--select=implicit-typing")
                         .arg("--extend-per-file-ignores=**/double_nested/*.f90:implicit-typing")
                         .current_dir(path),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    baz0.f90:2:1: C001 module uses implicit typing
    foo0.f90:2:1: C001 module uses implicit typing
    nested/baz1.f90:2:1: C001 module uses implicit typing
    nested/foo1.f90:2:1: C001 module uses implicit typing
    fortitude: 9 files scanned.
    Number of errors: 4

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    No fixes available (4 hidden fixes can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");

    Ok(())
}

fn exclude_test_path<P: AsRef<Path>>(tempdir: P) -> PathBuf {
    let base_path = tempdir.as_ref().join("base");
    let foo_path = base_path.join("foo");
    let bar_path = foo_path.join("bar");
    // Simulate a Python env, which is in the default exclude list
    let venv_path = base_path.join(".venv/lib/site-packages/scipy");
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
extend-exclude = [
    "foo.f90",
]
"#;
    fs::write(&config_file, config).unwrap();
    base_path
}

#[test]
fn check_exclude() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::new()?;
    // Expect:
    // - Override 'foo.f90' in config file, see 'base.f90' and 'foo.f90' but not 'bar.f90'
    // - Override builtins, including .venv
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("--isolated")
                         .arg("--select=implicit-typing")
                         .arg("--exclude=bar")
                         .arg(".")
                         .current_dir(exclude_test_path(cmd.root())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    .venv/lib/site-packages/scipy/scipy.f90:2:1: C001 module uses implicit typing
    base.f90:2:1: C001 module uses implicit typing
    foo/foo.f90:2:1: C001 module uses implicit typing
    fortitude: 3 files scanned.
    Number of errors: 3

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    No fixes available (3 hidden fixes can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_extend_exclude() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::new()?;
    // Expect:
    // - Don't overwrite 'foo.f90' in config file, see only base.f90
    // - Don't see anything in venv
    assert_cmd_snapshot!(cmd.check_command()
                         .arg("--select=implicit-typing")
                         .arg("--extend-exclude=bar")
                         .current_dir(exclude_test_path(cmd.root())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    base.f90:2:1: C001 module uses implicit typing
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
fn check_no_force_exclude() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::new()?;
    // Expect:
    // - See error in foo.f90 despite it being in the exclude list
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("foo/foo.f90")
                         .args(["--select=implicit-typing"])
                         .current_dir(exclude_test_path(cmd.root())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    foo/foo.f90:2:1: C001 module uses implicit typing
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
fn check_force_exclude() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::new()?;
    // Expect:
    // - Don't see error in foo.f90 despite it being asked for
    // - Also shouldn't see scipy.f90
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .args(["foo/foo.f90", ".venv/lib/site-packages/scipy/scipy.f90"])
                         .args(["--select=implicit-typing", "--force-exclude"])
                         .current_dir(exclude_test_path(cmd.root())),
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
    let cmd = FortitudeCheck::new()?;
    // Expect:
    // - See error in venv despite it being excluded by default
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg(".venv/lib/site-packages/")
                         .args(["--select=implicit-typing"])
                         .current_dir(exclude_test_path(cmd.root())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    .venv/lib/site-packages/scipy/scipy.f90:2:1: C001 module uses implicit typing
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
fn check_force_exclude_builtin() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::new()?;
    // Expect:
    // - Don't see error in venv even though it was asked for
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg(".venv/lib/site-packages/")
                         .args(["--select=implicit-typing", "--force-exclude"])
                         .current_dir(exclude_test_path(cmd.root())),
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
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
! allow(C001, unnamed-end-statement, literal-kind)
program test
  ! allow(star-kind)
  logical*4, parameter :: true = .true.
  ! allow(trailing-whitespace)
  logical*4, parameter :: false = .false.  
end program
"#,
    )?;

    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args(["--select=C001,S061,PORT011,PORT021,S101"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:7:10: PORT021 'logical*4' uses non-standard syntax
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
fn ignore_per_line_ignores() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
! allow(C001, unnamed-end-statement, literal-kind)
program test
  ! allow(star-kind)
  logical*4, parameter :: true = .true.
  ! allow(trailing-whitespace)
  logical*4, parameter :: false = .false.  
end program
"#,
    )?;

    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args(["--select=C001,S061,PORT011,PORT021,S101", "--ignore-allow-comments"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:3:1: C001 program uses implicit typing
    test.f90:5:10: PORT021 'logical*4' uses non-standard syntax
    test.f90:5:11: PORT011 logical kind set with number literal '4'
    test.f90:7:10: PORT021 'logical*4' uses non-standard syntax
    test.f90:7:11: PORT011 logical kind set with number literal '4'
    test.f90:7:42: S101 [*] trailing whitespace
    test.f90:8:1: S061 [*] end statement should be named.
    fortitude: 1 files scanned.
    Number of errors: 7

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 2 fixable with the `--fix` option (3 hidden fixes can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");

    Ok(())
}

#[test]
fn apply_fixes_with_allow_comment() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
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
    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args(["--select=superfluous-implicit-none", "--fix"]),
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

    let transformed = fs::read_to_string(cmd.root().join("test.f90"))?;
    assert_eq!(transformed, expected);

    Ok(())
}

#[test]
fn check_toml_settings() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_files([
        (
            "fortitude.toml",
            r#"
[check]
file-extensions = ["ff"]
line-length = 10
"#,
        ),
        (
            "myfile.ff",
            r#"
program myprogram
end program myprogram
"#,
        ),
    ])?;
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .args(["--config", "fortitude.toml"])
                         .arg("myfile.ff")
                         .args(["--select=S001,S091,C001"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    myfile.ff:1:1: S091 file extension should be '.f90' or '.F90'
    myfile.ff:2:1: C001 program uses implicit typing
    myfile.ff:2:11: S001 line length of 17, exceeds maximum 10
    myfile.ff:3:11: S001 line length of 21, exceeds maximum 10
    fortitude: 1 files scanned.
    Number of errors: 4

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    No fixes available (1 hidden fix can be enabled with the `--unsafe-fixes` option).

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
    let cmd = FortitudeCheck::new()?;
    // Expect:
    // - See file include.f90 in the base path and include/include.f90
    // - Don't see file exclude.f90 in the base path or files in exclude directories
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg(".")
                         .args(["--select=implicit-typing"])
                         .current_dir(gitignore_test_path(cmd.root())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    include.f90:2:1: C001 module uses implicit typing
    include/include.f90:2:1: C001 module uses implicit typing
    fortitude: 2 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    No fixes available (2 hidden fixes can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn check_no_respect_gitignore() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::new()?;
    // Expect to see all 8 files, even though exclude.f90 and exclude/ are in the .gitignore
    assert_cmd_snapshot!(cmd
                         .check_command()
                         .arg("--isolated")
                         .arg("--select=implicit-typing")
                         .arg("--no-respect-gitignore")
                         .current_dir(gitignore_test_path(cmd.root())),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    exclude.f90:2:1: C001 module uses implicit typing
    exclude/exclude.f90:2:1: C001 module uses implicit typing
    exclude/include.f90:2:1: C001 module uses implicit typing
    include.f90:2:1: C001 module uses implicit typing
    include/exclude.f90:2:1: C001 module uses implicit typing
    include/exclude/exclude.f90:2:1: C001 module uses implicit typing
    include/exclude/include.f90:2:1: C001 module uses implicit typing
    include/include.f90:2:1: C001 module uses implicit typing
    fortitude: 8 files scanned.
    Number of errors: 8

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    No fixes available (8 hidden fixes can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn preview_enabled_prefix() -> anyhow::Result<()> {
    // All the FORT99XX test rules should be triggered
    assert_cmd_snapshot!(FortitudeCheck::new()?
                         .check_command()
                         .args(["--select=FORT99", "--preview"])
                         .arg("-"), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    -:1:1: error[FORT9900] Hey this is a stable test rule.
    -:1:1: error[FORT9901] [*] Hey this is a stable test rule with a safe fix.
    -:1:1: error[FORT9902] Hey this is a stable test rule with an unsafe fix.
    -:1:1: error[FORT9903] Hey this is a stable test rule with a display only fix.
    -:1:1: error[FORT9911] Hey this is a preview test rule.
    -:1:1: error[FORT9950] Hey this is a test rule that was redirected from another.
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
    // Only selected rule is in preview, so should get a warning
    assert_cmd_snapshot!(FortitudeCheck::new()?
                         .check_command()
                         .arg("--select=FORT9911")
                         .arg("-"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    fortitude: 1 files scanned.
    All checks passed!


    ----- stderr -----
    warning: Selection `FORT9911` has no effect because preview is not enabled.
    ");
    Ok(())
}

#[test]
fn show_statistics() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program test
  logical*4, parameter :: true = .true.
  logical*4, parameter :: false = .false.  
end program test
"#,
    )?;

    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args(["--select=C001,S061,PORT011,PORT021,S101", "--statistics"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    2	PORT011	[ ] literal-kind
    2	PORT021	[ ] star-kind
    1	C001   	[ ] implicit-typing
    1	S101   	[*] trailing-whitespace
    fortitude: 1 files scanned.
    Number of errors: 6

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 1 fixable with the `--fix` option (3 hidden fixes can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");

    Ok(())
}

#[test]
fn show_statistics_unsafe_fixes() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program test
  logical*4, parameter :: true = .true.
  logical*4, parameter :: false = .false.  
end program test
"#,
    )?;

    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args(["--select=C001,S061,PORT011,PORT021,S101", "--statistics", "--unsafe-fixes"]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    2	PORT011	[ ] literal-kind
    2	PORT021	[*] star-kind
    1	C001   	[*] implicit-typing
    1	S101   	[*] trailing-whitespace
    fortitude: 1 files scanned.
    Number of errors: 6

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    [*] 4 fixable with the `--fix` option.

    ----- stderr -----
    ");

    Ok(())
}

#[test]
fn show_statistics_json() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program test
  logical*4, parameter :: true = .true.
  logical*4, parameter :: false = .false.  
end program test
"#,
    )?;

    assert_cmd_snapshot!(cmd
                         .check_command_plain()
                         .arg("test.f90")
                         .args(["--select=C001,S061,PORT011,PORT021,S101", "--statistics", "--output-format=json"]),
                         @r#"
    success: false
    exit_code: 1
    ----- stdout -----
    [
      {
        "code": "PORT011",
        "name": "literal-kind",
        "count": 2,
        "fixable": false,
        "fixable_count": 0
      },
      {
        "code": "PORT021",
        "name": "star-kind",
        "count": 2,
        "fixable": false,
        "fixable_count": 0
      },
      {
        "code": "C001",
        "name": "implicit-typing",
        "count": 1,
        "fixable": false,
        "fixable_count": 0
      },
      {
        "code": "S101",
        "name": "trailing-whitespace",
        "count": 1,
        "fixable": true,
        "fixable_count": 1
      }
    ]

    ----- stderr -----
    "#);

    Ok(())
}

/// Check that fixing in stdin mode outputs the fixed file and nothing else.
#[test]
fn stdin_fix_mode() -> anyhow::Result<()> {
    let input_file = r#"
program test
  implicit none
end program test
"#;
    assert_cmd_snapshot!(FortitudeCheck::new()?.check_command()
                         .args(["--select=C003", "--fix-only", "--unsafe-fixes", "--quiet", "--stdin-filename=test.f90"])
                         .pass_stdin(input_file),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----

    program test
      implicit none (type, external)
    end program test

    ----- stderr -----
    ");
    Ok(())
}

/// Check that fixing in stdin mode outputs the unmodified input file if there
/// are no fixes to be made.
#[test]
fn stdin_fix_mode_no_fix() -> anyhow::Result<()> {
    let input_file = r#"
program test
  implicit none (type, external)
end program test
"#;
    assert_cmd_snapshot!(FortitudeCheck::new()?
                         .check_command()
                         .args([
                             "--select=C003",
                             "--fix-only",
                             "--unsafe-fixes",
                             "--quiet",
                             "--stdin-filename=test.f90"
                         ])
                         .pass_stdin(input_file),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----

    program test
      implicit none (type, external)
    end program test

    ----- stderr -----
    ");

    Ok(())
}

/// Issue 429, ignoring syntax errors from missing nodes
#[test]
fn ignore_syntax_errors() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program foo
  implicit none (type, external)
  type(bar, pointer :: zing
end program foo
"#,
    )?;

    assert_cmd_snapshot!(cmd.check_command().arg("test.f90")
                         .args(["--select=C003", "--ignore=E001"]),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----
    fortitude: 1 files scanned.
    All checks passed!


    ----- stderr -----
    warning: Syntax errors detected in file: [TMP]/test.f90. Discarding all fixes. Some violations from the AST may be unreliable.
    ");

    Ok(())
}

#[test]
fn isolated_mode() -> anyhow::Result<()> {
    let tempdir = TempDir::new()?;
    let test_file = tempdir.path().join("test.f90");
    fs::write(&test_file, "")?;
    let config_file = tempdir.path().join("fpm.toml");
    fs::write(
        &config_file,
        r#"
[extra.fortitude.check]
ignore = ["missing-intent"]
"#,
    )?;

    let cmd = fortitude_cmd()
        .args(["--isolated", "check", "--show-settings"])
        .current_dir(&tempdir)
        .output()?;

    let result = format!("{:?}", cmd);
    assert!(
        result.contains("missing-intent"),
        "'missing-intent' in {result}"
    );

    let cmd = fortitude_cmd()
        .args(["check", "--show-settings"])
        .current_dir(&tempdir)
        .output()?;

    let result = format!("{:?}", cmd);
    assert!(
        !result.contains("missing-intent"),
        "no 'missing-intent' in {result}"
    );

    Ok(())
}

#[test]
fn diff_mode() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program foo
  implicit none
  integer :: zing
end program foo
"#,
    )?;

    assert_cmd_snapshot!(cmd.check_command().arg("test.f90")
                         .args(["--select=C", "--diff", "--unsafe-fixes"]),
                         @r"
    success: true
    exit_code: 0
    ----- stdout -----
    --- test.f90
    +++ test.f90
    @@ -1,5 +1,5 @@
     
     program foo
    -  implicit none
    +  implicit none (type, external)
       integer :: zing
     end program foo

    Would fix 1 error.

    ----- stderr -----
    ");

    Ok(())
}

#[test]
fn nonblock_do_rules() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::new()?;
    let test_file = FortitudeCheck::crates_root()
        .join("fortitude_linter/resources/test/fixtures/obsolescent/labelled_do.f90");

    assert_cmd_snapshot!(
        cmd.check_command()
            .args(["--select=OB09", "--diff", "--unsafe-fixes", "--preview"])
            .arg(test_file)
    );

    Ok(())
}

#[test]
fn line_filter() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::with_file(
        "test.f90",
        r#"
program test
  logical*4, parameter :: true = .true.
  logical*4, parameter :: false = .false.
  ! more lines
  !
  !
  logical*4 :: maybe
end program test
"#,
    )?;

    // Note: path must be formatted with Debug to ensure backslashes on Windows
    // are properly escaped
    let filter_arg = format!(
        "--line-filter=[{{\"name\":{:?}, \"lines\":[[1, 3], [8, 8]]}}]",
        std::fs::canonicalize(cmd.root().join("test.f90"))?
    );

    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args([
                             &filter_arg,
                             "--select=PORT011",
                         ]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:3:11: PORT011 logical kind set with number literal '4'
    test.f90:8:11: PORT011 logical kind set with number literal '4'
    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");

    Ok(())
}

/// Helper for a tests in a git repo with commit changes
fn set_up_git_repo<P: AsRef<Path>>(
    tempdir: P,
) -> anyhow::Result<(git2::Repository, PathBuf, git2::Oid)> {
    let filename = Path::new("test.f90");
    let test_file = fortitude_linter::fs::fully_normalize_path_to(
        tempdir.as_ref().join(filename),
        fortitude_linter::fs::normalize_path(&tempdir),
    );
    fs::write(
        &test_file,
        r#"
program test
  logical*4, parameter :: true = .true.
  ! space out the diff hunks



  logical*4, parameter :: false = .false.



end program test
"#,
    )?;

    // This is a bit of a complicated test, because we need to make a new repo,
    // make a commit, and then stage a change
    // git init
    let repo = git2::Repository::init(&tempdir)?;
    // git add
    let mut index = repo.index()?;
    index.add_path(filename)?;
    index.write()?;
    // git commit
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let sig = git2::Signature::now("fortitude_test", "fortitude@example.com")?;
    let first_commit = repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])?;

    // `tree` is borrowing `repo`, preventing us from returning it
    drop(tree);

    Ok((repo, test_file, first_commit))
}

const GIT_TEST_FILE_UPDATED_CONTENTS: &str = r#"
program test
  logical*4, parameter :: true = .true.
  ! space out the diff hunks



  logical*4, parameter :: false = .false.



  ! new line, only this should get flagged
  logical*4 :: maybe
end program test
"#;

/// Test changes that have been staged in a git repo
#[test]
fn git_staged() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::new()?;
    let filename = Path::new("test.f90");
    let (repo, test_file, _) = set_up_git_repo(cmd.root())?;
    // Now edit the file
    fs::write(&test_file, GIT_TEST_FILE_UPDATED_CONTENTS)?;

    // git add
    let mut index = repo.index()?;
    index.add_path(filename)?;
    index.write()?;

    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args([
                             "--git-staged",
                             "--select=PORT011",
                         ]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:13:11: PORT011 logical kind set with number literal '4'
    fortitude: 1 files scanned.
    Number of errors: 1

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");

    Ok(())
}

/// Test filtering to just the commited changes on a git branch
#[test]
fn git_since() -> anyhow::Result<()> {
    let cmd = FortitudeCheck::new()?;
    let filename = Path::new("test.f90");
    let (repo, test_file, commit_oid) = set_up_git_repo(cmd.root())?;

    // git switch -c
    let commit = repo.find_commit(commit_oid)?;
    repo.branch("test-branch", &commit, false)?;

    // Now edit the file
    fs::write(&test_file, GIT_TEST_FILE_UPDATED_CONTENTS)?;

    // git add
    let mut index = repo.index()?;
    index.add_path(filename)?;
    index.write()?;
    // git commit
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let sig = git2::Signature::now("fortitude_test", "fortitude@example.com")?;
    repo.commit(Some("HEAD"), &sig, &sig, "second commit", &tree, &[&commit])?;

    assert_cmd_snapshot!(cmd.check_command()
                         .arg("test.f90")
                         .args([
                             "--git-since",
                             "test-branch",
                             "--select=PORT011",
                         ]),
                         @r"
    success: false
    exit_code: 1
    ----- stdout -----
    test.f90:13:11: PORT011 logical kind set with number literal '4'
    fortitude: 1 files scanned.
    Number of errors: 1

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");

    Ok(())
}

#[test_case::test_case("concise")]
#[test_case::test_case("full")]
#[test_case::test_case("json")]
#[test_case::test_case("json-lines")]
#[test_case::test_case("junit")]
#[test_case::test_case("grouped")]
#[test_case::test_case("github")]
#[test_case::test_case("gitlab")]
#[test_case::test_case("pylint")]
#[test_case::test_case("rdjson")]
#[test_case::test_case("azure")]
#[test_case::test_case("sarif")]
fn output_format(output_format: &str) -> Result<()> {
    const CONTENT: &str = "\
program test
  logical*4, parameter :: true = .true.
end program
";

    let cmd = FortitudeCheck::with_settings(|_project_dir, mut settings| {
        // JSON double escapes backslashes
        settings.add_filter(r#""[^"]+\\?/?test.f90"#, r#""[TMP]/test.f90"#);

        settings
    })?;

    cmd.write_file("test.f90", CONTENT)?;

    let snapshot = format!("output_format_{output_format}");

    assert_cmd_snapshot!(
        snapshot,
        cmd.check_command_plain().args([
            "--output-format",
            output_format,
            "--select",
            "S061,C001,PORT011,PORT021",
            "test.f90",
        ])
    );

    Ok(())
}

#[test_case::test_case("concise"; "concise_show_fixes")]
#[test_case::test_case("full"; "full_show_fixes")]
#[test_case::test_case("grouped"; "grouped_show_fixes")]
fn output_format_show_fixes(output_format: &str) -> Result<()> {
    let fixture = FortitudeCheck::with_file("test.f90", "program foo\nend program foo")?;
    let snapshot = format!("output_format_show_fixes_{output_format}");

    assert_cmd_snapshot!(
        snapshot,
        fixture.check_command_plain().args([
            "--output-format",
            output_format,
            "--select",
            "implicit-typing",
            "--fix",
            "--unsafe-fixes",
            "--show-fixes",
            "test.f90",
        ])
    );

    Ok(())
}

#[test]
fn config_override_rejected_if_invalid_toml() {
    assert_cmd_snapshot!(fortitude_cmd().arg("check")
        .args(STDIN_BASE_OPTIONS)
        .args(["--config", "foo = bar", "."]), @"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: invalid value 'foo = bar' for '--config <CONFIG_OPTION>'

      tip: A `--config` flag must either be a path to a `.toml` configuration file
           or a TOML `<KEY> = <VALUE>` pair overriding a specific configuration
           option

    The supplied argument is not valid TOML:

    TOML parse error at line 1, column 7
      |
    1 | foo = bar
      |       ^^^
    string values must be quoted, expected literal string

    For more information, try '--help'.
    ");
}

#[test]
fn too_many_config_files() -> Result<()> {
    let fixture = FortitudeCheck::new()?;
    fixture.write_file("fortitude.toml", "")?;
    fixture.write_file("fortitude2.toml", "")?;

    assert_cmd_snapshot!(fixture
        .check_command()
        .arg("--config")
        .arg("fortitude.toml")
        .arg("--config")
        .arg("fortitude2.toml")
        .arg("."), @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    fortitude failed
    Error: You cannot specify more than one configuration file on the command line.

      tip: remove either `--config=fortitude.toml` or `--config=fortitude2.toml`.
           For more information, try `--help`.
    ");
    Ok(())
}

#[test]
fn config_file_and_isolated() -> Result<()> {
    let fixture = FortitudeCheck::new()?;
    fixture.write_file("fortitude.toml", "")?;

    assert_cmd_snapshot!(fixture
        .check_command()
        .arg("--config")
        .arg("fortitude.toml")
        .arg("--isolated")
        .arg("."), @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    fortitude failed
    Error: The argument `--config=fortitude.toml` cannot be used with `--isolated`

      tip: You cannot specify a configuration file and also specify `--isolated`,
           as `--isolated` causes fortitude to ignore all configuration files.
           For more information, try `--help`.
    ");
    Ok(())
}

#[test]
fn config_override_via_cli() -> Result<()> {
    let fixture = FortitudeCheck::with_file(
        "fortitude.toml",
        r#"
[check]
line-length = 100
select = ["E"]
        "#,
    )?;
    let test_code = r#"
program test
integer, dimension(3) :: foo

x = "longer_than_90_charactersssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss"
end
"#;
    assert_cmd_snapshot!(fixture
        .check_command()
        .arg("--config")
        .arg("fortitude.toml")
        .args(["--config", "check.line-length=90"])
        .args(["--config", "check.extend-select=['S001', 'S263']"])
        .args(["--config", "check.inconsistent-dimensions.prefer-attribute = \"never\""])
        .arg("-")
        .pass_stdin(test_code), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    -:3:26: S263 Bad declaration of array
    -:5:91: S001 line length of 97, exceeds maximum 90
    fortitude: 1 files scanned.
    Number of errors: 2

    For more information about specific rules, run:

        fortitude explain X001,Y002,...

    No fixes available (1 hidden fix can be enabled with the `--unsafe-fixes` option).

    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn valid_toml_but_nonexistent_option_provided_via_config_argument() {
    assert_cmd_snapshot!(fortitude_cmd().arg("check")
        .args(STDIN_BASE_OPTIONS)
        .args([".", "--config", "check.extend-select=['NOTACODE']"]),
        @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: invalid value 'check.extend-select=['NOTACODE']' for '--config <CONFIG_OPTION>'

      tip: A `--config` flag must either be a path to a `.toml` configuration file
           or a TOML `<KEY> = <VALUE>` pair overriding a specific configuration
           option

    Could not parse the supplied argument as a `fortitude.toml` configuration option:

    Unknown rule selector: `NOTACODE`
    in `check.extend-select`

    For more information, try '--help'.
    ");
}

#[test]
fn each_toml_option_requires_a_new_flag_1() {
    assert_cmd_snapshot!(fortitude_cmd().arg("check")
        .args(STDIN_BASE_OPTIONS)
        // commas can't be used to delimit different config overrides;
        // you need a new --config flag for each override
        .args([".", "--config", "check.extend-select=['S001'], check.line-length=90"]),
        @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: invalid value 'check.extend-select=['S001'], check.line-length=90' for '--config <CONFIG_OPTION>'

      tip: A `--config` flag must either be a path to a `.toml` configuration file
           or a TOML `<KEY> = <VALUE>` pair overriding a specific configuration
           option

    The supplied argument is not valid TOML:

    TOML parse error at line 1, column 29
      |
    1 | check.extend-select=['S001'], check.line-length=90
      |                             ^
    unexpected key or value, expected newline, `#`

    For more information, try '--help'.
    ");
}

#[test]
fn each_toml_option_requires_a_new_flag_2() {
    assert_cmd_snapshot!(fortitude_cmd().arg("check")
        .args(STDIN_BASE_OPTIONS)
        // spaces *also* can't be used to delimit different config overrides;
        // you need a new --config flag for each override
        .args([".", "--config", "check.extend-select=['S001'] check.line-length=90"]),
        @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: invalid value 'check.extend-select=['S001'] check.line-length=90' for '--config <CONFIG_OPTION>'

      tip: A `--config` flag must either be a path to a `.toml` configuration file
           or a TOML `<KEY> = <VALUE>` pair overriding a specific configuration
           option

    The supplied argument is not valid TOML:

    TOML parse error at line 1, column 30
      |
    1 | check.extend-select=['S001'] check.line-length=90
      |                              ^
    unexpected key or value, expected newline, `#`

    For more information, try '--help'.
    ");
}

#[test]
fn config_doubly_overridden_via_cli() -> Result<()> {
    let fixture = FortitudeCheck::with_file(
        "fortitude.toml",
        r#"
[check]
line-length = 100
select=["S001"]
"#,
    )?;
    let test_code = "program test\nx = 'longer_than_90_charactersssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss'\nend";
    assert_cmd_snapshot!(fixture
        .check_command()
        // The --line-length flag takes priority over both the config file
        // and the `--config="check.line-length=110"` flag,
        // despite them both being specified after this flag on the command line:
        .args(["--line-length", "90"])
        .arg("--config")
        .arg("fortitude.toml")
        .args(["--config", "check.line-length=110"])
        .arg("-")
        .pass_stdin(test_code), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    -:2:91: S001 line length of 97, exceeds maximum 90
    fortitude: 1 files scanned.
    Number of errors: 1

    For more information about specific rules, run:

        fortitude explain X001,Y002,...


    ----- stderr -----
    ");
    Ok(())
}

#[test]
fn complex_config_setting_overridden_via_cli() -> Result<()> {
    let fixture = FortitudeCheck::with_file("fortitude.toml", "check.select = ['C001']")?;
    let test_code = "program violates_c001; end";
    assert_cmd_snapshot!(fixture
        .check_command()
        .arg("--config")
        .arg("fortitude.toml")
        .args(["--config", "check.per-file-ignores = {'generated.f90' = ['C001']}"])
        .args(["--stdin-filename", "generated.f90"])
        .arg("-")
        .pass_stdin(test_code), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    fortitude: 1 files scanned.
    All checks passed!


    ----- stderr -----
    ");
    Ok(())
}
