// Adapted from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use ruff_macros::{CombineOptions, OptionsMetadata};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use fortitude_linter::{
    line_width::IndentWidth,
    rule_selector::RuleSelector,
    rules::{
        correctness::exit_labels,
        portability::{self, invalid_tab},
        style::{
            inconsistent_dimension::{self, settings::PreferAttribute},
            keywords,
            strings::{self, settings::Quote},
        },
    },
    settings::{FortranStandard, OutputFormat, ProgressBar},
};

#[derive(Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Options {
    /// A list of file patterns to include when linting.
    ///
    /// Inclusion are based on globs, and should be single-path patterns, like
    /// `*.f90`, to include any file with the `.f90` extension.
    ///
    /// For more information on the glob syntax, refer to the [`globset` documentation](https://docs.rs/globset/latest/globset/#syntax).
    ///
    /// !!! info "_Introduced in 0.8.0_"
    #[option(
        default = r#"["*.f90", "*.F90", "*.f95", "*.F95", "*.f03", "*.F03", "*.f08", "*.F08", "*.f18", "*.F18", "*.f23", "*.F23"]"#,
        value_type = "list[str]",
        example = r#"
            include = ["*.f90", "*.F90"]
        "#
    )]
    pub include: Option<Vec<String>>,

    #[option_group]
    pub check: Option<CheckOptions>,
}

