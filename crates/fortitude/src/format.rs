use std::{io::BufWriter, process::ExitCode};

use fortitude_linter::{
    format::{create_formatter, format_file},
    fs::{get_files, FilePatternSet, FORTRAN_EXTS},
    settings::{FileResolverSettings, PreviewMode},
};
use fortitude_workspace::configuration::{self, resolve_bool_arg};
use itertools::Itertools;

use crate::cli::FormatArgs;

use anyhow::Result;

/// Run the formatter over a whole project
///
/// TODO: Proper options like ``check``
pub fn format(args: FormatArgs) -> Result<ExitCode> {
    if !args.i_understand_the_risks.unwrap_or_default() {
        println!(
            "The format command is still in development and may break your code (although it
_probably_ won't), and future changes may give different results. To use the
format command, you must set `--i-understand-the-risks` on the command line to acknowledge the
risks."
        );
        return Ok(ExitCode::FAILURE);
    }

    let preview = resolve_bool_arg(args.preview, args.no_preview)
        .map(PreviewMode::from)
        .unwrap_or_default();

    if preview == PreviewMode::Disabled {
        println!("Format mode is currently in preview; nothing to do");
        return Ok(ExitCode::SUCCESS);
    }

    let files = args.files.unwrap_or_default();
    let file_extensions = args
        .file_extensions
        .unwrap_or(FORTRAN_EXTS.iter().map(|ext| ext.to_string()).collect_vec());
    let project_root = configuration::project_root(path_absolutize::path_dedot::CWD.as_path())?;

    let language = create_formatter();

    let file_resolver = FileResolverSettings {
        excludes: FilePatternSet::default(),
        force_exclude: true,
        files,
        file_extensions,
        respect_gitignore: true,
        project_root,
    };

    for file in get_files(&file_resolver, false)? {
        let output = std::io::stdout();
        let mut buf_output = BufWriter::new(output);

        match format_file(file, &language, &mut buf_output) {
            Ok(_) => continue,
            Err(err) => {
                println!("Formatter error: {err}");
                return Ok(ExitCode::FAILURE);
            }
        };
    }

    Ok(ExitCode::SUCCESS)
}
