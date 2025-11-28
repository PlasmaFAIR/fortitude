// Adapted from ruff
// Copyright 2022-2025 Charles Marsh
// SPDX-License-Identifier: MIT

use std::cmp::Ordering;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use anyhow::{Context, Result};
use anyhow::{anyhow, bail};
use fortitude_linter::settings::FileResolverSettings;
use globset::{Candidate, GlobSet};
use ignore::{DirEntry, Error, ParallelVisitor, WalkBuilder, WalkState};
use itertools::Itertools;
use log::debug;
use matchit::{InsertError, Match, Router};
use path_absolutize::path_dedot;

use fortitude_linter::{fs, settings::Settings};
use path_slash::PathExt;
use rustc_hash::FxHashSet;

use crate::configuration::{Configuration, ConfigurationTransformer, load_options, settings_toml};

/// The configuration information from a `fortitude.toml` file.
#[derive(Debug)]
pub struct ConfigFile {
    /// The strategy used to discover the relevant `fortitude.toml` file for
    /// each Fortran file.
    pub strategy: ConfigFileDiscoveryStrategy,
    /// All settings from the `fortitude.toml` file.
    pub settings: Settings,
    /// Absolute path to the `fortitude.toml` file. This would be `None` when
    /// either using the default settings or the `--isolated` flag is set.
    pub path: Option<PathBuf>,
}

