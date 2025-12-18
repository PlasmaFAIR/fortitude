use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::{path::Path, sync::Arc};

use anyhow::anyhow;
use fortitude_settings::FortitudeSettings;
use lsp_types::{FileEvent, Url};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::edit::LanguageId;
use crate::session::options::Combine;
use crate::workspace::{Workspace, Workspaces};
use crate::{
    PositionEncoding, TextDocument,
    edit::{DocumentKey, DocumentVersion},
};

use super::Client;
use super::settings::{ClientSettings, GlobalClientSettings};

mod fortitude_settings;

/// Stores and tracks all open documents in a session, along with their associated settings.
#[derive(Default)]
pub(crate) struct Index {
    /// Maps all document file URLs to the associated document controller
    documents: FxHashMap<Url, DocumentController>,

    /// Maps a workspace folder root to its settings.
    settings: WorkspaceSettingsIndex,
}

/// Settings associated with a workspace.
struct WorkspaceSettings {
    client_settings: Arc<ClientSettings>,
    fortitude_settings: fortitude_settings::FortitudeSettingsIndex,
}

/// A mutable handler to an underlying document.
#[derive(Debug)]
enum DocumentController {
    Text(Arc<TextDocument>),
}

/// A read-only query to an open document.
/// This query can 'select' a text document.
/// It also includes document settings.
#[derive(Clone)]
pub enum DocumentQuery {
    Text {
        file_url: Url,
        document: Arc<TextDocument>,
        settings: Arc<FortitudeSettings>,
    },
}

impl Index {
    pub(super) fn new(
        workspaces: &Workspaces,
        global: &GlobalClientSettings,
        client: &Client,
    ) -> crate::Result<Self> {
        let mut settings = WorkspaceSettingsIndex::default();
        for workspace in &**workspaces {
            settings.register_workspace(workspace, global, client)?;
        }

        Ok(Self {
            documents: FxHashMap::default(),
            settings,
        })
    }

