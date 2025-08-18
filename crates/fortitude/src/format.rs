use std::{io::BufWriter, process::ExitCode};

use fortitude_formatter::{create_formatter, format_file};
use fortitude_linter::{fs::get_files, settings::PreviewMode};
use fortitude_workspace::configuration::{
    self, parse_config_file, Configuration, ConfigurationTransformer,
};

use crate::cli::{FormatCommand, GlobalConfigArgs};

use anyhow::Result;

/// Run the formatter over a whole project
///
/// TODO: Proper options like ``check``
pub fn format(args: FormatCommand, global_options: &GlobalConfigArgs) -> Result<ExitCode> {
    let (cli, config_arguments) = args.partition()?;

    if !cli.i_understand_the_risks {
        println!(
            "The format command is still in development and may break your code (although it
_probably_ won't), and future changes may give different results. To use the
format command, you must set `--i-understand-the-risks` on the command line to acknowledge the
risks."
        );
        return Ok(ExitCode::FAILURE);
    }

    let project_root = configuration::project_root(path_absolutize::path_dedot::CWD.as_path())?;
    let file_configuration = Configuration::from_options(
        parse_config_file(&global_options.config_file)?,
        &project_root,
    );

    // Now, we can override settings from the config file with options
    // from the CLI
    let config = config_arguments.transform(file_configuration);
    let settings = config.into_settings(&project_root)?;

    if settings.format.preview == PreviewMode::Disabled {
        println!("Format mode is currently in preview; nothing to do");
        return Ok(ExitCode::SUCCESS);
    }

    let language = create_formatter();

    for file in get_files(&settings.file_resolver, false)? {
        let output = std::io::stdout();
        let mut buf_output = BufWriter::new(output);

        match format_file(file, &language, &settings.format, &mut buf_output) {
            Ok(_) => continue,
            Err(err) => {
                println!("Formatter error: {err}");
                return Ok(ExitCode::FAILURE);
            }
        };
    }

    Ok(ExitCode::SUCCESS)
}