impl ConfigFile {
    pub fn new(
        strategy: ConfigFileDiscoveryStrategy,
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
/// Fortran file.
#[derive(Debug, Clone, Copy, is_macro::Is)]
pub enum ConfigFileDiscoveryStrategy {
    /// Use a fixed `fortitude.toml` file for all Fortran files (i.e., one
    /// provided on the command-line).
    Fixed,
    /// Use the closest `fortitude.toml` file in the filesystem hierarchy, or
    /// the default settings.
    Hierarchical,
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

#[derive(Debug)]
pub struct Resolver<'a> {
    fortitude_config: &'a ConfigFile,
    /// All [`Settings`] that have been added to the resolver.
    settings: Vec<Settings>,
    /// A router from path to index into the `settings` vector.
    router: Router<usize>,
}

impl<'a> Resolver<'a> {
    /// Create a new [`Resolver`] for the given [`ConfigFile`].
    pub fn new(fortitude_config: &'a ConfigFile) -> Self {
        Self {
            fortitude_config,
            settings: Vec::new(),
            router: Router::new(),
        }
    }

    /// Return the [`Settings`] from the [`ConfigFile`].
    #[inline]
    pub fn base_settings(&self) -> &Settings {
        &self.fortitude_config.settings
    }

    /// Return `true` if the [`Resolver`] is using a hierarchical discovery strategy.
    #[inline]
    pub fn is_hierarchical(&self) -> bool {
        self.fortitude_config.strategy.is_hierarchical()
    }

    /// Return `true` if the [`Resolver`] should force-exclude files passed directly to the CLI.
    #[inline]
    pub fn force_exclude(&self) -> bool {
        self.fortitude_config.settings.file_resolver.force_exclude
    }

    /// Return `true` if the [`Resolver`] should respect `.gitignore` files.
    #[inline]
    pub fn respect_gitignore(&self) -> bool {
        self.fortitude_config
            .settings
            .file_resolver
            .respect_gitignore
    }

    /// Add a resolved [`Settings`] under a given [`PathBuf`] scope.
    fn add(&mut self, path: &Path, settings: Settings) {
        self.settings.push(settings);

        // Normalize the path to use `/` separators and escape the '{' and '}' characters,
        // which matchit uses for routing parameters.
        let path = path.to_slash_lossy().replace('{', "{{").replace('}', "}}");

        match self
            .router
            .insert(format!("{path}/{{*filepath}}"), self.settings.len() - 1)
        {
            Ok(()) => {}
            Err(InsertError::Conflict { .. }) => {
                return;
            }
            Err(_) => unreachable!("file paths are escaped before being inserted in the router"),
        }

        // Insert a mapping that matches the directory itself (without a trailing slash).
        // Inserting should always succeed because conflicts are resolved above and the above insertion guarantees
        // that the path is correctly escaped.
        self.router.insert(path, self.settings.len() - 1).unwrap();
    }

    /// Return the appropriate [`Settings`] for a given [`Path`].
    pub fn resolve(&self, path: &Path) -> &Settings {
        match self.fortitude_config.strategy {
            ConfigFileDiscoveryStrategy::Fixed => &self.fortitude_config.settings,
            ConfigFileDiscoveryStrategy::Hierarchical => self
                .router
                .at(path.to_slash_lossy().as_ref())
                .map(|Match { value, .. }| &self.settings[*value])
                .unwrap_or(&self.fortitude_config.settings),
        }
    }

    /// Return an iterator over the resolved [`Settings`] in this [`Resolver`].
    pub fn settings(&self) -> impl Iterator<Item = &Settings> {
        std::iter::once(&self.fortitude_config.settings).chain(&self.settings)
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

/// Find all Fortran (`.f90`, `.F90` files, and so on) in a set of paths.
pub fn fortran_files_in_path<'a>(
    paths: &[PathBuf],
    fortitude_config: &'a ConfigFile,
    transformer: &(dyn ConfigurationTransformer + Sync),
) -> Result<(Vec<Result<ResolvedFile, ignore::Error>>, Resolver<'a>)> {
    // Normalize every path (e.g., convert from relative to absolute).
    let mut paths: Vec<PathBuf> = paths.iter().map(fs::normalize_path).unique().collect();

    // Search for `fortitude.toml` files in all parent directories.
    let mut resolver = Resolver::new(fortitude_config);
    let mut seen = FxHashSet::default();

    // Insert the path to the root configuration to avoid parsing the configuration a second time.
    if let Some(config_path) = &fortitude_config.path {
        seen.insert(config_path.parent().unwrap());
    }

    if resolver.is_hierarchical() {
        for path in &paths {
            for ancestor in path.ancestors() {
                if seen.insert(ancestor) {
                    if let Some(fortitude) = settings_toml(ancestor)? {
                        let (root, settings) = resolve_scoped_settings(
                            &fortitude,
                            transformer,
                            ConfigurationOrigin::Ancestor,
                        )?;
                        resolver.add(root, settings);
                        // We found the closest configuration.
                        break;
                    }
                } else {
                    // We already visited this ancestor, we can stop here.
                    break;
                }
            }
        }
    }

    // Check if the paths themselves are excluded.
    if resolver.force_exclude() {
        paths.retain(|path| !is_file_excluded(path, &resolver));
        if paths.is_empty() {
            return Ok((vec![], resolver));
        }
    }

    let (first_path, rest_paths) = paths
        .split_first()
        .ok_or_else(|| anyhow!("Expected at least one path to search for Fortran files"))?;
    // Create the `WalkBuilder`.
    let mut builder = WalkBuilder::new(first_path);
    for path in rest_paths {
        builder.add(path);
    }
    builder.standard_filters(resolver.respect_gitignore());
    builder.hidden(false);

    builder.threads(
        std::thread::available_parallelism()
            .map_or(1, std::num::NonZeroUsize::get)
            .min(12),
    );

    let walker = builder.build_parallel();

    // Run the `WalkParallel` to collect all Fortran files.
    let state = WalkFortranFilesState::new(resolver);
    let mut visitor = FortranFilesVisitorBuilder::new(transformer, &state);
    walker.visit(&mut visitor);

    state.finish()
}

type ResolvedFiles = Vec<Result<ResolvedFile, ignore::Error>>;

struct WalkFortranFilesState<'config> {
    is_hierarchical: bool,
    merged: std::sync::Mutex<(ResolvedFiles, Result<()>)>,
    resolver: RwLock<Resolver<'config>>,
}

impl<'config> WalkFortranFilesState<'config> {
    fn new(resolver: Resolver<'config>) -> Self {
        Self {
            is_hierarchical: resolver.is_hierarchical(),
            merged: std::sync::Mutex::new((Vec::new(), Ok(()))),
            resolver: RwLock::new(resolver),
        }
    }

    fn finish(self) -> Result<(Vec<Result<ResolvedFile, ignore::Error>>, Resolver<'config>)> {
        let (files, error) = self.merged.into_inner().unwrap();
        error?;

        Ok((files, self.resolver.into_inner().unwrap()))
    }
}

struct FortranFilesVisitorBuilder<'s, 'config> {
    state: &'s WalkFortranFilesState<'config>,
    transformer: &'s (dyn ConfigurationTransformer + Sync),
}

impl<'s, 'config> FortranFilesVisitorBuilder<'s, 'config> {
    fn new(
        transformer: &'s (dyn ConfigurationTransformer + Sync),
        state: &'s WalkFortranFilesState<'config>,
    ) -> Self {
        Self { state, transformer }
    }
}

struct FortranFilesVisitor<'s, 'config> {
    local_files: Vec<Result<ResolvedFile, ignore::Error>>,
    local_error: Result<()>,
    global: &'s WalkFortranFilesState<'config>,
    transformer: &'s (dyn ConfigurationTransformer + Sync),
}

impl<'config, 's> ignore::ParallelVisitorBuilder<'s> for FortranFilesVisitorBuilder<'s, 'config>
where
    'config: 's,
{
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(FortranFilesVisitor {
            local_files: vec![],
            local_error: Ok(()),
            global: self.state,
            transformer: self.transformer,
        })
    }
}

impl ParallelVisitor for FortranFilesVisitor<'_, '_> {
    fn visit(&mut self, result: std::result::Result<DirEntry, Error>) -> WalkState {
        // Respect our own exclusion behavior.
        if let Ok(entry) = &result {
            if entry.depth() > 0 {
                let path = entry.path();
                let resolver = self.global.resolver.read().unwrap();
                let settings = resolver.resolve(path);
                if let Some(file_name) = path.file_name() {
                    let file_path = Candidate::new(path);
                    let file_basename = Candidate::new(file_name);
                    if match_candidate_exclusion(
                        &file_path,
                        &file_basename,
                        &settings.file_resolver.exclude,
                    ) {
                        debug!("Ignored path via `exclude`: {path:?}");
                        return WalkState::Skip;
                    } else if match_candidate_exclusion(
                        &file_path,
                        &file_basename,
                        &settings.file_resolver.extend_exclude,
                    ) {
                        debug!("Ignored path via `extend-exclude`: {path:?}");
                        return WalkState::Skip;
                    }
                } else {
                    debug!("Ignored path due to error in parsing: {path:?}");
                    return WalkState::Skip;
                }
            }
        }

        // Search for the `fortitude.toml` file in this directory, before we visit any
        // of its contents.
        if self.global.is_hierarchical {
            if let Ok(entry) = &result {
                if entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_dir())
                {
                    match settings_toml(entry.path()) {
                        Ok(Some(fortitude)) => match resolve_scoped_settings(
                            &fortitude,
                            self.transformer,
                            ConfigurationOrigin::Ancestor,
                        ) {
                            Ok((root, settings)) => {
                                self.global.resolver.write().unwrap().add(root, settings);
                            }
                            Err(err) => {
                                self.local_error = Err(err);
                                return WalkState::Quit;
                            }
                        },
                        Ok(None) => {}
                        Err(err) => {
                            self.local_error = Err(err);
                            return WalkState::Quit;
                        }
                    }
                }
            }
        }