    pub(super) fn text_document_urls(&self) -> impl Iterator<Item = &Url> + '_ {
        self.documents
            .iter()
            .filter(|(_, doc)| doc.as_text().is_some())
            .map(|(url, _)| url)
    }

    pub(super) fn update_text_document(
        &mut self,
        key: &DocumentKey,
        content_changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
        new_version: DocumentVersion,
        encoding: PositionEncoding,
    ) -> crate::Result<()> {
        let controller = self.document_controller_for_key(key)?;
        let Some(document) = controller.as_text_mut() else {
            anyhow::bail!("Text document URI does not point to a text document");
        };

        if content_changes.is_empty() {
            document.update_version(new_version);
            return Ok(());
        }

        document.apply_changes(content_changes, new_version, encoding);

        Ok(())
    }

    pub(super) fn key_from_url(&self, url: Url) -> DocumentKey {
        DocumentKey::Text(url)
    }

    pub(super) fn open_workspace_folder(
        &mut self,
        url: Url,
        global: &GlobalClientSettings,
        client: &Client,
    ) -> crate::Result<()> {
        // TODO(jane): Find a way for workspace client settings to be added or changed dynamically.
        self.settings
            .register_workspace(&Workspace::new(url), global, client)
    }

    pub(super) fn close_workspace_folder(&mut self, workspace_url: &Url) -> crate::Result<()> {
        let workspace_path = workspace_url.to_file_path().map_err(|()| {
            anyhow!("Failed to convert workspace URL to file path: {workspace_url}")
        })?;

        self.settings.remove(&workspace_path).ok_or_else(|| {
            anyhow!(
                "Tried to remove non-existent workspace URI {}",
                workspace_url
            )
        })?;

        // O(n) complexity, which isn't ideal... but this is an uncommon operation.
        self.documents
            .retain(|url, _| !Path::new(url.path()).starts_with(&workspace_path));

        Ok(())
    }

    pub(super) fn make_document_ref(
        &self,
        key: DocumentKey,
        global: &GlobalClientSettings,
    ) -> Option<DocumentQuery> {
        let url = self.url_for_key(&key)?.clone();

        let document_settings = self
            .settings_for_url(&url)
            .map(|settings| {
                if let Ok(file_path) = url.to_file_path() {
                    settings.fortitude_settings.get(&file_path)
                } else {
                    // For a new unsaved and untitled document, use the fortitude settings from the top of the workspace
                    // but only IF:
                    // * It is the only workspace
                    // * The fortitude setting is at the top of the workspace (in the root folder)
                    // Otherwise, use the fallback settings.
                    if self.settings.len() == 1 {
                        let workspace_path = self.settings.keys().next().unwrap();
                        settings
                            .fortitude_settings
                            .get(&workspace_path.join("untitled"))
                    } else {
                        tracing::debug!("Use the fallback settings for the new document '{url}'.");
                        settings.fortitude_settings.fallback()
                    }
                }
            })
            .unwrap_or_else(|| {
                tracing::warn!(
                    "No settings available for {} - falling back to default settings",
                    url
                );
                // The path here is only for completeness, it's okay to use a non-existing path
                // in case this is an unsaved (untitled) document.
                let path = Path::new(url.path());
                let root = path.parent().unwrap_or(path);
                Arc::new(FortitudeSettings::fallback(
                    global.to_settings().editor_settings(),
                    root,
                ))
            });

        let controller = self.documents.get(&url)?;
        Some(controller.make_ref(url, document_settings))
    }

    /// Reload the settings for all the workspace folders that contain the changed files.
    ///
    /// This method avoids re-indexing the same workspace multiple times if multiple files
    /// belonging to the same workspace have been changed.
    ///
    /// The file events are expected to only contain paths that map to configuration files. This is
    /// registered in [`try_register_capabilities`] method.
    ///
    /// [`try_register_capabilities`]: crate::server::Server::try_register_capabilities
    pub(super) fn reload_settings(&mut self, changes: &[FileEvent], client: &Client) {
        let mut indexed = FxHashSet::default();

        for change in changes {
            let Ok(changed_path) = change.uri.to_file_path() else {
                // Files that don't map to a path can't be a workspace configuration file.
                return;
            };

            let Some(enclosing_folder) = changed_path.parent() else {
                return;
            };

            for (root, settings) in self
                .settings
                .range_mut(..=enclosing_folder.to_path_buf())
                .rev()
            {
                if !enclosing_folder.starts_with(root) {
                    break;
                }

                if indexed.contains(root) {
                    continue;
                }
                indexed.insert(root.clone());

                settings.fortitude_settings = fortitude_settings::FortitudeSettingsIndex::new(
                    client,
                    root,
                    settings.client_settings.editor_settings(),
                    false,
                );
            }
        }
    }

    pub(super) fn open_text_document(&mut self, url: Url, document: TextDocument) {
        self.documents
            .insert(url, DocumentController::new_text(document));
    }

    pub(super) fn close_document(&mut self, key: &DocumentKey) -> crate::Result<()> {
        let Some(url) = self.url_for_key(key).cloned() else {
            anyhow::bail!("Tried to close unavailable document `{key}`");
        };

        let Some(_) = self.documents.remove(&url) else {
            anyhow::bail!(
                "tried to close document that didn't exist at {}",
                url.as_str()
            )
        };
        Ok(())
    }

    pub(super) fn client_settings(&self, key: &DocumentKey) -> Option<Arc<ClientSettings>> {
        let url = self.url_for_key(key)?;
        let WorkspaceSettings {
            client_settings, ..
        } = self.settings_for_url(url)?;
        Some(client_settings.clone())
    }

    fn document_controller_for_key(
        &mut self,
        key: &DocumentKey,
    ) -> crate::Result<&mut DocumentController> {
        let Some(url) = self.url_for_key(key).cloned() else {
            anyhow::bail!("Tried to open unavailable document `{key}`");
        };
        let Some(controller) = self.documents.get_mut(&url) else {
            anyhow::bail!("Document controller not available at `{}`", url.as_str());
        };
        Ok(controller)
    }

    fn url_for_key<'a>(&'a self, key: &'a DocumentKey) -> Option<&'a Url> {
        match key {
            DocumentKey::Text(path) => Some(path),
        }
    }
    fn settings_for_url(&self, url: &Url) -> Option<&WorkspaceSettings> {
        if let Ok(path) = url.to_file_path() {
            self.settings_for_path(&path)
        } else {
            // If there's only a single workspace, use that configuration for an untitled document.
            if self.settings.len() == 1 {
                tracing::debug!(
                    "Falling back to configuration of the only active workspace for the new document '{url}'."
                );
                self.settings.values().next()
            } else {
                None
            }
        }
    }

    fn settings_for_path(&self, path: &Path) -> Option<&WorkspaceSettings> {
        self.settings
            .range(..path.to_path_buf())
            .next_back()
            .map(|(_, settings)| settings)
    }

    /// Returns an iterator over the workspace root folders contained in this index.
    pub(super) fn workspace_root_folders(&self) -> impl Iterator<Item = &Path> {
        self.settings.keys().map(PathBuf::as_path)
    }

    /// Returns the number of open documents.
    pub(super) fn open_documents_len(&self) -> usize {
        self.documents.len()
    }

    /// Returns an iterator over the paths to the configuration files in the index.
    pub(super) fn config_file_paths(&self) -> impl Iterator<Item = &Path> {
        self.settings.values().flat_map(
            |WorkspaceSettings {
                 fortitude_settings, ..
             }| fortitude_settings.config_file_paths(),
        )
    }
}

/// Maps a workspace folder root to its settings.
#[derive(Default)]
struct WorkspaceSettingsIndex {
    index: BTreeMap<PathBuf, WorkspaceSettings>,
}

