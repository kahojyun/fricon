use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use async_stream::try_stream;
use futures::prelude::*;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions},
};
use tonic::transport::server::Connected;
use tracing::{debug, error};
use uuid::Uuid;

use super::ConnectError;

// Binary format constants
const HEADER: &[u8; 4] = b"FRIC";
const BINARY_FORMAT_SIZE: usize = 20; // 4 bytes header + 16 bytes UUID

fn write_uuid_to_socket_file(path: &Path, uuid: &Uuid) -> io::Result<()> {
    let mut binary_data = [0u8; BINARY_FORMAT_SIZE];
    let (header, uuid_bytes) = binary_data.split_at_mut(4);
    header.copy_from_slice(HEADER);
    uuid_bytes.copy_from_slice(uuid.as_bytes());
    fs::write(path, binary_data)
}

fn read_uuid_from_socket_file(path: &Path) -> io::Result<Uuid> {
    let mut buffer = [0u8; BINARY_FORMAT_SIZE];
    let mut file = fs::File::open(path)?;
    file.read_exact(&mut buffer)?;

    let uuid_bytes = buffer.strip_prefix(HEADER).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "Invalid binary format header")
    })?;
    Ok(Uuid::from_bytes(uuid_bytes.try_into().unwrap()))
}

pub async fn connect(path: impl AsRef<Path>) -> Result<NamedPipeClient, ConnectError> {
    let socket_path = path.as_ref();

    // Read UUID from socket file
    let server_uuid = match read_uuid_from_socket_file(socket_path) {
        Ok(uuid) => uuid,
        Err(e) => {
            debug!("Failed to parse binary format from socket file: {}", e);
            return Err(ConnectError::NotFound(io::Error::other(e)));
        }
    };

    let pipe_name = format!(r"\\.\pipe\fricon-{server_uuid}");
    debug!(
        "Connecting to named pipe with UUID: {}, pipe name: {}",
        server_uuid, pipe_name
    );

    ClientOptions::new()
        .open(pipe_name)
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => ConnectError::NotFound(e),
            _ => ConnectError::Io(e),
        })
}

pub fn listen(
    path: impl Into<PathBuf>,
) -> io::Result<impl Stream<Item = io::Result<NamedPipeConnector>> + 'static> {
    let socket_path = path.into();

    // Generate a new UUID for this server instance
    let server_uuid = Uuid::new_v4();
    let pipe_name = format!(r"\\.\\pipe\\fricon-{server_uuid}");

    // Write binary format to socket file
    if socket_path.exists() {
        // Check if existing file has our format before removing
        match read_uuid_from_socket_file(&socket_path) {
            Ok(_) => {
                // File has our format, safe to remove
                fs::remove_file(&socket_path)?;
            }
            Err(_) => {
                // File doesn't have our format, return error
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Existing file is not a fricon socket file",
                ));
            }
        }
    }

    write_uuid_to_socket_file(&socket_path, &server_uuid)?;

    debug!(
        "Created named pipe server with UUID: {}, socket file: {}",
        server_uuid,
        socket_path.display()
    );

    let mut server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(&pipe_name)
        .inspect_err(|e| error!("Failed to create named pipe server: {e}"))?;

    let stream = try_stream! {
        let _file = SocketFile {
            socket_path,
            server_uuid,
        };
        loop {
            server.connect().await?;
            let connector = NamedPipeConnector(server);
            server = ServerOptions::new().create(&pipe_name)?;
            yield connector;
        }
    };
    Ok(stream)
}

pub struct SocketFile {
    socket_path: PathBuf,
    server_uuid: Uuid,
}

impl Drop for SocketFile {
    fn drop(&mut self) {
        match read_uuid_from_socket_file(&self.socket_path) {
            Ok(file_uuid) => {
                if file_uuid == self.server_uuid {
                    debug!("Removing socket file at {}", self.socket_path.display());
                    fs::remove_file(&self.socket_path).ok();
                } else {
                    debug!(
                        "Socket file at {} has different UUID, not removing",
                        self.socket_path.display()
                    );
                }
            }
            Err(e) => {
                debug!(
                    "Socket file at {} has invalid binary format ({}), not removing",
                    self.socket_path.display(),
                    e
                );
            }
        }
    }
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