        match result {
            Ok(entry) => {
                // Ignore directories
                let resolved = if entry.file_type().is_none_or(|ft| ft.is_dir()) {
                    None
                } else if entry.depth() == 0 {
                    // Accept all files that are passed-in directly.
                    Some(ResolvedFile::Root(entry.into_path()))
                } else {
                    // Otherwise, check if the file is included.
                    let path = entry.path();
                    let resolver = self.global.resolver.read().unwrap();
                    let settings = resolver.resolve(path);
                    if settings.file_resolver.include.is_match(path) {
                        debug!("Included path via `include`: {path:?}");
                        Some(ResolvedFile::Nested(entry.into_path()))
                    }
                    // TODO(peter): restore when adding extend-include input option
                    //  else if settings.file_resolver.extend_include.is_match(path) {
                    //     debug!("Included path via `extend-include`: {path:?}");
                    //     Some(ResolvedFile::Nested(entry.into_path()))
                    // }
                    else {
                        None
                    }
                };

                if let Some(resolved) = resolved {
                    self.local_files.push(Ok(resolved));
                }
            }
            Err(err) => {
                self.local_files.push(Err(err));
            }
        }

        WalkState::Continue
    }
}

impl Drop for FortranFilesVisitor<'_, '_> {
    fn drop(&mut self) {
        let mut merged = self.global.merged.lock().unwrap();
        let (files, error) = &mut *merged;

        if files.is_empty() {
            *files = std::mem::take(&mut self.local_files);
        } else {
            files.append(&mut self.local_files);
        }

        let local_error = std::mem::replace(&mut self.local_error, Ok(()));
        if error.is_ok() {
            *error = local_error;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedFile {
    /// File explicitly passed to the CLI
    Root(PathBuf),
    /// File in a sub-directory
    Nested(PathBuf),
}

impl ResolvedFile {
    pub fn into_path(self) -> PathBuf {
        match self {
            ResolvedFile::Root(path) => path,
            ResolvedFile::Nested(path) => path,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            ResolvedFile::Root(root) => root.as_path(),
            ResolvedFile::Nested(root) => root.as_path(),
        }
    }

    pub fn file_name(&self) -> &OsStr {
        let path = self.path();
        path.file_name().unwrap_or(path.as_os_str())
    }

    pub fn is_root(&self) -> bool {
        matches!(self, ResolvedFile::Root(_))
    }
}

impl PartialOrd for ResolvedFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResolvedFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path().cmp(other.path())
    }
}

/// Return `true` if the Fortran file at [`Path`] is _not_ excluded.
pub fn fortran_file_at_path(
    path: &Path,
    resolver: &mut Resolver,
    transformer: &dyn ConfigurationTransformer,
) -> Result<bool> {
    // Normalize the path (e.g., convert from relative to absolute).
    let path = fs::normalize_path(path);

    // Search for `fortitude.toml` files in all parent directories.
    if resolver.is_hierarchical() {
        for ancestor in path.ancestors() {
            if let Some(fortitude) = settings_toml(ancestor)? {
                let (root, settings) =
                    resolve_scoped_settings(&fortitude, transformer, ConfigurationOrigin::Unknown)?;
                resolver.add(root, settings);
                break;
            }
        }
    }

    // Check exclusions.
    Ok(!is_file_excluded(&path, resolver))
}

/// Return `true` if the given top-level [`Path`] should be excluded.
fn is_file_excluded(path: &Path, resolver: &Resolver) -> bool {
    // TODO(charlie): Respect gitignore.
    for path in path.ancestors() {
        let settings = resolver.resolve(path);
        if let Some(file_name) = path.file_name() {
            let file_path = Candidate::new(path);
            let file_basename = Candidate::new(file_name);
            if match_candidate_exclusion(
                &file_path,
                &file_basename,
                &settings.file_resolver.exclude,
            ) {
                debug!("Ignored path via `exclude`: {path:?}");
                return true;
            } else if match_candidate_exclusion(
                &file_path,
                &file_basename,
                &settings.file_resolver.extend_exclude,
            ) {
                debug!("Ignored path via `extend-exclude`: {path:?}");
                return true;
            }
        } else {
            break;
        }
        if path == settings.file_resolver.project_root {
            // Bail out; we'd end up past the project root on the next iteration
            // (excludes etc. are thus "rooted" to the project).
            break;
        }
    }
    false
}

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

#[derive(Debug, Copy, Clone)]
pub enum ExclusionKind {
    /// The exclusion came from the `exclude` setting.
    Exclude,
    /// The exclusion came from the `extend-exclude` setting.
    ExtendExclude,
    /// The exclusion came from the `lint.exclude` setting.
    LintExclude,
    /// The exclusion came from the `lint.extend-exclude` setting.
    FormatExclude,
}

impl std::fmt::Display for ExclusionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExclusionKind::Exclude => write!(f, "exclude"),
            ExclusionKind::ExtendExclude => write!(f, "extend-exclude"),
            ExclusionKind::LintExclude => write!(f, "lint.exclude"),
            ExclusionKind::FormatExclude => write!(f, "lint.extend-exclude"),
        }
    }
}

