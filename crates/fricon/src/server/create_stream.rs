use arrow_buffer::Buffer;
use arrow_ipc::reader::StreamDecoder;
use futures::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tonic::{Status, Streaming};
use tracing::{error, instrument, warn};

use crate::{
    dataset_manager::{CreateDatasetRequest, CreateIngestEvent, CreateTerminal},
    proto::{CreateMetadata, CreateRequest, create_request::CreateMessage},
};

pub(crate) struct CreateStreamParts {
    pub request: CreateDatasetRequest,
    pub events_rx: mpsc::Receiver<CreateIngestEvent>,
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
    tokio::spawn(produce_create_events(stream, shutdown_token, events_tx));

    Ok(CreateStreamParts {
        request: CreateDatasetRequest {
            name,
            description,
            tags,
        },
        events_rx,
    })
}

async fn produce_create_events<S>(
    mut stream: S,
    shutdown_token: CancellationToken,
    events_tx: mpsc::Sender<CreateIngestEvent>,
) where
    S: Stream<Item = Result<CreateRequest, Status>> + Unpin,
{
    let mut decoder = StreamDecoder::new();

    loop {
        tokio::select! {
            item = stream.next() => {
                let terminal = match item {
                    Some(Ok(request)) => {
                        handle_stream_message(
                            request.create_message,
                            &mut stream,
                            &mut decoder,
                            &events_tx,
                        )
                        .await
                    }
                    Some(Err(error)) => {
                        error!(error = %error, "Client connection error while uploading dataset");
                        Some(CreateTerminal::Abort)
                    }
                    None => {
                        warn!("Stream closed without finish message");
                        Some(CreateTerminal::Abort)
                    }
                };

                if let Some(terminal) = terminal {
                    let _ = events_tx.send(CreateIngestEvent::Terminal(terminal)).await;
                    break;
                }
            }
            () = shutdown_token.cancelled() => {
                warn!("Create stream cancelled because server is shutting down");
                let _ = events_tx
                    .send(CreateIngestEvent::Terminal(CreateTerminal::Abort))
                    .await;
                break;
            }
        }
    }
}

async fn handle_stream_message<S>(
    message: Option<CreateMessage>,
    stream: &mut S,
    decoder: &mut StreamDecoder,
    events_tx: &mpsc::Sender<CreateIngestEvent>,
) -> Option<CreateTerminal>
where
    S: Stream<Item = Result<CreateRequest, Status>> + Unpin,
{
    match message {
        Some(CreateMessage::Payload(payload)) => decode_payload(payload, decoder, events_tx).await,
        Some(CreateMessage::Metadata(_)) => {
            warn!("Unexpected metadata message after initial create metadata");
            Some(CreateTerminal::Abort)
        }
        Some(CreateMessage::Finish(_)) => {
            // Finish must be the terminal message.
            match stream.next().await {
                None => match decoder.finish() {
                    Ok(()) => Some(CreateTerminal::Finish),
                    Err(error) => {
                        error!(error = %error, "Failed to finalize Arrow stream on CreateFinish");
                        Some(CreateTerminal::Abort)
                    }
                },
                Some(Ok(_)) => {
                    warn!("Unexpected message after CreateFinish");
                    Some(CreateTerminal::Abort)
                }
                Some(Err(error)) => {
                    error!(error = %error, "Client connection error while validating CreateFinish termination");
                    Some(CreateTerminal::Abort)
                }
            }
        }
        None => {
            warn!("Received empty CreateRequest message");
            Some(CreateTerminal::Abort)
        }
    }
}

async fn decode_payload(
    payload: bytes::Bytes,
    decoder: &mut StreamDecoder,
    events_tx: &mpsc::Sender<CreateIngestEvent>,
) -> Option<CreateTerminal> {
    let mut buffer = Buffer::from(payload);
    while !buffer.is_empty() {
        match decoder.decode(&mut buffer) {
            Ok(Some(batch)) => {
                if events_tx
                    .send(CreateIngestEvent::Batch(batch))
                    .await
                    .is_err()
                {
                    error!("create ingest receiver dropped while forwarding payload batch");
                    return Some(CreateTerminal::Abort);
                }
            }
            Ok(None) => {}
            Err(error) => {
                error!(error = %error, "Failed to decode Arrow payload");
                return Some(CreateTerminal::Abort);
            }
        }
    }
    None
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
    use crate::proto::CreateFinish;

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
    ) -> Vec<CreateIngestEvent> {
        let (tx, mut rx) = mpsc::channel(16);
        produce_create_events(stream, shutdown_token, tx).await;
        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        events
    }

    #[tokio::test]
    async fn payload_then_finish_produces_finish_terminal() {
        let payload = build_payload_bytes();
        let stream = stream::iter(vec![Ok(payload_message(payload)), Ok(finish_message())]);
        let events = collect_events(stream, CancellationToken::new()).await;

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
    async fn stream_closed_without_finish_produces_abort_terminal() {
        let payload = build_payload_bytes();
        let stream = stream::iter(vec![Ok(payload_message(payload))]);
        let events = collect_events(stream, CancellationToken::new()).await;

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        ));
    }

    #[tokio::test]
    async fn metadata_after_first_message_produces_abort_terminal() {
        let stream = stream::iter(vec![Ok(metadata_message())]);
        let events = collect_events(stream, CancellationToken::new()).await;

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        ));
    }

    #[tokio::test]
    async fn finish_with_trailing_message_produces_abort_terminal() {
        let payload = build_payload_bytes();
        let stream = stream::iter(vec![
            Ok(payload_message(payload)),
            Ok(finish_message()),
            Ok(finish_message()),
        ]);
        let events = collect_events(stream, CancellationToken::new()).await;

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        ));
    }

    #[tokio::test]
    async fn invalid_payload_produces_abort_terminal() {
        let stream = stream::iter(vec![
            Ok(payload_message(bytes::Bytes::from_static(b"not-arrow"))),
            Ok(finish_message()),
        ]);
        let events = collect_events(stream, CancellationToken::new()).await;

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        ));
    }

    #[tokio::test]
    async fn shutdown_produces_abort_terminal() {
        let token = CancellationToken::new();
        token.cancel();
        let stream = stream::pending::<Result<CreateRequest, Status>>();
        let events = collect_events(stream, token).await;

        assert!(matches!(
            events.last(),
            Some(CreateIngestEvent::Terminal(CreateTerminal::Abort))
        ));
    }
}
