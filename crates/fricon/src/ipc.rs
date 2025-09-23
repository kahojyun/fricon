//! Provides cross-platform inter-process communication (IPC) functionality.
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod win;

use thiserror::Error;

#[cfg(unix)]
pub use self::unix::{connect, listen};
#[cfg(windows)]
pub use self::win::{connect, listen};

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Connect target not found: {0}")]
    NotFound(#[source] std::io::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::pin::pin;

    use futures::StreamExt;
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    #[tokio::test]
    async fn connect_succeed() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("fricon.sock");
        {
            let server = listen(&path).unwrap();
            let mut client = connect(&path).await.unwrap();

            let server_task = tokio::spawn(async move {
                let mut stream = pin!(server).next().await.unwrap().unwrap();
                let mut buf = [0; 11];
                stream.read_exact(&mut buf).await.unwrap();
                assert_eq!(&buf, b"hello world");
            });

            let client_task = tokio::spawn(async move {
                client.write_all(b"hello world").await.unwrap();
            });

            server_task.await.unwrap();
            client_task.await.unwrap();
        }
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn not_found() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("fricon.sock");

        let result = connect(&path).await;
        assert!(matches!(result, Err(ConnectError::NotFound(_))));
    }
}
