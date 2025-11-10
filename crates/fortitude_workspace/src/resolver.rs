// Adapted from ruff
// Copyright 2022-2025 Charles Marsh
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

use anyhow::bail;
use anyhow::{Context, Result};
use itertools::Itertools;
use path_absolutize::path_dedot;

use fortitude_linter::{fs, settings::Settings};

use crate::configuration::{Configuration, ConfigurationTransformer, load_options};

/// The configuration information from a `fortitude.toml` file.
#[derive(Debug)]
pub struct FortitudeConfig {
    /// The strategy used to discover the relevant `fortitude.toml` file for
    /// each Python file.
    pub strategy: FortconfigDiscoveryStrategy,
    /// All settings from the `fortitude.toml` file.
    pub settings: Settings,
    /// Absolute path to the `fortitude.toml` file. This would be `None` when
    /// either using the default settings or the `--isolated` flag is set.
    pub path: Option<PathBuf>,
}

impl FortitudeConfig {
    pub fn new(
        strategy: FortconfigDiscoveryStrategy,
        settings: Settings,
        path: Option<PathBuf>,
    ) -> Self {
        Self {
            strategy,
            settings,
            path: path.map(fs::normalize_path),
        }
    }
}

/// The strategy used to discover the relevant `fortitude.toml` file for each
/// Python file.
#[derive(Debug, Copy, Clone)]
pub enum FortconfigDiscoveryStrategy {
    /// Use a fixed `fortitude.toml` file for all Python files (i.e., one
    /// provided on the command-line).
    Fixed,
    /// Use the closest `fortitude.toml` file in the filesystem hierarchy, or
    /// the default settings.
    Hierarchical,
}

impl FortconfigDiscoveryStrategy {
    #[inline]
    pub const fn is_fixed(self) -> bool {
        matches!(self, FortconfigDiscoveryStrategy::Fixed)
    }

    #[inline]
    pub const fn is_hierarchical(self) -> bool {
        matches!(self, FortconfigDiscoveryStrategy::Hierarchical)
    }
}

/// The strategy for resolving file paths in a `fortitude.toml`.
#[derive(Copy, Clone)]
pub enum Relativity {
    /// Resolve file paths relative to the current working directory.
    Cwd,
    /// Resolve file paths relative to the directory containing the
    /// `fortitude.toml`.
    Parent,
}

impl Relativity {
    pub fn resolve(self, path: &Path) -> &Path {
        match self {
            Relativity::Parent => path
                .parent()
                .expect("Expected fortitude.toml file to be in parent directory"),
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
    /// User-level configuration (e.g. in `~/.config/fortitude/fortitude.toml`)
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

/// Recursively resolve a [`Configuration`] from a `fortitude.toml` file at the
/// specified [`Path`].
// TODO(charlie): This whole system could do with some caching. Right now, if a
// configuration file extends another in the same path, we'll re-parse the same
// file at least twice (possibly more than twice, since we'll also parse it when
// resolving the "default" configuration).
// TODO(peter): Currently very boring and overcomplicated until we add
// `Configuration::extend`
pub fn resolve_configuration(
    fortconfig: &Path,
    transformer: &dyn ConfigurationTransformer,
    origin: ConfigurationOrigin,
) -> Result<Configuration> {
    let relativity = Relativity::from(origin);
    let mut configurations = indexmap::IndexMap::new();
    let mut next = Some(fs::normalize_path(fortconfig));
    while let Some(path) = next {
        if configurations.contains_key(&path) {
            bail!(format!(
                "Circular configuration detected: {chain}",
                chain = configurations
                    .keys()
                    .chain([&path])
                    .map(|p| format!("`{}`", p.display()))
                    .join(" extends "),
            ));
        }

        // Resolve the current path.
        let options = load_options(&path).with_context(|| {
            if configurations.is_empty() {
                format!(
                    "Failed to load configuration `{path}`",
                    path = path.display()
                )
            } else {
                let chain = configurations
                    .keys()
                    .chain([&path])
                    .map(|p| format!("`{}`", p.display()))
                    .join(" extends ");
                format!(
                    "Failed to load extended configuration `{path}` ({chain})",
                    path = path.display()
                )
            }
        })?;

        let project_root = relativity.resolve(&path);
        let configuration = Configuration::from_options(options, project_root);

        // If extending, continue to collect.
        // TODO(peter): add Configuration::extend
        next = None;

        // Keep track of (1) the paths we've already resolved (to avoid cycles), and (2)
        // the base configuration for every path.
        configurations.insert(path, configuration);
    }

    // Merge the configurations, in order.
    let mut configurations = configurations.into_values();
    let configuration = configurations.next().unwrap();
    Ok(transformer.transform(configuration))
}

/// Extract the project root (scope) and [`Settings`] from a given
/// `fortitude.toml`.
fn resolve_scoped_settings<'a>(
    fortconfig: &'a Path,
    transformer: &dyn ConfigurationTransformer,
    origin: ConfigurationOrigin,
) -> Result<(&'a Path, Settings)> {
    let relativity = Relativity::from(origin);

    let configuration = resolve_configuration(fortconfig, transformer, origin)?;
    let project_root = relativity.resolve(fortconfig);
    let settings = configuration.into_settings(project_root)?;
    Ok((project_root, settings))
}

/// Extract the [`Settings`] from a given `fortitude.toml` and process the
/// configuration with the given [`ConfigurationTransformer`].
pub fn resolve_root_settings(
    fortconfig: &Path,
    transformer: &dyn ConfigurationTransformer,
    origin: ConfigurationOrigin,
) -> Result<Settings> {
    let (_project_root, settings) = resolve_scoped_settings(fortconfig, transformer, origin)?;
    Ok(settings)
}
