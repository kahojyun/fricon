//! Provides cross-platform inter-process communication (IPC) functionality.
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::{connect, listen};
#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::{connect, listen};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Connect target not found")]
    NotFound(#[source] std::io::Error),
    #[error("Unexpected IO error")]
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