impl WorkspaceSettingsIndex {
    /// Register a workspace folder with the given settings.
    ///
    /// If the `workspace_settings` is [`Some`], it is preferred over the global settings for the
    /// workspace. Otherwise, the global settings are used exclusively.
    fn register_workspace(
        &mut self,
        workspace: &Workspace,
        global: &GlobalClientSettings,
        client: &Client,
    ) -> crate::Result<()> {
        let workspace_url = workspace.url();
        if workspace_url.scheme() != "file" {
            tracing::info!("Ignoring non-file workspace URL: {workspace_url}");
            client.show_warning_message(format_args!(
                "Fortitude does not support non-file workspaces; Ignoring {workspace_url}"
            ));
            return Ok(());
        }
        let workspace_path = workspace_url.to_file_path().map_err(|()| {
            anyhow!("Failed to convert workspace URL to file path: {workspace_url}")
        })?;

        let client_settings = if let Some(workspace_options) = workspace.options() {
            let options = workspace_options.clone().combine(global.options().clone());
            let settings = match options.into_settings() {
                Ok(settings) => settings,
                Err(settings) => {
                    client.show_error_message(format_args!(
                        "The settings for the workspace {workspace_path} are invalid. Refer to the logs for more information.",
                        workspace_path = workspace_path.display()
                    ));
                    settings
                }
            };
            Arc::new(settings)
        } else {
            global.to_settings_arc()
        };

        let workspace_settings_index = fortitude_settings::FortitudeSettingsIndex::new(
            client,
            &workspace_path,
            client_settings.editor_settings(),
            workspace.is_default(),
        );

        tracing::info!("Registering workspace: {}", workspace_path.display());
        self.insert(
            workspace_path,
            WorkspaceSettings {
                client_settings,
                fortitude_settings: workspace_settings_index,
            },
        );

        Ok(())
    }
}

impl Deref for WorkspaceSettingsIndex {
    type Target = BTreeMap<PathBuf, WorkspaceSettings>;

    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

impl DerefMut for WorkspaceSettingsIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.index
    }
}

impl DocumentController {
    fn new_text(document: TextDocument) -> Self {
        Self::Text(Arc::new(document))
    }

    fn make_ref(&self, file_url: Url, settings: Arc<FortitudeSettings>) -> DocumentQuery {
        match &self {
            Self::Text(document) => DocumentQuery::Text {
                file_url,
                document: document.clone(),
                settings,
            },
        }
    }

    pub(crate) fn as_text(&self) -> Option<&TextDocument> {
        match self {
            Self::Text(document) => Some(document),
        }
    }

    pub(crate) fn as_text_mut(&mut self) -> Option<&mut TextDocument> {
        Some(match self {
            Self::Text(document) => Arc::make_mut(document),
        })
    }
}

impl DocumentQuery {
    /// Get the document settings associated with this query.
    pub(crate) fn settings(&self) -> &FortitudeSettings {
        match self {
            Self::Text { settings, .. } => settings,
        }
    }

    /// Get source code for the linter
    pub(crate) fn make_source_kind(&self) -> String {
        match self {
            Self::Text { document, .. } => document.contents().to_string(),
        }
    }

    /// Retrieve the original key that describes this document query.
    pub(crate) fn make_key(&self) -> DocumentKey {
        match self {
            Self::Text { file_url, .. } => DocumentKey::Text(file_url.clone()),
        }
    }

    /// Get the version of document selected by this query.
    pub(crate) fn version(&self) -> DocumentVersion {
        match self {
            Self::Text { document, .. } => document.version(),
        }
    }

    /// Get the URL for the document selected by this query.
    pub(crate) fn file_url(&self) -> &Url {
        match self {
            Self::Text { file_url, .. } => file_url,
        }
    }

    /// Get the path for the document selected by this query.
    ///
    /// Returns `None` if this is an unsaved (untitled) document.
    ///
    /// The path isn't guaranteed to point to a real path on the filesystem. This is the case
    /// for unsaved (untitled) documents.
    pub(crate) fn file_path(&self) -> Option<PathBuf> {
        self.file_url().to_file_path().ok()
    }

    /// Get the path for the document selected by this query, ignoring whether the file exists on disk.
    ///
    /// Returns the URL's path if this is an unsaved (untitled) document.
    pub(crate) fn virtual_file_path(&self) -> Cow<'_, Path> {
        self.file_path()
            .map(Cow::Owned)
            .unwrap_or_else(|| Cow::Borrowed(Path::new(self.file_url().path())))
    }

    /// Attempt to access the single inner text document selected by the query.
    /// Adapted from a Ruff function that would return `None` when selecting
    /// entire notebook documents. May be extended in the future to support
    /// similar funcitonality here.
    pub(crate) fn as_single_document(&self) -> &TextDocument {
        match self {
            Self::Text { document, .. } => document,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn text_document_language_id(&self) -> Option<LanguageId> {
        match self {
            DocumentQuery::Text { document, .. } => document.language_id(),
        }
    }
}
