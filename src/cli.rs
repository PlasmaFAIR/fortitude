use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_config::ClapConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use toml::Table;

/// Default extensions to check
const FORTRAN_EXTS: &[&str] = &[
    "f90", "F90", "f95", "F95", "f03", "F03", "f08", "F08", "f18", "F18", "f23", "F23",
];

#[derive(Debug, Parser, ClapConfig)]
#[command(version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: SubCommands,

    /// Path to a TOML configuration file
    #[arg(long)]
    pub config_file: Option<PathBuf>,
}

#[derive(Debug, Subcommand, ClapConfig, Clone, PartialEq)]
pub enum SubCommands {
    Check(CheckArgs),
    Explain(ExplainArgs),
}

/// Get descriptions, rationales, and solutions for each rule.
#[derive(Debug, clap::Parser, ClapConfig, Clone, PartialEq)]
pub struct ExplainArgs {
    /// List of rules to explain. If omitted, explains all rules.
    #[arg()]
    pub rules: Vec<String>,
}

/// Perform static analysis on files and report issues.
#[derive(Debug, clap::Parser, Deserialize, Serialize, ClapConfig, Clone, PartialEq)]
pub struct CheckArgs {
    /// List of files to analyze
    #[arg(default_value = ".")]
    pub files: Vec<PathBuf>,
    /// Comma-separated list of extra rules to include.
    #[arg(long, value_delimiter = ',')]
    pub include: Vec<String>,
    /// Comma-separated list of rules to ignore.
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,
    /// Comma-separated list of the only rules you wish to use.
    #[arg(long, value_delimiter=',', conflicts_with_all=["include", "ignore"])]
    pub select: Vec<String>,
    /// Set the maximum allowable line length.
    #[arg(long, default_value = "100")]
    pub line_length: usize,
    /// File extensions to check
    #[arg(long, value_delimiter = ',', default_values = FORTRAN_EXTS)]
    pub file_extensions: Vec<String>,
}

// These are just helper structs to let us quickly work out if there's
// a fortitude section in an fpm.toml file
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
struct Fpm {
    extra: Option<Extra>,
}

#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
struct Extra {
    fortitude: Option<EmptyConfig>,
}

#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
struct EmptyConfig {}

// Adapted from ruff
fn parse_fpm_toml<P: AsRef<Path>>(path: P) -> Result<Fpm> {
    let contents = std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read {}", path.as_ref().display()))?;
    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse {}", path.as_ref().display()))
}

pub fn fortitude_enabled<P: AsRef<Path>>(path: P) -> Result<bool> {
    let fpm = parse_fpm_toml(path)?;
    Ok(fpm.extra.and_then(|extra| extra.fortitude).is_some())
}

/// Return the path to the `fpm.toml` or `fortitude.toml` file in a given
/// directory. Adapated from ruff
pub fn settings_toml<P: AsRef<Path>>(path: P) -> Result<Option<PathBuf>> {
    // Check for `.fortitude.toml`.
    let fortitude_toml = path.as_ref().join(".fortitude.toml");
    if fortitude_toml.is_file() {
        return Ok(Some(fortitude_toml));
    }

    // Check for `fortitude.toml`.
    let fortitude_toml = path.as_ref().join("fortitude.toml");
    if fortitude_toml.is_file() {
        return Ok(Some(fortitude_toml));
    }

    // Check for `fpm.toml`.
    let fpm_toml = path.as_ref().join("fpm.toml");
    if fpm_toml.is_file() && fortitude_enabled(&fpm_toml)? {
        return Ok(Some(fpm_toml));
    }

    Ok(None)
}

/// Find the path to the `fpm.toml` or `fortitude.toml` file, if such a file
/// exists. Adapated from ruff
pub fn find_settings_toml<P: AsRef<Path>>(path: P) -> Result<Option<PathBuf>> {
    for directory in path.as_ref().ancestors() {
        if let Some(settings) = settings_toml(directory)? {
            return Ok(Some(settings));
        }
    }
    Ok(None)
}

fn from_clap_config_subsection<P: AsRef<Path>>(path: P) -> Result<Cli> {
    let matches = <Cli as CommandFactory>::command().get_matches();

    let config_str = if path.as_ref().ends_with("fpm.toml") {
        let config = std::fs::read_to_string(path)?.parse::<Table>()?;

        // Unwrap should be ok here because we've already checked this
        // file has these tables
        let extra = &config["extra"].as_table().unwrap();
        let fortitude = &extra["fortitude"].as_table().unwrap();
        fortitude.to_string()
    } else {
        std::fs::read_to_string(path)?
    };

    let config: CliConfig = toml::from_str(&config_str)?;

    Ok(Cli::from_merged(matches, Some(config)))
}

pub fn parse_args() -> Result<Cli> {
    let args = Cli::parse();

    if args.config_file.is_some() {
        return from_clap_config_subsection(args.config_file.unwrap());
    }

    if let Some(toml_file) = find_settings_toml(".")? {
        from_clap_config_subsection(toml_file)
    } else {
        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::{Context, Result};
    use tempfile::TempDir;
    use textwrap::dedent;

    use super::*;

    #[test]
    fn find_and_check_fpm_toml() -> Result<()> {
        let tempdir = TempDir::new()?;
        let fpm_toml = tempdir.path().join("fpm.toml");
        fs::write(
            fpm_toml,
            dedent(
                r#"
                some-stuff = 1
                other-things = "hello"

                [extra.fortitude.check]
                ignore = ["T001"]
                "#,
            ),
        )?;

        let fpm = find_settings_toml(tempdir.path())?.context("Failed to find fpm.toml")?;
        let enabled = fortitude_enabled(fpm)?;
        assert!(enabled);

        Ok(())
    }
}