/// Return the [`ExclusionKind`] for a given [`Path`], if the path or any of its ancestors match
/// any of the exclusion criteria.
pub fn match_any_exclusion(
    path: &Path,
    resolver_settings: &FileResolverSettings,
    lint_exclude: Option<&GlobSet>,
    format_exclude: Option<&GlobSet>,
) -> Option<ExclusionKind> {
    for path in path.ancestors() {
        if let Some(basename) = path.file_name() {
            let path = Candidate::new(path);
            let basename = Candidate::new(basename);
            if match_candidate_exclusion(&path, &basename, &resolver_settings.exclude) {
                return Some(ExclusionKind::Exclude);
            }
            if match_candidate_exclusion(&path, &basename, &resolver_settings.extend_exclude) {
                return Some(ExclusionKind::ExtendExclude);
            }
            // These two bits are currently useless until we get the format
            // command in, and want separate exclude options
            if let Some(lint_exclude) = lint_exclude {
                if match_candidate_exclusion(&path, &basename, lint_exclude) {
                    return Some(ExclusionKind::LintExclude);
                }
            }
            if let Some(format_exclude) = format_exclude {
                if match_candidate_exclusion(&path, &basename, format_exclude) {
                    return Some(ExclusionKind::FormatExclude);
                }
            }
        }
        if path == resolver_settings.project_root {
            // Bail out; we'd end up past the project root on the next iteration
            // (excludes etc. are thus "rooted" to the project).
            break;
        }
    }
    None
}

