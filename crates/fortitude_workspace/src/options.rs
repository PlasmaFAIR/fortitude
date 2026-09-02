// Adapted from ruff
// Copyright 2022 Charles Marsh
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use ruff_macros::{CombineOptions, OptionsMetadata};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use fortitude_linter::{
    diagnostics::OutputRuleIdFormat,
    diagnostics::Severity,
    line_width::IndentWidth,
    rule_selector::RuleSelector,
    rules::{
        correctness::{exit_labels, shadowed_variable, use_statements},
        portability::{self},
        style::{
            complexity,
            inconsistent_dimension::{self, settings::PreferAttribute},
            keywords, line_length,
            strings::{self, settings::Quote},
            whitespace,
        },
    },
    settings::{FortranStandard, OutputFormat, ProgressBar},
    stylist::Capitalisation,
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
        default = r#"["*.f90", "*.F90", "*.f95", "*.F95", "*.f03", "*.F03", "*.f08", "*.F08", "*.f18", "*.F18", "*.f23", "*.F23", "*.pf"]"#,
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
    /// `"pylint"` (Pylint text format), `"azure"` (Azure Pipeline logging commands),
    /// `"name"` (filenames only), or `"count"` (filename + count of violations).
    #[option(
        default = r#""full""#,
        value_type = r#""full" | "concise" | "grouped" | "json" | "junit" | "github" | "gitlab" | "pylint" | "azure" | "name" | "count""#,
        example = r#"
            # Group violations by containing file.
            output-format = "grouped"
        "#
    )]
    pub output_format: Option<OutputFormat>,

    /// The default severity for violations. `"error"` (default) will report
    /// violations as errors. `"warning"` will report them as warnings, and
    /// `"info"` will report them as informational messages.
    ///
    /// Note that the LSP server instead, by default, reports all violations as
    /// `"warning"` except for violation of type `"error"` (`E`). Explicitly
    /// setting `severity-default` to `"warning"` will also downgrade `"error"`
    /// violations to `"warning"` in the LSP server.
    #[option(
        default = r#""error""#,
        value_type = r#""info" | "warning" | "error""#,
        example = r#"
               # Treat all violations only as informational messages.
               severity-default = "info"
            "#
    )]
    pub severity_default: Option<Severity>,

    /// Override the severity for specific rules.
    #[option(
        default = "{}",
        value_type = "dict[RuleSelector, str]",
        example = r#"
               # Treat `C001` as an error and `C003` as informational.
               severity-overrides = { C001 = "error", C003 = "info" }
           "#
    )]
    pub severity_overrides: Option<FxHashMap<RuleSelector, Severity>>,

    /// Whether to prefer rule codes, human-readable rule names, or both, in
    /// diagnostic output, even when preview mode is enabled.
    ///
    /// In preview mode, we now prefer to use human-readable rule names by
    /// default, but you can switch back to the older style with just a short
    /// code, or get both:
    ///
    /// ```console
    /// $ fortitude check --preview --config 'check.output-rule-id-format = "both"' --output-format=concise example.f90
    /// example.f90:1:8: error[implicit-typing (C001)] program uses implicit typing
    /// $ fortitude check --preview --config 'check.output-rule-id-format = "code"' --output-format=concise example.f90
    /// example.f90:1:8: error[C001] program uses implicit typing
    /// ```
    #[option(
        default = r#""name""#,
        value_type = r#""name" | "code" | "both""#,
        example = r#"
            # Display rule codes instead of human-readable rule names.
            output-format-rule-id = "code"
        "#
    )]
    pub output_rule_id_format: Option<OutputRuleIdFormat>,

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

    /// A list of rule codes or prefixes to consider fixable. By default,
    /// all rules are considered fixable.
    #[option(
        default = r#"["ALL"]"#,
        value_type = "list[RuleSelector]",
        example = r#"
            # Only allow fix behavior for style (`S`) and modernisation (`MOD`) rules.
            fixable = ["S", "MOD"]
        "#
    )]
    pub fixable: Option<Vec<RuleSelector>>,

    /// A list of rule codes or prefixes to consider non-fixable.
    #[option(
        default = "[]",
        value_type = "list[RuleSelector]",
        example = r#"
            # Disable fix for implicit-external-procedures (`C003`).
            unfixable = ["C003"]
        "#
    )]
    pub unfixable: Option<Vec<RuleSelector>>,

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

    /// A list of rule codes or prefixes to consider fixable, in addition to those
    /// specified by [`fixable`](#check_fixable).
    #[option(
        default = r#"[]"#,
        value_type = "list[RuleSelector]",
        example = r#"
            # On top of the current `fixable` rules, enable fix for implicit-typing (`C001`) and style rules (`S`).
            extend-fixable = ["C001", "S"]
        "#
    )]
    pub extend_fixable: Option<Vec<RuleSelector>>,

    // File resolver options
    /// A list of file extensions to check
    #[option(
        default = r#"["f90", "F90", "f95", "F95", "f03", "F03", "f08", "F08", "f18", "F18", "f23", "F23", "pf"]"#,
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

    // Global Formatting options
    /// The number of spaces to use for a single indent. Used when enforcing
    /// violations such as the use of tabs (`PORT031`) and incorrect indentation
    /// (`S105`).
    ///
    /// The indentation is determined by the number of spaces (tabs are equal to one indent_width).
    #[option(default = "4", value_type = "int", example = "indent-width = 2")]
    pub indent_width: Option<IndentWidth>,

    /// By default disable ignore-comment-length behavior when running `fortitude`.
    #[option(
        default = "false",
        value_type = "bool",
        example = "ignore-comment-length = true"
    )]
    pub ignore_comment_length: Option<bool>,

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

    /// Options for the `shadowed-variable` rule
    #[option_group]
    pub shadowed_variables: Option<ShadowedVariableOptions>,

    /// Options for the `incorrect-keyword-case` rule
    #[option_group]
    pub incorrect_keyword_case: Option<IncorrectKeywordCaseOptions>,

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

    /// Options for the `line-too-long` rule
    #[option_group]
    pub line_too_long: Option<LineTooLongOptions>,

    /// Options for rules related to the `use` statement, such as `use-all`.
    #[option_group]
    pub use_statements: Option<UseStatementsOptions>,

    /// Options for rules related to code complexity, such as `too-complex`,
    /// `too-many-arguments`, `too-many-nested-blocks`, etc.
    #[option_group]
    pub complexity: Option<ComplexityOptions>,

    /// Options for `incorrect-indentation` rule
    #[option_group]
    pub incorrect_indentation: Option<IncorrectIndentationOptions>,
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
            allow_unnested_loops: self.allow_unnested_loops.unwrap_or_default(),
        }
    }
}