/// Configures how Fortitude checks your code.
///
/// Options specified in the `check` section take precedence over the deprecated top-level settings.
#[derive(Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct CheckOptions {
    /// A list of file patterns to include when linting.
    ///
    /// Inclusion are based on globs, and should be single-path patterns, like
    /// `*.f90`, to include any file with the `.f90` extension.
    ///
    /// For more information on the glob syntax, refer to the [`globset` documentation](https://docs.rs/globset/latest/globset/#syntax).
    #[option(
        default = r#"["."]"#,
        value_type = "list[str]",
        example = r#"
            files = ["foo.f90"]
        "#
    )]
    #[deprecated(
        since = "0.8.0",
        note = "The `files` option is now deprecated in favour of the top-level [`include`](#include). Please update your configuration to use the [`include`](#include) instead."
    )]
    pub files: Option<Vec<PathBuf>>,

    /// Enable fix behavior by-default when running `fortitude` (overridden
    /// by the `--fix` and `--no-fix` command-line flags).
    /// Only includes automatic fixes unless `--unsafe-fixes` is provided.
    #[option(default = "false", value_type = "bool", example = "fix = true")]
    pub fix: Option<bool>,

    /// Enable application of unsafe fixes.
    /// If excluded, a hint will be displayed when unsafe fixes are available.
    /// If set to false, the hint will be hidden.
    #[option(
        default = r#"null"#,
        value_type = "bool",
        example = "unsafe-fixes = true"
    )]
    pub unsafe_fixes: Option<bool>,

    /// Whether to show an enumeration of all fixed lint violations
    /// (overridden by the `--show-fixes` command-line flag).
    #[option(
        default = "false",
        value_type = "bool",
        example = r#"
            # Enumerate all fixed violations.
            show-fixes = true
        "#
    )]
    pub show_fixes: Option<bool>,

    /// Like [`fix`](#fix), but disables reporting on leftover violation. Implies [`fix`](#fix).
    #[option(default = "false", value_type = "bool", example = "fix-only = true")]
    pub fix_only: Option<bool>,

    /// The style in which violation messages should be formatted: `"full"` (default)
    /// (shows source), `"concise"`, `"grouped"` (group messages by file), `"json"`
    /// (machine-readable), `"junit"` (machine-readable XML), `"github"` (GitHub
    /// Actions annotations), `"gitlab"` (GitLab CI code quality report),
    /// `"pylint"` (Pylint text format) or `"azure"` (Azure Pipeline logging commands).
    #[option(
        default = r#""full""#,
        value_type = r#""full" | "concise" | "grouped" | "json" | "junit" | "github" | "gitlab" | "pylint" | "azure""#,
        example = r#"
            # Group violations by containing file.
            output-format = "grouped"
        "#
    )]
    pub output_format: Option<OutputFormat>,

    /// Whether to enable preview mode. When preview mode is enabled, Fortitude will
    /// use unstable rules, fixes, and formatting.
    #[option(
        default = "false",
        value_type = "bool",
        example = r#"
            # Enable preview features.
            preview = true
        "#
    )]
    pub preview: Option<bool>,

    /// Minimum Fortran standard to check files against.
    /// Options are "f2023", "f2018" (default), "f2008", "f2003", and "f95".
    #[option(
        default = "f2018",
        value_type = r#""f2023" | "f2018" | "f2008" | "f2003" | "f95""#,
        example = r#"
          # Set standard to Fortran 2008
          target-std = "f2008"
       "#
    )]
    pub target_std: Option<FortranStandard>,

    /// Progress bar settings.
    /// Options are "off" (default), "ascii", and "fancy"
    #[option(
        default = "off",
        value_type = "str",
        scope = "progress-bar",
        example = r#"
          # Enable unicode progress bar
          progress-bar = "fancy"
       "#
    )]
    pub progress_bar: Option<ProgressBar>,

    // Rule selection
    /// A list of rule codes or prefixes to ignore. Prefixes can specify exact
    /// rules (like `S201` or `superfluous-implicit-none`), entire categories
    /// (like `C` or `correctness`), or anything in between.
    ///
    /// When breaking ties between enabled and disabled rules (via `select` and
    /// `ignore`, respectively), more specific prefixes override less
    /// specific prefixes.
    #[option(
        default = "[]",
        value_type = "list[RuleSelector]",
        example = r#"ignore = ["superfluous-implicit-none"]"#
    )]
    pub ignore: Option<Vec<RuleSelector>>,

    /// A list of rule codes or prefixes to enable. Prefixes can specify exact
    /// rules (like `S201` or `superfluous-implicit-none`), entire categories
    /// (like `C` or `correctness`), or anything in between.
    ///
    /// By default, a curated set of rules across all categories is enabled; see
    /// the documentation for details.
    ///
    /// When breaking ties between enabled and disabled rules (via `select` and
    /// `ignore`, respectively), more specific prefixes override less
    /// specific prefixes.
    #[option(
        default = "[]",
        value_type = "list[RuleSelector]",
        example = r#"
            # Only check errors and obsolescent features
            select = ["E", "OB"]
        "#
    )]
    pub select: Option<Vec<RuleSelector>>,

    /// A list of rule codes or prefixes to enable, in addition to those
    /// specified by [`select`](#check_select).
    #[option(
        default = "[]",
        value_type = "list[RuleSelector]",
        example = r#"
            # On top of the current `select` rules, enable missing-intent (`C061`) and portability rules (`PORT`).
            extend-select = ["C061", "PORT"]
        "#
    )]
    pub extend_select: Option<Vec<RuleSelector>>,

    // File resolver options
    /// A list of file extensions to check
    #[option(
        default = r#"["f90", "F90", "f95", "F95", "f03", "F03", "f08", "F08", "f18", "F18", "f23", "F23"]"#,
        value_type = "list[str]",
        example = r#"
          file-extensions = ["f90", "fpp"]
        "#
    )]
    #[deprecated(
        since = "0.8.0",
        note = "The `file_extensions` option is now deprecated in favour of the top-level [`include`](#include). Please update your configuration to use the [`include`](#include) instead."
    )]
    pub file_extensions: Option<Vec<String>>,

    /// A list of file patterns to exclude from formatting and linting.
    ///
    /// Exclusions are based on globs, and can be either:
    ///
    /// - Single-path patterns, like `build` (to exclude any directory named
    ///   `build` in the tree), `foo.f90` (to exclude any file named `foo.f90`),
    ///   or `foo_*.f90` (to exclude any file matching `foo_*.f90`).
    /// - Relative patterns, like `directory/foo.f90` (to exclude that specific
    ///   file) or `directory/*.f90` (to exclude any Fortran files in
    ///   `directory`). Note that these paths are relative to the project root
    ///   (e.g., the directory containing your `fpm.toml`).
    ///
    /// For more information on the glob syntax, refer to the [`globset` documentation](https://docs.rs/globset/latest/globset/#syntax).
    ///
    /// Note that you'll typically want to use
    /// [`extend-exclude`](#extend-exclude) to modify the excluded paths.
    #[option(
        default = r#"[".git", ".git-rewrite", ".hg", ".svn", "venv", ".venv", "pyenv", ".pyenv", ".eggs", "site-packages", ".vscode", "build", "_build", "dist", "_dist"]"#,
        value_type = "list[str]",
        example = r#"
            exclude = [".venv"]
        "#
    )]
    pub exclude: Option<Vec<String>>,

    /// A list of file patterns to omit from formatting and linting, in addition to those
    /// specified by [`exclude`](#exclude).
    ///
    /// Exclusions are based on globs, and can be either:
    ///
    /// - Single-path patterns, like `build` (to exclude any directory named
    ///   `build` in the tree), `foo.f90` (to exclude any file named `foo.f90`),
    ///   or `foo_*.f90` (to exclude any file matching `foo_*.f90`).
    /// - Relative patterns, like `directory/foo.f90` (to exclude that specific
    ///   file) or `directory/*.f90` (to exclude any Fortran files in
    ///   `directory`). Note that these paths are relative to the project root
    ///   (e.g., the directory containing your `fpm.toml`).
    ///
    /// For more information on the glob syntax, refer to the [`globset` documentation](https://docs.rs/globset/latest/globset/#syntax).
    #[option(
        default = "[]",
        value_type = "list[str]",
        example = r#"
            # In addition to the standard set of exclusions, omit all tests, plus a specific file.
            extend-exclude = ["tests", "src/bad.f90"]
        "#
    )]
    pub extend_exclude: Option<Vec<String>>,

    /// Whether to enforce [`exclude`](#exclude) and [`extend-exclude`](#extend-exclude) patterns,
    /// even for paths that are passed to Fortitude explicitly. Typically, Fortitude will lint
    /// any paths passed in directly, even if they would typically be
    /// excluded. Setting `force-exclude = true` will cause Fortitude to
    /// respect these exclusions unequivocally.
    ///
    /// This is useful for CI jobs which might explicitly pass all changed
    /// files, regardless of whether they're marked as excluded by Fortitude's
    /// own settings.
    #[option(
        default = r#"false"#,
        value_type = "bool",
        example = r#"
            force-exclude = true
        "#
    )]
    pub force_exclude: Option<bool>,

    /// Whether to automatically exclude files that are ignored by `.ignore`,
    /// `.gitignore`, `.git/info/exclude`, and global `gitignore` files.
    /// Enabled by default.
    #[option(
        default = "true",
        value_type = "bool",
        example = r#"
            respect-gitignore = false
        "#
    )]
    pub respect_gitignore: Option<bool>,

    // Global Formatting options
    /// The line length to use when enforcing long-lines violations (like `S001`).
    ///
    /// The length is determined by the number of characters per line, except for lines containing East Asian characters or emojis.
    /// For these lines, the [unicode width](https://unicode.org/reports/tr11/) of each character is added up to determine the length.
    #[option(
        default = "100",
        value_type = "int",
        example = r#"
        # Allow lines to be as long as 120.
        line-length = 120
        "#
    )]
    pub line_length: Option<usize>,

    // Tables are required to go last.
    /// A list of mappings from file pattern to rule codes or prefixes to
    /// exclude, when considering any matching files. An initial '!' negates
    /// the file pattern.
    #[option(
        default = "{}",
        value_type = "dict[str, list[RuleSelector]]",
        scope = "per-file-ignores",
        example = r#"
            # Ignore `S201` (superfluous implicit none) in all `test.f90` files, and in `path/to/file.f90`.
            "test.f90" = ["S201"]
            "path/to/file.f90" = ["S201"]
            # Ignore `S` rules everywhere except for the `src/` directory.
            "!src/**.f90" = ["S"]
        "#
    )]
    pub per_file_ignores: Option<FxHashMap<String, Vec<RuleSelector>>>,

    /// Options for the `exit-or-cycle-in-unlabelled-loops` rule
    #[option_group]
    pub exit_unlabelled_loops: Option<ExitUnlabelledLoopOptions>,

    /// Options for the `keyword-missing-space` and `keyword-has-whitespace` rules
    #[option_group]
    pub keyword_whitespace: Option<KeywordWhitespaceOptions>,

    /// Options for the `bad-string-quote` rule
    #[option_group]
    pub strings: Option<StringOptions>,

    /// Options for the `portability` set of rules
    #[option_group]
    pub portability: Option<PortabilityOptions>,

    /// Options for the `invalid-tab` rule
    #[option_group]
    pub invalid_tab: Option<InvalidTabOptions>,

    /// Options for the `inconsistent-dimensions` set of rules
    #[option_group]
    pub inconsistent_dimensions: Option<InconsistentDimensionOptions>,
}

