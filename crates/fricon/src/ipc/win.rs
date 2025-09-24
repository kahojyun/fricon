use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

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

fn write_uuid_to_socket_file(path: &Path, uuid: Uuid) -> io::Result<()> {
    let mut binary_data = [0u8; BINARY_FORMAT_SIZE];
    let (header, uuid_bytes) = binary_data.split_at_mut(4);
    header.copy_from_slice(HEADER);
    uuid_bytes.copy_from_slice(uuid.as_bytes());

    if path.exists() && read_uuid_from_socket_file(path).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Existing file is not a fricon socket file",
        ));
    }
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

fn get_pipe_name(server_uuid: Uuid) -> String {
    format!(r"\\.\pipe\fricon-{server_uuid}")
}

pub async fn connect(path: impl AsRef<Path>) -> Result<NamedPipeClient, ConnectError> {
    let socket_path = path.as_ref();
    let server_uuid = match read_uuid_from_socket_file(socket_path) {
        Ok(uuid) => uuid,
        Err(e) => {
            debug!("Failed to parse binary format from socket file: {}", e);
            return Err(ConnectError::NotFound(io::Error::other(e)));
        }
    };

    let pipe_name = get_pipe_name(server_uuid);
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
    let pipe_name: Arc<str> = get_pipe_name(server_uuid).into();
    write_uuid_to_socket_file(&socket_path, server_uuid)?;
    let first_server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(&*pipe_name)
        .inspect_err(|e| error!("Failed to create named pipe server: {e}"))?;
    debug!(
        "Created named pipe server with UUID: {}, socket file: {}",
        server_uuid,
        socket_path.display()
    );

    let file = SocketFile {
        socket_path,
        server_uuid,
    };
    let stream = stream::try_unfold((first_server, file), move |(current_server, file)| {
        let pipe_name = pipe_name.clone();
        async move {
            current_server.connect().await?;
            let connector = NamedPipeConnector(current_server);
            let new_server = ServerOptions::new().create(&*pipe_name)?;
            Ok(Some((connector, (new_server, file))))
        }
    });
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
