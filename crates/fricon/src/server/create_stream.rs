use std::io::{Error as IoError, ErrorKind};

use arrow_array::RecordBatchReader;
use arrow_ipc::reader::StreamReader;
use futures::{StreamExt, stream};
use tokio_util::{
    io::{StreamReader as TokioStreamReader, SyncIoBridge},
    sync::CancellationToken,
};
use tonic::{Status, Streaming};
use tracing::{error, instrument, warn};

use crate::{
    dataset_manager::{CreateDatasetRequest, DatasetManagerError},
    proto::{CreateAbort, CreateMetadata, CreateRequest, create_request::CreateMessage},
};

pub(crate) type BatchReader = Box<dyn RecordBatchReader + Send>;
pub(crate) type CreateBatchReader =
    Box<dyn FnOnce() -> Result<BatchReader, DatasetManagerError> + Send>;

pub(crate) struct CreateStreamParts {
    pub request: CreateDatasetRequest,
    pub reader: CreateBatchReader,
}

#[instrument(skip_all, fields(rpc.method = "dataset.create"))]
pub(crate) async fn parse_create_stream(
    mut stream: Streaming<CreateRequest>,
    shutdown_token: CancellationToken,
) -> Result<CreateStreamParts, Status> {
    let first_message = stream
        .next()
        .await
        .ok_or_else(|| Status::invalid_argument("request stream is empty"))?
        .map_err(|e| {
            error!(error = %e, "Failed to read first message");
            Status::internal("failed to read first message")
        })?;
    let Some(CreateMessage::Metadata(CreateMetadata {
        name,
        description,
        tags,
    })) = first_message.create_message
    else {
        warn!("First create stream message must be metadata");
        return Err(Status::invalid_argument(
            "first message must be CreateMetadata",
        ));
    };

    let bytes_stream = stream.map(|request| {
        let request = request.map_err(|e| {
            error!(error = %e, "Client connection error while uploading dataset");
            IoError::other(e)
        })?;
        match request.create_message {
            Some(CreateMessage::Payload(data)) => Ok(data),
            Some(CreateMessage::Metadata(_)) => {
                warn!("Unexpected metadata message after initial create metadata");
                Err(IoError::new(
                    ErrorKind::InvalidInput,
                    "unexpected CreateMetadata message after the first message",
                ))
            }
            Some(CreateMessage::Abort(CreateAbort { reason })) => {
                warn!(reason = %reason, "Client aborted dataset upload");
                Err(IoError::new(
                    ErrorKind::UnexpectedEof,
                    format!("client aborted the upload: {reason}"),
                ))
            }
            None => {
                warn!("Received empty CreateRequest message");
                Err(IoError::new(
                    ErrorKind::InvalidInput,
                    "empty CreateRequest message",
                ))
            }
        }
    });

    let abortable_stream = stream::unfold(
        (bytes_stream, shutdown_token, false),
        |(mut stream, token, cancelled)| async move {
            if cancelled {
                return None;
            }

            tokio::select! {
                item = stream.next() => {
                    item.map(|item| (item, (stream, token, false)))
                }
                () = token.cancelled() => {
                    Some((
                        Err(IoError::other(
                            "Stream aborted because server is shutting down.")),
                        (stream, token, true),
                    ))
                }
            }
        },
    )
    .boxed();

    let reader: CreateBatchReader = Box::new(move || {
        let sync_reader = SyncIoBridge::new(TokioStreamReader::new(abortable_stream));
        StreamReader::try_new(sync_reader, None)
            .map(|reader| Box::new(reader) as BatchReader)
            .map_err(|e| DatasetManagerError::BatchStream {
                message: e.to_string(),
            })
    });

    Ok(CreateStreamParts {
        request: CreateDatasetRequest {
            name,
            description,
            tags,
        },
        reader,
    })
}
