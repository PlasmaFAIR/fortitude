use std::borrow::Cow;
use std::path::PathBuf;
use std::{path::Path, sync::Arc};

use lsp_types::Uri;
use rustc_hash::FxHashMap;

use crate::edit::LanguageId;
use crate::{
    PositionEncoding, TextDocument,
    edit::{DocumentKey, DocumentVersion},
};

/// Stores and tracks all open documents in a session, along with their associated settings.
#[derive(Default)]
pub(crate) struct Index {
    /// Maps all document file URLs to the associated document controller
    documents: FxHashMap<Uri, DocumentController>,
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
        file_url: Uri,
        document: Arc<TextDocument>,
    },
}

impl Index {
    pub(super) fn new() -> crate::Result<Self> {
        Ok(Self {
            documents: FxHashMap::default(),
        })
    }

    pub(super) fn text_document_urls(&self) -> impl Iterator<Item = &Uri> + '_ {
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

    pub(super) fn key_from_url(&self, url: Uri) -> DocumentKey {
        DocumentKey::Text(url)
    }

    pub(super) fn close_workspace_folder(&mut self, workspace_url: &Uri) -> crate::Result<()> {
        // O(n) complexity, which isn't ideal... but this is an uncommon operation.
        self.documents
            .retain(|url, _| !url.as_str().starts_with(workspace_url.as_str()));

        Ok(())
    }

    pub(super) fn make_document_ref(&self, key: DocumentKey) -> Option<DocumentQuery> {
        let url = self.url_for_key(&key)?.clone();

        let controller = self.documents.get(&url)?;
        Some(controller.make_ref(url))
    }

    pub(super) fn open_text_document(&mut self, url: Uri, document: TextDocument) {
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

    fn url_for_key<'a>(&'a self, key: &'a DocumentKey) -> Option<&'a Uri> {
        match key {
            DocumentKey::Text(path) => Some(path),
        }
    }

    /// Returns the number of open documents.
    pub(super) fn open_documents_len(&self) -> usize {
        self.documents.len()
    }
}

impl DocumentController {
    fn new_text(document: TextDocument) -> Self {
        Self::Text(Arc::new(document))
    }

    fn make_ref(&self, file_url: Uri) -> DocumentQuery {
        match &self {
            Self::Text(document) => DocumentQuery::Text {
                file_url,
                document: document.clone(),
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
    pub(crate) fn file_url(&self) -> &Uri {
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
        PathBuf::try_from(self.file_url().path().as_str()).ok()
    }

    /// Get the path for the document selected by this query, ignoring whether the file exists on disk.
    ///
    /// Returns the URL's path if this is an unsaved (untitled) document.
    pub(crate) fn virtual_file_path(&self) -> Cow<'_, Path> {
        self.file_path()
            .map(Cow::Owned)
            .unwrap_or_else(|| Cow::Borrowed(Path::new(self.file_url().path().as_str())))
    }

    /// Attempt to access the single inner text document selected by the query.
    /// If this query is selecting an entire notebook document, this will return `None`.
    pub(crate) fn as_single_document(&self) -> &TextDocument {
        match self {
            Self::Text { document, .. } => document,
        }
    }

    pub(crate) fn text_document_language_id(&self) -> Option<LanguageId> {
        match self {
            DocumentQuery::Text { document, .. } => document.language_id(),
        }
    }
}