/// Options for the `shadowed-variable` rule
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ShadowedVariableOptions {
    /// A list of variable names that are allowed to be shadowed. By default,
    /// common loop variables and error flags are allowed to be shadowed.
    #[option(
        default = r#"["i", "j", "k", "l", "m", "n", "ii", "jj", "kk", "ll", "mm", "nn", "idx", "index", "err", "ierr", "ioerr", "ios", "info", "stat", "iostat", "istat", "status"]"#,
        value_type = "list[str]",
        example = r#"allow = ["array", "x"]"#
    )]
    pub allow: Option<Vec<String>>,

    /// Strict mode will also check dummy arguments for violations
    #[option(default = "false", value_type = "bool", example = "strict = true")]
    pub strict: Option<bool>,
}

impl ShadowedVariableOptions {
    pub fn into_settings(self) -> shadowed_variable::settings::Settings {
        shadowed_variable::settings::Settings {
            allow: self.allow.unwrap_or_default(),
            strict: self.strict.unwrap_or_default(),
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
            inout_with_space: self.inout_with_space.unwrap_or_default(),
            goto_with_space: self.goto_with_space.unwrap_or_default(),
        }
    }
}

/// Options for the `incorrect-indentation` rule. The indent widths all default to using [`check.indent-width`](#check_indent-width)
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct IncorrectIndentationOptions {
    /// Whether lines containing semicolons should be ignored
    #[option(
        default = "true",
        value_type = "bool",
        example = "ignore-semicolons = false"
    )]
    pub ignore_semicolons: Option<bool>,

    /// The number of spaces to indent the contents of a program
    #[option(
        default = "`check.indent-width`",
        value_type = "int",
        example = "program-indent = 2"
    )]
    pub program_indent: Option<usize>,

    /// The number of spaces to indent the contents of modules and submodules
    #[option(
        default = "`check.indent-width`",
        value_type = "int",
        example = "module-indent = 2"
    )]
    pub module_indent: Option<usize>,

    /// The number of spaces to indent the contents of subroutines and functions
    #[option(
        default = "`check.indent-width`",
        value_type = "int",
        example = "procedure-indent = 2"
    )]
    pub procedure_indent: Option<usize>,

    /// The number of spaces to indent the contents of a derived type
    #[option(
        default = "`check.indent-width`",
        value_type = "int",
        example = "derived-type-indent = 2"
    )]
    pub derived_type_indent: Option<usize>,

    /// The number of spaces to indent the contents of control flow units (i.e. `block`, `if`, `associate`, `do`, `select`)
    #[option(
        default = "`check.indent-width`",
        value_type = "int",
        example = "control-flow-indent = 2"
    )]
    pub control_flow_indent: Option<usize>,

    /// The number of spaces to indent the contents of a interface
    #[option(
        default = "`check.indent-width`",
        value_type = "int",
        example = "interface-indent = 2"
    )]
    pub interface_indent: Option<usize>,

    /// The number of spaces to indent after a line continuation (`&`)
    #[option(
        default = "`check.indent-width`",
        value_type = "int",
        example = "line-continuation-indent = 2"
    )]
    pub line_continuation_indent: Option<usize>,
}

