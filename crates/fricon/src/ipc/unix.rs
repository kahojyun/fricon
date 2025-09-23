use std::{
    fs, io,
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use tokio::net::{UnixListener, UnixStream};
use tokio_stream::wrappers::UnixListenerStream;
use tracing::debug;

use super::ConnectError;

pub async fn connect(path: impl AsRef<Path>) -> Result<UnixStream, ConnectError> {
    UnixStream::connect(path).await.map_err(|e| match e.kind() {
        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused => ConnectError::NotFound(e),
        _ => ConnectError::Io(e),
    })
}

pub fn listen(path: impl Into<PathBuf>) -> io::Result<IpcListenerStream> {
    let path = path.into();
    if let Ok(metadata) = fs::metadata(&path) {
        if metadata.file_type().is_socket() {
            debug!("Removing existing IPC socket at {}", path.display());
            fs::remove_file(&path)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "A file already exists at {} but it is not a socket.",
                    path.display()
                ),
            ));
        }
    }
    let listener = UnixListener::bind(&path)?;
    let inode = fs::metadata(&path)?.ino();
    Ok(IpcListenerStream {
        path,
        listener: UnixListenerStream::new(listener),
        inode,
    })
}

pub struct IpcListenerStream {
    path: PathBuf,
    listener: UnixListenerStream,
    inode: u64,
}

impl Stream for IpcListenerStream {
    type Item = io::Result<UnixStream>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.listener.poll_next_unpin(cx)
    }
}

impl Drop for IpcListenerStream {
    fn drop(&mut self) {
        if let Ok(metadata) = fs::metadata(&self.path) {
            if metadata.ino() == self.inode {
                debug!("Removing IPC socket at {}", self.path.display());
                fs::remove_file(&self.path).ok();
            } else {
                debug!(
                    "Socket file at {} has different inode, not removing",
                    self.path.display()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn last_opened_kept() {
        let temp_dir = tempfile::tempdir().unwrap();
        let socket_path = temp_dir.path().join("test_socket.sock");

        // Create the first listener
        let listener1 = listen(&socket_path).unwrap();
        let inode1 = fs::metadata(&socket_path).unwrap().ino();
        assert!(socket_path.exists());
        assert_eq!(listener1.inode, inode1);

        // Create the second listener at the same path
        // This will cause the old file to be removed and a new one created.
        let listener2 = listen(&socket_path).unwrap();
        let inode2 = fs::metadata(&socket_path).unwrap().ino();
        assert!(socket_path.exists());
        assert_eq!(listener2.inode, inode2);
        assert_ne!(
            inode1, inode2,
            "Inodes should be different after recreation"
        );

        // Drop listener1. It should NOT remove the file because its inode doesn't match
        // the current one.
        drop(listener1);
        assert!(
            socket_path.exists(),
            "Socket file should still exist after listener1 drops"
        );
        assert_eq!(
            fs::metadata(&socket_path).unwrap().ino(),
            inode2,
            "File inode should still be inode2"
        );

        // Drop listener2. It SHOULD remove the file.
        drop(listener2);
        assert!(
            !socket_path.exists(),
            "Socket file should be removed after listener2 drops"
        );
    }
}
