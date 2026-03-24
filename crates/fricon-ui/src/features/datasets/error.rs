use fricon::dataset::{catalog::CatalogError, read::ReadError};

#[derive(Debug, thiserror::Error)]
pub(crate) enum UiDatasetError {
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error(transparent)]
    Read(#[from] ReadError),
    #[error("{message}")]
    Validation { message: String },
}

impl UiDatasetError {
    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }
}