impl IncorrectIndentationOptions {
    pub fn into_settings(
        self,
        default_indent: usize,
    ) -> whitespace::settings::IncorrectIndentationSettings {
        use whitespace::settings::IncorrectIndentationSettings;

        IncorrectIndentationSettings {
            ignore_semicolons: self
                .ignore_semicolons
                .unwrap_or(IncorrectIndentationSettings::default().ignore_semicolons),
            program_indent: self.program_indent.unwrap_or(default_indent),
            module_indent: self.module_indent.unwrap_or(default_indent),
            procedure_indent: self.procedure_indent.unwrap_or(default_indent),
            derived_type_indent: self.derived_type_indent.unwrap_or(default_indent),
            control_flow_indent: self.control_flow_indent.unwrap_or(default_indent),
            interface_indent: self.interface_indent.unwrap_or(default_indent),
            line_continuation_indent: self.line_continuation_indent.unwrap_or(default_indent),
        }
    }
}

/// Options for the `incorrect-keyword-case` rule
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct IncorrectKeywordCaseOptions {
    /// Preferred casing for Fortran keywords, as enforced by the [`incorrect-keyword-case`](rules/incorrect-keyword-case.md) rule.
    ///
    /// Defaults to `"lowercase"`, consistent with modern Fortran conventions.
    #[option(
        default = "lowercase",
        value_type = r#""lowercase" | "uppercase" | "titlecase""#,
        example = r#"keyword-case = "lowercase""#
    )]
    pub keyword_case: Option<Capitalisation>,
}

impl IncorrectKeywordCaseOptions {
    pub fn into_settings(self) -> keywords::settings_keyword_case::Settings {
        keywords::settings_keyword_case::Settings {
            keyword_case: self.keyword_case.unwrap_or_default(),
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
    #[deprecated(
        since = "0.10.0",
        note = "`check.invalid-tab.indent-width` has been renamed to [`check.indent-width`](#check_indent-width). Please updated your configuration to use that instead."
    )]
    pub indent_width: Option<IndentWidth>,
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

/// Options for `line-too-long` rule
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct LineTooLongOptions {
    /// If `true`, don't take comments into account when checking if a line is
    /// too long. This can be useful when dealing with existing codebases with
    /// long comments, for instance, or inline comments used for other tools.
    #[option(
        default = "false",
        value_type = "bool",
        example = "ignore-comments = true"
    )]
    pub ignore_comments: Option<bool>,
}

impl LineTooLongOptions {
    pub fn into_settings(self) -> line_length::settings::Settings {
        line_length::settings::Settings {
            ignore_comments: self.ignore_comments.unwrap_or_default(),
        }
    }
}

/// Options for the `use` statement rules
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct UseStatementsOptions {
    /// List of exceptions to the [`use-all`](rules/use-all.md) rule.  That is, modules allowed to
    /// appear in a `use` statement without an `only` clause.
    ///
    /// While it is recommended to list all `use`d components in an `only` clause, this can
    /// occasionally be impractical for some modules. For example, if the `only` list would
    /// commonly be very long, or would often list all or nearly all of the module's contents.
    ///
    /// Note that this option is intended for modules that are safe to `use` without an `only`
    /// clause across the whole codebase.  For one-off instances, consider [inline error
    /// suppression comments](linter.md#error-suppression) such as `! allow(use-all)` instead.
    #[option(
        default = "[]",
        value_type = r#"list[str]"#,
        example = r#"allow-bare-use = ["utils"]"#
    )]
    pub allow_bare_use: Option<Vec<String>>,
}

impl UseStatementsOptions {
    pub fn into_settings(self) -> use_statements::settings::Settings {
        use_statements::settings::Settings {
            allow_bare_use: self
                .allow_bare_use
                .unwrap_or_default()
                .iter()
                .map(|m| m.to_lowercase())
                .collect(),
        }
    }
}

/// Options for `too-complex` rule
#[derive(
    Clone, Debug, PartialEq, Eq, Default, OptionsMetadata, CombineOptions, Serialize, Deserialize,
)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ComplexityOptions {
    /// The maximum cyclomatic complexity allowed for a procedure.
    /// Procedures exceeding this threshold will be flagged.
    #[option(default = "10", value_type = "usize", example = "max-complexity = 15")]
    pub max_complexity: Option<usize>,

    /// The maximum number of arguments allowed for a procedure.
    /// Procedures exceeding this threshold will be flagged.
    #[option(default = "5", value_type = "usize", example = "max-args = 15")]
    pub max_args: Option<usize>,
}

impl ComplexityOptions {
    pub fn into_settings(self) -> complexity::settings::Settings {
        complexity::settings::Settings {
            max_complexity: self.max_complexity.unwrap_or(10),
            max_args: self.max_args.unwrap_or(5),
        }
    }
}
