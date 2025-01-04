use std::{
    io,
    path::{self, Path},
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::Result;
use async_stream::try_stream;
use futures::{prelude::*, stream::BoxStream};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions},
};
use tonic::transport::server::Connected;
use tracing::error;

use crate::paths::IpcFile;

use super::Ipc;

impl Ipc for &IpcFile {
    type ClientStream = NamedPipeClient;
    type ListenerStream = BoxStream<'static, Result<NamedPipeConnector>>;

    async fn connect(self) -> Result<Self::ClientStream> {
        let pipe_name = get_pipe_name(&self.0)?;
        Ok(ClientOptions::new().open(pipe_name)?)
    }

    async fn listen(self) -> Result<Self::ListenerStream> {
        let pipe_name = get_pipe_name(&self.0)?;
        Ok(try_stream! {
            let mut server = ServerOptions::new()
                .first_pipe_instance(true)
                .create(&pipe_name)
                .inspect_err(|e| error!("Failed to create server: {e}"))?;
            loop {
                server.connect().await?;
                let connector = NamedPipeConnector(server);
                server = ServerOptions::new().create(&pipe_name)?;
                yield connector;
            }
        }
        .boxed())
    }

    fn cleanup(self) {}
}

fn get_pipe_name(path: &Path) -> Result<String> {
    let abspath = path::absolute(path)?;
    Ok(format!(r"\\.\pipe\{}", abspath.display()))
}

pub struct NamedPipeConnector(NamedPipeServer);

impl Connected for NamedPipeConnector {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {}
}

impl AsyncWrite for NamedPipeConnector {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().0).poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }
}

impl AsyncRead for NamedPipeConnector {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
    }
}
