use std::path::Path;

use anyhow::Result;
use globset::{Candidate, GlobSet};
use path_absolutize::path_dedot;

use crate::configuration::{Configuration, ConfigurationTransformer, load_options};
use fortitude_linter::settings::Settings;

/// Return `true` if the given file should be ignored based on the exclusion
/// criteria.
#[inline]
pub fn match_exclusion<P: AsRef<Path>, R: AsRef<Path>>(
    file_path: P,
    file_basename: R,
    exclusion: &GlobSet,
) -> bool {
    match_candidate_exclusion(
        &Candidate::new(file_path.as_ref()),
        &Candidate::new(file_basename.as_ref()),
        exclusion,
    )
}

/// Return `true` if the given candidates should be ignored based on the exclusion
/// criteria.
pub fn match_candidate_exclusion(
    file_path: &Candidate,
    file_basename: &Candidate,
    exclusion: &GlobSet,
) -> bool {
    if exclusion.is_empty() {
        return false;
    }
    exclusion.is_match_candidate(file_path) || exclusion.is_match_candidate(file_basename)
}

/// Recursively resolve a [`Configuration`] from a `pyproject.toml` file at the
/// specified [`Path`].
// TODO(charlie): This whole system could do with some caching. Right now, if a
// configuration file extends another in the same path, we'll re-parse the same
// file at least twice (possibly more than twice, since we'll also parse it when
// resolving the "default" configuration).
pub fn resolve_configuration(
    config_path: &Path,
    transformer: &dyn ConfigurationTransformer,
    origin: ConfigurationOrigin,
) -> Result<Configuration> {
    let relativity = Relativity::from(origin);
    let project_root = relativity.resolve(config_path);

    let options = load_options(config_path)?;
    let configuration = Configuration::from_options(options, project_root);

    Ok(transformer.transform(configuration))
}

/// Extract the project root (scope) and [`Settings`] from a given
/// `fortitude.toml`.
fn resolve_scoped_settings<'a>(
    pyproject: &'a Path,
    transformer: &dyn ConfigurationTransformer,
    origin: ConfigurationOrigin,
) -> Result<(&'a Path, Settings)> {
    let relativity = Relativity::from(origin);

    let configuration = resolve_configuration(pyproject, transformer, origin)?;
    let project_root = relativity.resolve(pyproject);
    let settings = configuration.into_settings(project_root)?;
    Ok((project_root, settings))
}

/// Extract the [`Settings`] from a given `pyproject.toml` and process the
/// configuration with the given [`ConfigurationTransformer`].
pub fn resolve_root_settings(
    pyproject: &Path,
    transformer: &dyn ConfigurationTransformer,
    origin: ConfigurationOrigin,
) -> Result<Settings> {
    let (_project_root, settings) = resolve_scoped_settings(pyproject, transformer, origin)?;
    Ok(settings)
}

/// The strategy for resolving file paths in a `pyproject.toml`.
#[derive(Copy, Clone)]
pub enum Relativity {
    /// Resolve file paths relative to the current working directory.
    Cwd,
    /// Resolve file paths relative to the directory containing the
    /// `pyproject.toml`.
    Parent,
}

impl Relativity {
    pub fn resolve(self, path: &Path) -> &Path {
        match self {
            Relativity::Parent => path
                .parent()
                .expect("Expected pyproject.toml file to be in parent directory"),
            Relativity::Cwd => &path_dedot::CWD,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// How the configuration is provided.
pub enum ConfigurationOrigin {
    /// Origin is unknown to the caller
    Unknown,
    /// User specified path to specific configuration file
    UserSpecified,
    /// User-level configuration (e.g. in `~/.config/ruff/pyproject.toml`)
    UserSettings,
    /// In parent or higher ancestor directory of path
    Ancestor,
}

impl From<ConfigurationOrigin> for Relativity {
    fn from(value: ConfigurationOrigin) -> Self {
        match value {
            ConfigurationOrigin::Unknown => Self::Parent,
            ConfigurationOrigin::UserSpecified => Self::Cwd,
            ConfigurationOrigin::UserSettings => Self::Cwd,
            ConfigurationOrigin::Ancestor => Self::Parent,
        }
    }
}