/// Options for the `exit-or-cycle-in-unlabelled-loops` rule
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ExitUnlabelledLoopOptions {
    /// Whether to check for `exit`/`cycle` in unlabelled loops only if the loop has at
    /// least one level of nesting. With this setting off (default), the following will
    /// raise a warning, and with it on, it won't:
    ///
    /// ```f90
    /// do i = 1, 100
    ///     if (i == 50) exit
    /// end do
    /// ```
    #[option(
        default = "false",
        value_type = "bool",
        example = "allow-unnested-loops = true"
    )]
    pub allow_unnested_loops: Option<bool>,
}

impl ExitUnlabelledLoopOptions {
    pub fn into_settings(self) -> exit_labels::settings::Settings {
        exit_labels::settings::Settings {
            allow_unnested_loops: self.allow_unnested_loops.unwrap_or(false),
        }
    }
}

/// Options for the `keyword-missing-space` and `keyword-has-whitespace` rules
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct KeywordWhitespaceOptions {
    /// Whether to enforce the use of `in out` instead of `inout`.
    #[option(
        default = "false",
        value_type = "bool",
        example = "inout-with-space = true"
    )]
    pub inout_with_space: Option<bool>,

    /// Whether to enforce the use of `go to` instead of `goto`.
    #[option(
        default = "false",
        value_type = "bool",
        example = "goto-with-space = true"
    )]
    pub goto_with_space: Option<bool>,
}

