//! This crate implements an internal CLI for developers of Fortitude.
//!
//! Within the fortitude repository you can run it with `cargo dev`.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod generate_all;
mod generate_cli_help;
mod generate_docs;
mod generate_options;
mod generate_rules_table;
mod parse;

const ROOT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant, clippy::enum_variant_names)]
enum Command {
    /// Run all code and documentation generation steps.
    GenerateAll(generate_all::Args),
    // /// Generate JSON schema for the TOML configuration file.
    // GenerateJSONSchema(generate_json_schema::Args),
    /// Generate a Markdown-compatible table of supported lint rules.
    GenerateRulesTable,
    /// Generate a Markdown-compatible listing of configuration options.
    GenerateOptions(generate_options::Args),
    /// Generate CLI help.
    GenerateCliHelp(generate_cli_help::Args),
    /// Generate Markdown docs.
    GenerateDocs(generate_docs::Args),
    /// Print the AST for a given Fortran file
    PrintAST(parse::Args),
}

fn main() -> Result<ExitCode> {
    let Args { command } = Args::parse();
    #[allow(clippy::print_stdout)]
    match command {
        Command::GenerateAll(args) => generate_all::main(&args)?,
        // Command::GenerateJSONSchema(args) => generate_json_schema::main(&args)?,
        Command::GenerateRulesTable => println!("{}", generate_rules_table::generate()),
        Command::GenerateOptions(args) => generate_options::main(&args)?,
        Command::GenerateCliHelp(args) => generate_cli_help::main(&args)?,
        Command::GenerateDocs(args) => generate_docs::main(&args)?,
        Command::PrintAST(args) => parse::main(&args)?,
    }
    Ok(ExitCode::SUCCESS)
}
