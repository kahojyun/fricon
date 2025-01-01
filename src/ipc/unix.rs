use std::fs;

use anyhow::Result;
use tokio::net::{UnixListener, UnixStream};
use tokio_stream::wrappers::UnixListenerStream;
use tracing::debug;

use crate::paths::IpcFile;

use super::IpcConnect;

impl IpcConnect for &IpcFile {
    type ClientStream = UnixStream;
    type ListenerStream = UnixListenerStream;

    async fn connect(self) -> Result<Self::ClientStream> {
        Ok(UnixStream::connect(&self.0).await?)
    }

    async fn listen(self) -> Result<Self::ListenerStream> {
        Ok(UnixListenerStream::new(UnixListener::bind(&self.0)?))
    }

    fn cleanup(self) {
        debug!("Remove socket file: {}", self.0.display());
        fs::remove_file(&self.0).ok();
    }
}
