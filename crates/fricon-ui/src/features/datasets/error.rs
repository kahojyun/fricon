use fricon::{CatalogAppError, ReadAppError};

#[derive(Debug, thiserror::Error)]
pub(crate) enum UiDatasetError {
    #[error(transparent)]
    Catalog(#[from] CatalogAppError),
    #[error(transparent)]
    Read(#[from] ReadAppError),
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
