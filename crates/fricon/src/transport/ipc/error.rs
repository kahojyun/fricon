use std::io::Error as IoError;

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ConnectError {
    #[error("Connect target not found: {0}")]
    NotFound(#[source] IoError),
    #[error("IO error: {0}")]
    Io(#[from] IoError),
}
