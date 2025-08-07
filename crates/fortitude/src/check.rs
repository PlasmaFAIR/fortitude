use crate::cli::{CheckCommand, GlobalConfigArgs};
use crate::printer::{Flags as PrinterFlags, Printer};
use crate::show_files::show_files;
use crate::show_settings::show_settings;
use crate::stdin::read_from_stdin;
use fortitude_linter::diagnostic_message::DiagnosticMessage;
use fortitude_linter::diagnostics::{Diagnostics, FixMap};
use fortitude_linter::fs::{self, get_files, read_to_string};
use fortitude_linter::rule_table::RuleTable;
use fortitude_linter::rules::Rule;
use fortitude_linter::rules::{error::ioerror::IoError, AstRuleEnum, PathRuleEnum, TextRuleEnum};
use fortitude_linter::settings::{self, CheckSettings, FixMode, ProgressBar, Settings};
use fortitude_linter::{
    ast_entrypoint_map, check_and_fix_file, check_file, check_only_file, rules_to_path_rules,
    rules_to_text_rules, FixerResult,
};
use fortitude_linter::{warn_user_once, warn_user_once_by_message};
use fortitude_workspace::configuration::{
    self, parse_config_file, Configuration, ConfigurationTransformer,
};

use anyhow::Result;
use colored::Colorize;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use log::{debug, warn};
use rayon::prelude::*;
use ruff_diagnostics::Diagnostic;
use ruff_source_file::SourceFileBuilder;
use ruff_text_size::TextRange;
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

/// Returns true if the command should read from standard input.
fn is_stdin(files: &[PathBuf], stdin_filename: Option<&Path>) -> bool {
    // If the user provided a `--stdin-filename`, always read from standard input.
    if stdin_filename.is_some() {
        if let Some(file) = files.iter().find(|file| file.as_path() != Path::new("-")) {
            warn_user_once!(
                "Ignoring file {} in favor of standard input.",
                file.display()
            );
        }
        return true;
    }

    let [file] = files else {
        return false;
    };
    // If the user provided exactly `-`, read from standard input.
    file == Path::new("-")
}

/// Helper object to store the results of all checks
pub(crate) struct CheckResults {
    /// All diagnostics found in all files
    pub(crate) diagnostics: Diagnostics,
    /// The number of files checked
    pub(crate) files_checked: usize,
    /// The number of files skipped
    pub(crate) files_skipped: usize,
}

impl CheckResults {
    fn new() -> Self {
        Self {
            diagnostics: Diagnostics::default(),
            files_checked: 0,
            files_skipped: 0,
        }
    }

    fn from_stdin(diagnostics: Diagnostics) -> Self {
        Self {
            diagnostics,
            files_checked: 1,
            files_skipped: 0,
        }
    }

    fn add(mut self, status: CheckStatus) -> Self {
        match status {
            CheckStatus::Ok => self.files_checked += 1,
            CheckStatus::Violations(diagnostics) => {
                self.diagnostics += diagnostics;
                self.files_checked += 1;
            }
            CheckStatus::Skipped(diagnostics) => {
                self.diagnostics += diagnostics;
                self.files_skipped += 1;
            }
            CheckStatus::SkippedNoDiagnostic => self.files_skipped += 1,
        }
        self
    }

    fn merge(mut self, other: CheckResults) -> Self {
        self.diagnostics += other.diagnostics;
        self.files_checked += other.files_checked;
        self.files_skipped += other.files_skipped;
        self
    }

    fn sort(&mut self) {
        self.diagnostics.messages.par_sort_unstable();
    }
}

/// Enum used to report the result of a single file check
enum CheckStatus {
    /// The file was checked and no issues were found
    Ok,
    /// The file was checked and issues were found
    Violations(Diagnostics),
    /// The file was skipped due to an error
    Skipped(Diagnostics),
    /// The file was skipped but no violations were raised
    SkippedNoDiagnostic,
}