#[derive(Debug, Copy, Clone)]
pub enum InclusionKind {
    /// The inclusion came from the `include` setting.
    Include,
    /// The inclusion came from the `extend-include` setting.
    ExtendInclude,
}

impl std::fmt::Display for InclusionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InclusionKind::Include => write!(f, "include"),
            InclusionKind::ExtendInclude => write!(f, "extend-include"),
        }
    }
}

/// Return the [`InclusionKind`] for a given [`Path`], if the path match any of the inclusion
/// criteria.
pub fn match_any_inclusion(
    path: &Path,
    resolver_settings: &FileResolverSettings,
) -> Option<InclusionKind> {
    if resolver_settings.include.is_match(path) {
        Some(InclusionKind::Include)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{File, create_dir};
    use std::path::Path;

    use anyhow::Result;
    use fortitude_linter::fs::{FilePattern, GlobPath};
    use globset::GlobSet;
    use itertools::Itertools;
    use path_absolutize::Absolutize;
    use tempfile::TempDir;

    use crate::configuration::{Configuration, find_settings_toml};
    use crate::resolver::{
        ConfigFile, ConfigFileDiscoveryStrategy, ConfigurationOrigin, ConfigurationTransformer,
        ResolvedFile, Resolver, fortran_files_in_path, is_file_excluded, match_exclusion,
        resolve_root_settings,
    };
    use crate::tests::test_resource_path;
    use fortitude_linter::settings::Settings;

    struct NoOpTransformer;

    impl ConfigurationTransformer for NoOpTransformer {
        fn transform(&self, config: Configuration) -> Configuration {
            config
        }
    }

    #[test]
    fn rooted_exclusion() -> Result<()> {
        let package_root = test_resource_path("package");
        let fortitude_config = ConfigFile::new(
            ConfigFileDiscoveryStrategy::Hierarchical,
            resolve_root_settings(
                &find_settings_toml(&package_root)?.unwrap(),
                &NoOpTransformer,
                ConfigurationOrigin::Ancestor,
            )?,
            None,
        );
        let resolver = Resolver::new(&fortitude_config);
        // src/app.f90 should not be excluded even if it lives in a hierarchy that should
        // be excluded by virtue of the fortitude.toml having `resources/*` in
        // it.
        assert!(!is_file_excluded(
            &package_root.join("src/app.f90"),
            &resolver,
        ));
        // However, resources/ignored.f90 should be ignored, since that `resources` is
        // beneath the package root.
        assert!(is_file_excluded(
            &package_root.join("resources/ignored.f90"),
            &resolver,
        ));
        Ok(())
    }

    #[test]
    fn find_fortran_files() -> Result<()> {
        // Initialize the filesystem:
        //   root
        //   ├── file1.f90
        //   ├── dir1.f90
        //   │   └── file2.f90
        //   └── dir2.f90
        let tmp_dir = TempDir::new()?;
        let root = tmp_dir.path();
        let file1 = root.join("file1.f90");
        let dir1 = root.join("dir1.f90");
        let file2 = dir1.join("file2.f90");
        let dir2 = root.join("dir2.f90");
        File::create(&file1)?;
        create_dir(dir1)?;
        File::create(&file2)?;
        create_dir(dir2)?;

        let (paths, _) = fortran_files_in_path(
            &[root.to_path_buf()],
            &ConfigFile::new(
                ConfigFileDiscoveryStrategy::Fixed,
                Settings::default(),
                None,
            ),
            &NoOpTransformer,
        )?;
        let paths = paths
            .into_iter()
            .flatten()
            .map(ResolvedFile::into_path)
            .sorted()
            .collect::<Vec<_>>();
        assert_eq!(paths, [file2, file1]);

        Ok(())
    }

    fn make_exclusion(file_pattern: FilePattern) -> GlobSet {
        let mut builder = globset::GlobSetBuilder::new();
        file_pattern.add_to(&mut builder).unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn exclusions() {
        let project_root = Path::new("/tmp/");

        let path = Path::new("foo").absolutize_from(project_root).unwrap();
        let exclude =
            FilePattern::User("foo".to_string(), GlobPath::normalize("foo", project_root));
        let file_path = &path;
        let file_basename = path.file_name().unwrap();
        assert!(match_exclusion(
            file_path,
            file_basename,
            &make_exclusion(exclude),
        ));

        let path = Path::new("foo/bar").absolutize_from(project_root).unwrap();
        let exclude =
            FilePattern::User("bar".to_string(), GlobPath::normalize("bar", project_root));
        let file_path = &path;
        let file_basename = path.file_name().unwrap();
        assert!(match_exclusion(
            file_path,
            file_basename,
            &make_exclusion(exclude),
        ));

        let path = Path::new("foo/bar/baz.f90")
            .absolutize_from(project_root)
            .unwrap();
        let exclude = FilePattern::User(
            "baz.f90".to_string(),
            GlobPath::normalize("baz.f90", project_root),
        );
        let file_path = &path;
        let file_basename = path.file_name().unwrap();
        assert!(match_exclusion(
            file_path,
            file_basename,
            &make_exclusion(exclude),
        ));

        let path = Path::new("foo/bar").absolutize_from(project_root).unwrap();
        let exclude = FilePattern::User(
            "foo/bar".to_string(),
            GlobPath::normalize("foo/bar", project_root),
        );
        let file_path = &path;
        let file_basename = path.file_name().unwrap();
        assert!(match_exclusion(
            file_path,
            file_basename,
            &make_exclusion(exclude),
        ));

        let path = Path::new("foo/bar/baz.f90")
            .absolutize_from(project_root)
            .unwrap();
        let exclude = FilePattern::User(
            "foo/bar/baz.f90".to_string(),
            GlobPath::normalize("foo/bar/baz.f90", project_root),
        );
        let file_path = &path;
        let file_basename = path.file_name().unwrap();
        assert!(match_exclusion(
            file_path,
            file_basename,
            &make_exclusion(exclude),
        ));

        let path = Path::new("foo/bar/baz.f90")
            .absolutize_from(project_root)
            .unwrap();
        let exclude = FilePattern::User(
            "foo/bar/*.f90".to_string(),
            GlobPath::normalize("foo/bar/*.f90", project_root),
        );
        let file_path = &path;
        let file_basename = path.file_name().unwrap();
        assert!(match_exclusion(
            file_path,
            file_basename,
            &make_exclusion(exclude),
        ));

        let path = Path::new("foo/bar/baz.f90")
            .absolutize_from(project_root)
            .unwrap();
        let exclude =
            FilePattern::User("baz".to_string(), GlobPath::normalize("baz", project_root));
        let file_path = &path;
        let file_basename = path.file_name().unwrap();
        assert!(!match_exclusion(
            file_path,
            file_basename,
            &make_exclusion(exclude),
        ));
    }
}
