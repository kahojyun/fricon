use std::{
    fs, io,
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
    let listener = UnixListener::bind(&path)?;
    Ok(IpcListenerStream {
        path,
        listener: UnixListenerStream::new(listener),
    })
}

pub struct IpcListenerStream {
    path: PathBuf,
    listener: UnixListenerStream,
}

impl Stream for IpcListenerStream {
    type Item = io::Result<UnixStream>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.listener.poll_next_unpin(cx)
    }
}

impl Drop for IpcListenerStream {
    fn drop(&mut self) {
        debug!("Removing IPC socket at {}", self.path.display());
        fs::remove_file(&self.path).ok();
    }
}
