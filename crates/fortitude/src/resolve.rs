// Adapted from ruff
// Copyright 2022-2025 Charles Marsh
// SPDX-License-Identifier: MIT

use std::path::Path;

use anyhow::{Result, bail};
use log::debug;

use fortitude_workspace::configuration::{
    Configuration, ConfigurationTransformer, find_settings_toml, find_user_settings_toml,
};
use fortitude_workspace::resolver::{
    ConfigFile, ConfigFileDiscoveryStrategy, ConfigurationOrigin, resolve_root_settings,
};

use crate::cli::ConfigArguments;

/// Resolve the relevant settings strategy and defaults for the current
/// invocation.
pub fn resolve(
    config_arguments: &ConfigArguments,
    stdin_filename: Option<&Path>,
) -> Result<ConfigFile> {
    let Ok(cwd) = std::env::current_dir() else {
        bail!("Working directory does not exist")
    };

    // First priority: if we're running in isolated mode, use the default settings.
    if config_arguments.isolated {
        let config = config_arguments.transform(Configuration::default());
        let settings = config.into_settings(&cwd)?;
        debug!("Isolated mode, not reading any TOML file");
        return Ok(ConfigFile::new(
            ConfigFileDiscoveryStrategy::Fixed,
            settings,
            None,
        ));
    }
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
        return Ok(ConfigFile::new(
            ConfigFileDiscoveryStrategy::Fixed,
            settings,
            Some(fortconfig.to_path_buf()),
        ));
    }

    // Third priority: find a `fortitude.toml` file in either an ancestor of
    // `stdin_filename` (if set) or the current working path, with all paths
    // resolved relative to that directory. (With `Strategy::Hierarchical`,
    // we'll end up finding the "closest" `fortitude.toml` file for every
    // Fortran file later on, so these act as the "default" settings.)
    if let Some(fortconfig) = find_settings_toml(stdin_filename.unwrap_or(&cwd))? {
        debug!(
            "Using configuration file (via parent) at: {}",
            fortconfig.display()
        );
        let settings =
            resolve_root_settings(&fortconfig, config_arguments, ConfigurationOrigin::Ancestor)?;
        return Ok(ConfigFile::new(
            ConfigFileDiscoveryStrategy::Hierarchical,
            settings,
            Some(fortconfig),
        ));
    }

    // Fourth priority: find a user-specific `fortitude.toml`, but resolve all paths
    // relative the current working directory. (With `Strategy::Hierarchical`, we'll
    // end up the "closest" `fortitude.toml` file for every Fortran file later on, so
    // these act as the "default" settings.)
    if let Some(user_config) = find_user_settings_toml() {
        debug!(
            "Using configuration file (via cwd) at: {}",
            user_config.display()
        );
        let settings = resolve_root_settings(
            &user_config,
            config_arguments,
            ConfigurationOrigin::UserSettings,
        )?;
        return Ok(ConfigFile::new(
            ConfigFileDiscoveryStrategy::Hierarchical,
            settings,
            Some(user_config),
        ));
    }

    // Fallback: load Fortitude's default settings, and resolve all paths relative to the
    // current working directory. (With `Strategy::Hierarchical`, we'll end up the
    // "closest" `fortitude.toml` file for every Fortran file later on, so these act
    // as the "default" settings.)
    debug!("Using Fortitude default settings");
    let config = config_arguments.transform(Configuration::default());
    let settings = config.into_settings(&cwd)?;
    Ok(ConfigFile::new(
        ConfigFileDiscoveryStrategy::Hierarchical,
        settings,
        None,
    ))
}