impl KeywordWhitespaceOptions {
    pub fn into_settings(self) -> keywords::settings::Settings {
        keywords::settings::Settings {
            inout_with_space: self.inout_with_space.unwrap_or(false),
            goto_with_space: self.goto_with_space.unwrap_or(false),
        }
    }
}

/// Options for the string literal rules
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct StringOptions {
    /// Quote style to prefer for string literals (either "single" or "double").
    #[option(
        default = r#""double""#,
        value_type = r#""single" | "double""#,
        example = r#"quotes = "single""#
    )]
    pub quotes: Option<Quote>,
}

impl StringOptions {
    pub fn into_settings(self) -> strings::settings::Settings {
        strings::settings::Settings {
            quotes: self.quotes.unwrap_or_default(),
        }
    }
}

/// Options for the portability rules
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct PortabilityOptions {
    /// Whether to allow file units of `100`, `101`, `102` in `read/write` statements
    /// for [`non-portable-io-unit`](rules/non-portable-io-unit.md). The Cray
    /// compiler pre-connects these to `stdin`, `stdout`, and `stderr`,
    /// respectively. However, if you are `open`-ing these units explicitly, you may
    /// wish to switch this to `true` -- but see also
    /// [`magic-io-unit`](rules/magic-io-unit.md).
    #[option(
        default = "false",
        value_type = "bool",
        example = "allow-cray-file-units = true"
    )]
    pub allow_cray_file_units: Option<bool>,
}

impl PortabilityOptions {
    pub fn into_settings(self) -> portability::settings::Settings {
        portability::settings::Settings {
            allow_cray_file_units: self.allow_cray_file_units.unwrap_or_default(),
        }
    }
}

/// Options for `invalid-tab` rule
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct InvalidTabOptions {
    /// The number of spaces to replace tabs with.
    #[option(default = "4", value_type = "int", example = "indent-width = 2")]
    pub indent_width: Option<IndentWidth>,
}

impl InvalidTabOptions {
    pub fn into_settings(self) -> invalid_tab::settings::Settings {
        invalid_tab::settings::Settings {
            indent_width: self.indent_width.unwrap_or_default(),
        }
    }
}

/// Options for `inconsistent-dimension` set of rules
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct InconsistentDimensionOptions {
    /// Prefer declaring arrays using the `dimension` attribute rather than an
    /// inline shape, `foo(N, M)` or vice-versa.
    ///
    /// Default behaviour is to keep the current method.
    #[option(
        default = "keep",
        value_type = r#""keep" | "always" | "never""#,
        example = r#"prefer-attribute = "always""#
    )]
    pub prefer_attribute: Option<PreferAttribute>,
}

impl InconsistentDimensionOptions {
    pub fn into_settings(self) -> inconsistent_dimension::settings::Settings {
        inconsistent_dimension::settings::Settings {
            prefer_attribute: self.prefer_attribute.unwrap_or_default(),
        }
    }
}
