use arrow_buffer::Buffer;
use arrow_ipc::reader::StreamDecoder;
use futures::{Stream, StreamExt};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tonic::{Code, Status, Streaming};
use tracing::{error, instrument, warn};

use crate::{
    dataset_manager::{CreateDatasetRequest, CreateIngestEvent, CreateTerminal},
    proto::{CreateMetadata, CreateRequest, create_request::CreateMessage},
};

pub(crate) struct CreateStreamParts {
    pub request: CreateDatasetRequest,
    pub events_rx: mpsc::Receiver<CreateIngestEvent>,
    pub events_task: JoinHandle<Result<(), Status>>,
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

    let (events_tx, events_rx) = mpsc::channel(16);
    let events_task = tokio::spawn(produce_create_events(stream, shutdown_token, events_tx));

    Ok(CreateStreamParts {
        request: CreateDatasetRequest {
            name,
            description,
            tags,
        },
        events_rx,
        events_task,
    })
}

async fn produce_create_events<S>(
    mut stream: S,
    shutdown_token: CancellationToken,
    events_tx: mpsc::Sender<CreateIngestEvent>,
) -> Result<(), Status>
where
    S: Stream<Item = Result<CreateRequest, Status>> + Unpin,
{
    let mut decoder = StreamDecoder::new();

    loop {
        tokio::select! {
            item = stream.next() => {
                let terminal = match item {
                    Some(Ok(request)) => {
                        match handle_stream_message(
                            request.create_message,
                            &mut decoder,
                            &events_tx,
                        )
                        .await
                        {
                            Ok(terminal) => terminal,
                            Err(status) => return send_abort_and_error(&events_tx, status).await,
                        }
                    }
                    Some(Err(error)) => {
                        return send_abort_and_error(
                            &events_tx,
                            Status::new(error.code(), "client stream error while uploading dataset"),
                        )
                        .await;
                    }
                    None => {
                        warn!("Stream closed without finish message");
                        return send_abort_and_error(
                            &events_tx,
                            Status::invalid_argument(
                                "create stream closed without terminal finish/abort message",
                            ),
                        )
                        .await;
                    }
                };

                if let Some(terminal) = terminal {
                    events_tx
                        .send(CreateIngestEvent::Terminal(terminal))
                        .await
                        .map_err(|_| Status::internal("create ingest receiver dropped"))?;
                    return Ok(());
                }
            }
            () = shutdown_token.cancelled() => {
                warn!("Create stream cancelled because server is shutting down");
                return send_abort_and_error(
                    &events_tx,
                    Status::new(Code::Unavailable, "server is shutting down"),
                )
                .await;
            }
        }
    }
}

