use std::{env, path::PathBuf};

use anyhow::Result;
use tree_sitter::{ffi, Parser};
use tree_sitter_cli::{
    parse::{parse_file_at_path, ParseFileOptions, ParseOutput, ParseStats, ParseTheme},
    util,
};

#[derive(clap::Args)]
pub(crate) struct Args {
    /// The Fortran file to parse
    pub path: PathBuf,
    /// Output the parse data in a pretty-printed CST format
    #[arg(long = "cst", short = 'c')]
    pub output_cst: bool,
    /// Omit ranges in the output
    #[arg(long)]
    pub no_ranges: bool,
}

pub(crate) fn main(args: &Args) -> Result<()> {
    let colour = env::var("NO_COLOR").map_or(true, |v| v != "1");
    let output = if args.output_cst {
        ParseOutput::Cst
    } else {
        ParseOutput::Normal
    };
    let parse_theme = if colour {
        ParseTheme::default()
    } else {
        ParseTheme::empty()
    };

    let mut parser = Parser::new();

    let mut stats = ParseStats::default();
    let edits: Vec<String> = vec![];
    let cancellation_flag = util::cancel_on_signal();

    let mut options = ParseFileOptions {
        edits: &edits
            .iter()
            .map(std::string::String::as_str)
            .collect::<Vec<&str>>(),
        output,
        print_time: false,
        timeout: 0,
        stats: &mut stats,
        debug: tree_sitter_cli::parse::ParseDebugType::Quiet,
        debug_graph: false,
        cancellation_flag: Some(&cancellation_flag),
        encoding: Some(ffi::TSInputEncodingUTF8),
        open_log: false,
        no_ranges: args.no_ranges,
        parse_theme: &parse_theme,
    };

    let max_path_length = args.path.to_string_lossy().chars().count();

    parse_file_at_path(
        &mut parser,
        &tree_sitter_fortran::LANGUAGE.into(),
        &args.path,
        &args.path.display().to_string(),
        max_path_length,
        &mut options,
    )?;

    Ok(())
}
