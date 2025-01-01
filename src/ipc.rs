#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod win;

use anyhow::Result;

pub trait IpcConnect {
    type ClientStream;
    type ListenerStream;

    async fn connect(self) -> Result<Self::ClientStream>;
    async fn listen(self) -> Result<Self::ListenerStream>;
    fn cleanup(self);
}