async fn send_abort_and_error(
    events_tx: &mpsc::Sender<CreateIngestEvent>,
    status: Status,
) -> Result<(), Status> {
    events_tx
        .send(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        .await
        .map_err(|_| Status::internal("create ingest receiver dropped"))?;
    Err(status)
}

async fn handle_stream_message(
    message: Option<CreateMessage>,
    decoder: &mut StreamDecoder,
    events_tx: &mpsc::Sender<CreateIngestEvent>,
) -> Result<Option<CreateTerminal>, Status> {
    match message {
        Some(CreateMessage::Payload(payload)) => decode_payload(payload, decoder, events_tx).await,
        Some(CreateMessage::Metadata(_)) => {
            warn!("Unexpected metadata message after initial create metadata");
            Err(Status::invalid_argument(
                "unexpected metadata message after initial create metadata",
            ))
        }
        Some(CreateMessage::Finish(_)) => match decoder.finish() {
            Ok(()) => Ok(Some(CreateTerminal::Finish)),
            Err(error) => {
                error!(error = %error, "Failed to finalize Arrow stream on CreateFinish");
                Err(Status::invalid_argument(
                    "invalid Arrow stream at create finish",
                ))
            }
        },
        Some(CreateMessage::Abort(_)) => Ok(Some(CreateTerminal::Abort)),
        None => {
            warn!("Received empty CreateRequest message");
            Err(Status::invalid_argument(
                "create request message body is empty",
            ))
        }
    }
}

async fn decode_payload(
    payload: bytes::Bytes,
    decoder: &mut StreamDecoder,
    events_tx: &mpsc::Sender<CreateIngestEvent>,
) -> Result<Option<CreateTerminal>, Status> {
    let mut buffer = Buffer::from(payload);
    while !buffer.is_empty() {
        match decoder.decode(&mut buffer) {
            Ok(Some(batch)) => {
                events_tx
                    .send(CreateIngestEvent::Batch(batch))
                    .await
                    .map_err(|_| Status::internal("create ingest receiver dropped"))?;
            }
            Ok(None) => {}
            Err(error) => {
                error!(error = %error, "Failed to decode Arrow payload");
                return Err(Status::invalid_argument("failed to decode Arrow payload"));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::{Int32Array, RecordBatch};
    use arrow_ipc::writer::StreamWriter;
    use arrow_schema::{DataType, Field, Schema};
    use futures::stream;
    use tokio::sync::mpsc;

    use super::*;
    use crate::proto::{CreateAbort, CreateFinish};

    fn payload_message(payload: bytes::Bytes) -> CreateRequest {
        CreateRequest {
            create_message: Some(CreateMessage::Payload(payload)),
        }
    }

    fn finish_message() -> CreateRequest {
        CreateRequest {
            create_message: Some(CreateMessage::Finish(CreateFinish {})),
        }
    }

    fn abort_message() -> CreateRequest {
        CreateRequest {
            create_message: Some(CreateMessage::Abort(CreateAbort {})),
        }
    }

    fn metadata_message() -> CreateRequest {
        CreateRequest {
            create_message: Some(CreateMessage::Metadata(CreateMetadata {
                name: "name".to_string(),
                description: "desc".to_string(),
                tags: vec![],
            })),
        }
    }

    fn build_payload_bytes() -> bytes::Bytes {
        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int32Array::from(vec![1, 2, 3]))],
        )
        .expect("batch");
        let mut bytes = vec![];
        let mut writer = StreamWriter::try_new(&mut bytes, &schema).expect("stream writer");
        writer.write(&batch).expect("write batch");
        writer.finish().expect("finish stream writer");
        bytes::Bytes::from(bytes)
    }

    async fn collect_events(
        stream: impl Stream<Item = Result<CreateRequest, Status>> + Unpin,
        shutdown_token: CancellationToken,
    ) -> (Vec<CreateIngestEvent>, Result<(), Status>) {
        let (tx, mut rx) = mpsc::channel(16);
        let result = produce_create_events(stream, shutdown_token, tx).await;
        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        (events, result)
    }

    #[tokio::test]
    async fn payload_then_finish_produces_finish_terminal() {
        let payload = build_payload_bytes();
        let stream = stream::iter(vec![Ok(payload_message(payload)), Ok(finish_message())]);
        let (events, result) = collect_events(stream, CancellationToken::new()).await;
        assert!(result.is_ok());

        assert!(
            events
                .iter()
                .any(|event| matches!(event, CreateIngestEvent::Batch(_)))
        );
        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Finish))
        ));
    }

    #[tokio::test]
    async fn stream_closed_without_finish_returns_invalid_argument() {
        let payload = build_payload_bytes();
        let stream = stream::iter(vec![Ok(payload_message(payload))]);
        let (events, result) = collect_events(stream, CancellationToken::new()).await;
        assert_eq!(
            result.expect_err("should fail").code(),
            Code::InvalidArgument
        );

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        ));
    }

    #[tokio::test]
    async fn payload_then_abort_produces_abort_terminal() {
        let payload = build_payload_bytes();
        let stream = stream::iter(vec![Ok(payload_message(payload)), Ok(abort_message())]);
        let (events, result) = collect_events(stream, CancellationToken::new()).await;
        assert!(result.is_ok());

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        ));
    }

    #[tokio::test]
    async fn metadata_after_first_message_returns_invalid_argument() {
        let stream = stream::iter(vec![Ok(metadata_message())]);
        let (events, result) = collect_events(stream, CancellationToken::new()).await;
        assert_eq!(
            result.expect_err("should fail").code(),
            Code::InvalidArgument
        );

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        ));
    }

    #[tokio::test]
    async fn finish_stops_stream_and_ignores_trailing_message() {
        let payload = build_payload_bytes();
        let stream = stream::iter(vec![
            Ok(payload_message(payload)),
            Ok(finish_message()),
            Ok(finish_message()),
        ]);
        let (events, result) = collect_events(stream, CancellationToken::new()).await;
        assert!(result.is_ok());

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Finish))
        ));
    }

    #[tokio::test]
    async fn invalid_payload_returns_invalid_argument() {
        let stream = stream::iter(vec![
            Ok(payload_message(bytes::Bytes::from_static(b"not-arrow"))),
            Ok(finish_message()),
        ]);
        let (events, result) = collect_events(stream, CancellationToken::new()).await;
        assert_eq!(
            result.expect_err("should fail").code(),
            Code::InvalidArgument
        );

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        ));
    }

    #[tokio::test]
    async fn shutdown_returns_unavailable() {
        let token = CancellationToken::new();
        token.cancel();
        let stream = stream::pending::<Result<CreateRequest, Status>>();
        let (events, result) = collect_events(stream, token).await;
        assert_eq!(result.expect_err("should fail").code(), Code::Unavailable);

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        ));
    }
}
