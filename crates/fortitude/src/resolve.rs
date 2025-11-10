// Adapted from ruff
// Copyright 2022-2025 Charles Marsh
// SPDX-License-Identifier: MIT

use std::path::Path;

use anyhow::{Result, bail};
use log::debug;

use fortitude_workspace::configuration::{
    Configuration, ConfigurationTransformer, find_settings_toml,
};
use fortitude_workspace::resolver::{
    ConfigurationOrigin, FortconfigDiscoveryStrategy, FortitudeConfig, resolve_root_settings,
};

use crate::cli::ConfigArguments;

/// Resolve the relevant settings strategy and defaults for the current
/// invocation.
pub fn resolve(
    config_arguments: &ConfigArguments,
    stdin_filename: Option<&Path>,
) -> Result<FortitudeConfig> {
    let Ok(cwd) = std::env::current_dir() else {
        bail!("Working directory does not exist")
    };

    // Second priority: the user specified a `fortitude.toml` file. Use that
    // `fortitude.toml` for _all_ configuration, and resolve paths relative to the
    // current working directory. (This matches ESLint's behavior.)
    if let Some(fortconfig) = config_arguments.config_file() {
        let settings = resolve_root_settings(
            fortconfig,
            config_arguments,
            ConfigurationOrigin::UserSpecified,
        )?;
        debug!(
            "Using user-specified configuration file at: {}",
            fortconfig.display()
        );
        return Ok(FortitudeConfig::new(
            FortconfigDiscoveryStrategy::Fixed,
            settings,
            Some(fortconfig.to_path_buf()),
        ));
    }

    // Third priority: find a `fortitude.toml` file in either an ancestor of
    // `stdin_filename` (if set) or the current working path all paths relative to
    // that directory. (With `Strategy::Hierarchical`, we'll end up finding
    // the "closest" `fortitude.toml` file for every Fortran file later on,
    // so these act as the "default" settings.)
    if let Some(fortconfig) = find_settings_toml(stdin_filename.unwrap_or(&cwd))? {
        debug!(
            "Using configuration file (via parent) at: {}",
            fortconfig.display()
        );
        let settings =
            resolve_root_settings(&fortconfig, config_arguments, ConfigurationOrigin::Ancestor)?;
        return Ok(FortitudeConfig::new(
            FortconfigDiscoveryStrategy::Hierarchical,
            settings,
            Some(fortconfig),
        ));
    }

    // Fallback: load Fortitude's default settings, and resolve all paths relative to the
    // current working directory. (With `Strategy::Hierarchical`, we'll end up the
    // "closest" `fortitude.toml` file for every Fortran file later on, so these act
    // as the "default" settings.)
    debug!("Using Fortitude default settings");
    let config = config_arguments.transform(Configuration::default());
    let settings = config.into_settings(&cwd)?;
    Ok(FortitudeConfig::new(
        FortconfigDiscoveryStrategy::Hierarchical,
        settings,
        None,
    ))
}
