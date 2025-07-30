use std::{
    io,
    path::{self, Path},
    pin::Pin,
    task::{Context, Poll},
};

use async_stream::try_stream;
use futures::{prelude::*, stream::BoxStream};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions},
};
use tonic::transport::server::Connected;
use tracing::error;

use super::ConnectError;

pub async fn connect(path: impl AsRef<Path>) -> Result<NamedPipeClient, ConnectError> {
    let pipe_name = get_pipe_name(path)?;
    ClientOptions::new()
        .open(pipe_name)
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => ConnectError::NotFound(e),
            _ => ConnectError::Io(e),
        })
}

pub fn listen(
    path: impl AsRef<Path>,
) -> io::Result<BoxStream<'static, io::Result<NamedPipeConnector>>> {
    let pipe_name = get_pipe_name(path)?;
    let mut server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(&pipe_name)
        .inspect_err(|e| error!("Failed to create named pipe server: {e}"))?;
    Ok(try_stream! {
        loop {
            server.connect().await?;
            let connector = NamedPipeConnector(server);
            server = ServerOptions::new().create(&pipe_name)?;
            yield connector;
        }
    }
    .boxed())
}

fn get_pipe_name(path: impl AsRef<Path>) -> io::Result<String> {
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