/// Check all files, report issues found, and return error code.
pub fn check(args: CheckCommand, global_options: &GlobalConfigArgs) -> Result<ExitCode> {
    let (cli, config_arguments) = args.partition()?;

    // First we need to find and read any config file
    let project_root = configuration::project_root(path_absolutize::path_dedot::CWD.as_path())?;
    let file_configuration = Configuration::from_options(
        parse_config_file(&global_options.config_file)?,
        &project_root,
    );

    // Now, we can override settings from the config file with options
    // from the CLI
    let config = config_arguments.transform(file_configuration);
    let settings = config.into_settings(&project_root)?;

    let stdin_filename = cli.stdin_filename;

    let mut writer: Box<dyn Write> = match cli.output_file {
        Some(path) => {
            colored::control::set_override(false);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = File::create(path)?;
            Box::new(BufWriter::new(file))
        }
        _ => Box::new(BufWriter::new(io::stdout())),
    };
    let stderr_writer = Box::new(BufWriter::new(io::stderr()));

    let is_stdin = is_stdin(&settings.file_resolver.files, stdin_filename.as_deref());

    if cli.show_settings {
        show_settings(&settings, &mut writer)?;
        return Ok(ExitCode::SUCCESS);
    }

    if cli.show_files {
        show_files(&settings.file_resolver, is_stdin, &mut writer)?;
        return Ok(ExitCode::SUCCESS);
    }

    let CheckSettings {
        fix,
        fix_only,
        ref rules,
        unsafe_fixes,
        show_fixes,
        output_format,
        ..
    } = settings.check;

    // Fix rules are as follows:
    // - By default, generate all fixes, but don't apply them to the filesystem.
    // - If `--fix` or `--fix-only` is set, apply applicable fixes to the filesystem (or
    //   print them to stdout, if we're reading from stdin).
    // - If `--diff` or `--fix-only` are set, don't print any violations (only applicable fixes)
    // - By default, applicable fixes only include [`Applicablility::Automatic`], but if
    //   `--unsafe-fixes` is set, then [`Applicablility::Suggested`] fixes are included.

    let fix_mode = if fix || fix_only {
        FixMode::Apply
    } else {
        FixMode::Generate
    };

    // At this point, we've assembled all our settings, and we're
    // ready to check the project

    let path_rules = rules_to_path_rules(rules);
    let text_rules = rules_to_text_rules(rules);
    let ast_entrypoints = ast_entrypoint_map(rules);

    let start = Instant::now();

    let files = get_files(&settings.file_resolver, is_stdin)?;
    debug!("Identified files to lint in: {:?}", start.elapsed());

    let results = if is_stdin {
        check_stdin(
            stdin_filename.map(fs::normalize_path).as_deref(),
            rules,
            &path_rules,
            &text_rules,
            &ast_entrypoints,
            &settings,
            fix_mode,
            cli.ignore_allow_comments,
        )?
    } else {
        check_files(
            &files,
            rules,
            &path_rules,
            &text_rules,
            &ast_entrypoints,
            &settings,
            fix_mode,
            cli.ignore_allow_comments,
        )?
    };

    // Always try to print violations (though the printer itself may suppress output)
    // If we're writing fixes via stdin, the transformed source code goes to the writer
    // so send the summary to stderr instead
    let mut summary_writer = if is_stdin && matches!(fix_mode, FixMode::Apply | FixMode::Diff) {
        stderr_writer
    } else {
        writer
    };

    let mut printer_flags = PrinterFlags::empty();
    if !fix_only {
        printer_flags |= PrinterFlags::SHOW_VIOLATIONS;
    }
    if show_fixes {
        printer_flags |= PrinterFlags::SHOW_FIX_SUMMARY;
    }

    let printer = Printer::new(
        output_format,
        global_options.log_level(),
        printer_flags,
        fix_mode,
        unsafe_fixes,
    );

    if cli.statistics {
        printer.write_statistics(&results, &mut summary_writer)?;
    } else {
        printer.write_once(&results, &mut summary_writer)?;
    }

    let diagnostics = results.diagnostics;
    if !cli.exit_zero {
        if fix_only {
            // If we're only fixing, we want to exit zero (since we've fixed all fixable
            // violations), unless we're explicitly asked to exit non-zero on fix.
            if cli.exit_non_zero_on_fix && !diagnostics.fixed.is_empty() {
                return Ok(ExitCode::FAILURE);
            }
        } else {
            // If we're running the linter (not just fixing), we want to exit non-zero if
            // there are any violations, unless we're explicitly asked to exit zero on
            // fix.
            if cli.exit_non_zero_on_fix {
                if !diagnostics.fixed.is_empty() || !diagnostics.messages.is_empty() {
                    return Ok(ExitCode::FAILURE);
                }
            } else if !diagnostics.messages.is_empty() {
                return Ok(ExitCode::FAILURE);
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

#[allow(clippy::too_many_arguments)]
fn check_files(
    files: &[PathBuf],
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    settings: &Settings,
    fix_mode: FixMode,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> Result<CheckResults> {
    let file_digits = files.len().to_string().len();
    let progress_bar_style = match settings.check.progress_bar {
        ProgressBar::Fancy => {
            // Make progress bar with 60 char width, bright cyan colour (51)
            // Colours use some 8-bit representation
            let style_template = format!(
                "{{prefix}} {{pos:>{file_digits}}}/{{len}} [{{bar:60.51}}] [{{elapsed_precise}}]"
            );
            ProgressStyle::with_template(style_template.as_str())
                .unwrap()
                .progress_chars("━╸ ")
            // Alt: sub-character resolution "█▉▊▋▌▍▎▏  "
        }
        ProgressBar::Ascii => {
            // Same as fancy, but without colours and using basic characters
            let style_template = format!(
                "{{prefix}} {{pos:>{file_digits}}}/{{len}} [{{bar:60}}] [{{elapsed_precise}}]"
            );
            ProgressStyle::with_template(style_template.as_str())
                .unwrap()
                .progress_chars("=> ")
        }
        ProgressBar::Off => ProgressStyle::with_template("").unwrap(),
    };

    let start = Instant::now();
    let mut results = files
        .par_iter()
        .progress_with_style(progress_bar_style)
        .with_prefix("Checking file:")
        .map(|path| {
            let filename = path.to_string_lossy();

            let source = match read_to_string(path) {
                Ok(source) => source,
                Err(error) => {
                    if rules.enabled(Rule::IoError) {
                        let message = format!("Error opening file: {error}");
                        let diagnostics = vec![DiagnosticMessage::from_error(
                            filename,
                            Diagnostic::new(IoError { message }, TextRange::default()),
                        )];
                        return CheckStatus::Skipped(Diagnostics::new(diagnostics));
                    } else {
                        warn!(
                            "{}{}{} {error}",
                            "Error opening file ".bold(),
                            fs::relativize_path(path).bold(),
                            ":".bold()
                        );
                        return CheckStatus::SkippedNoDiagnostic;
                    }
                }
            };

            let file = SourceFileBuilder::new(filename.as_ref(), source.as_str()).finish();

            match check_file(
                rules,
                path_rules,
                text_rules,
                ast_entrypoints,
                path,
                &file,
                settings,
                fix_mode,
                ignore_allow_comments,
            ) {
                Ok(violations) => {
                    if violations.is_empty() {
                        CheckStatus::Ok
                    } else {
                        CheckStatus::Violations(violations)
                    }
                }
                Err(msg) => {
                    if rules.enabled(Rule::IoError) {
                        let message = format!("Failed to process: {msg}");
                        let diagnostics = vec![DiagnosticMessage::from_error(
                            filename,
                            Diagnostic::new(IoError { message }, TextRange::default()),
                        )];
                        CheckStatus::Skipped(Diagnostics::new(diagnostics))
                    } else {
                        warn!(
                            "{}{}{} {msg}",
                            "Failed to process ".bold(),
                            fs::relativize_path(path).bold(),
                            ":".bold()
                        );
                        CheckStatus::SkippedNoDiagnostic
                    }
                }
            }
        })
        .fold(CheckResults::new, |results, status| results.add(status))
        .reduce(CheckResults::new, |a, b| a.merge(b));

    results.sort();

    let duration = start.elapsed();
    debug!(
        "Checked {:?} files and skipped {:?} in: {:?}",
        results.files_checked, results.files_skipped, duration
    );

    Ok(results)
}

#[allow(clippy::too_many_arguments)]
fn check_stdin(
    filename: Option<&Path>,
    rules: &RuleTable,
    path_rules: &Vec<PathRuleEnum>,
    text_rules: &Vec<TextRuleEnum>,
    ast_entrypoints: &BTreeMap<&str, Vec<AstRuleEnum>>,
    settings: &Settings,
    fix_mode: FixMode,
    ignore_allow_comments: settings::IgnoreAllowComments,
) -> Result<CheckResults> {
    let stdin = read_from_stdin()?;

    let path = filename.unwrap_or_else(|| Path::new("-"));
    let source_file = SourceFileBuilder::new(path.to_str().unwrap_or("-"), stdin.as_str()).finish();

    let (mut messages, fixed) = if matches!(fix_mode, FixMode::Apply | FixMode::Diff) {
        if let Ok(FixerResult {
            result,
            transformed,
            fixed,
        }) = check_and_fix_file(
            rules,
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            &source_file,
            settings,
            ignore_allow_comments,
        ) {
            match fix_mode {
                FixMode::Apply => {
                    // Write the contents to stdout, regardless of whether any errors were fixed.
                    let out_file = &mut io::stdout().lock();
                    out_file.write_all(transformed.source_text().as_bytes())?;
                }
                // TODO: diff
                FixMode::Diff => {
                    warn_user_once_by_message!("Diff mode is not yet supported for stdin");
                }
                FixMode::Generate => {}
            }

            (result, fixed)
        } else {
            // Failed to fix, so just lint the original source
            let result = check_only_file(
                rules,
                path_rules,
                text_rules,
                ast_entrypoints,
                path,
                &source_file,
                settings,
                ignore_allow_comments,
            )?;

            // Write the input to stdout anyway.
            // Necessary in case the user is using stdin fix mode to overwrite the current
            // buffer in their editor of choice, e.g. in vim:
            // `:%!fortitude check --fix-only --silent --stdin-filename=%`
            if fix_mode.is_apply() {
                let out_file = &mut io::stdout().lock();
                out_file.write_all(source_file.source_text().as_bytes())?;
            }

            let fixed = FxHashMap::default();
            (result, fixed)
        }
    } else {
        let result = check_only_file(
            rules,
            path_rules,
            text_rules,
            ast_entrypoints,
            path,
            &source_file,
            settings,
            ignore_allow_comments,
        )?;
        let fixed = FxHashMap::default();
        (result, fixed)
    };

    let per_file_ignores = &settings.check.per_file_ignores;
    // Ignore based on per-file-ignores.
    // If the DiagnosticMessage is discarded, its fix will also be ignored.
    let per_file_ignores = if !messages.is_empty() && !per_file_ignores.is_empty() {
        fs::ignores_from_path(path, per_file_ignores)
    } else {
        vec![]
    };
    if !per_file_ignores.is_empty() {
        messages.retain(|message| {
            if let Some(rule) = message.rule() {
                !per_file_ignores.contains(&rule)
            } else {
                true
            }
        });
    }

    let diagnostics = Diagnostics {
        messages,
        fixed: FixMap::from_iter([(fs::relativize_path(path), fixed)]),
    };
    Ok(CheckResults::from_stdin(diagnostics))
}
