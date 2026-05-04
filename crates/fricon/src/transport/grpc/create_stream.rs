use arrow_buffer::Buffer;
use arrow_ipc::reader::StreamDecoder;
use futures::{Stream, StreamExt};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tonic::{Code, Status, Streaming};
use tracing::{error, instrument, warn};

use crate::{
    dataset::{
        CreateDatasetInput, CreateDatasetRequest,
        semantics::{ColumnMetadata, ScanAxis, ScanAxisMode, ScanAxisValue, ScanPlan},
    },
    proto::{
        ColumnMetadata as ProtoColumnMetadata, CreateMetadata, CreateRequest,
        ScanAxisMetadata as ProtoScanAxisMetadata, ScanAxisValue as ProtoScanAxisValue,
        create_request::CreateMessage, scan_axis_metadata, scan_axis_value,
    },
};

const MAX_UNPAIRED_SIDECAR_BATCHES: usize = 1;

pub(crate) struct CreateStreamParts {
    pub request: CreateDatasetRequest,
    pub events_rx: mpsc::Receiver<CreateDatasetInput>,
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
        .inspect_err(|e| {
            error!(error = %e, "Failed to read first message");
        })
        .map_err(|_| Status::internal("failed to read first message"))?;
    let Some(CreateMessage::Metadata(CreateMetadata {
        name,
        description,
        tags,
        columns,
        scan_axes,
        logical_index_sidecar,
    })) = first_message.create_message
    else {
        warn!("First create stream message must be metadata");
        return Err(Status::invalid_argument(
            "first message must be CreateMetadata",
        ));
    };

    let scan_plan = scan_plan_from_proto(scan_axes)?;
    if logical_index_sidecar && scan_plan.is_none() {
        return Err(Status::invalid_argument(
            "logical-index sidecar requires scan axes",
        ));
    }

    let (events_tx, events_rx) = mpsc::channel(16);
    let events_task = tokio::spawn(produce_create_events(
        stream,
        shutdown_token,
        events_tx,
        logical_index_sidecar,
    ));

    Ok(CreateStreamParts {
        request: CreateDatasetRequest {
            name,
            description,
            tags,
            column_metadata: columns
                .into_iter()
                .map(column_metadata_from_proto)
                .collect(),
            scan_plan,
            logical_index_sidecar,
        },
        events_rx,
        events_task,
    })
}

fn scan_plan_from_proto(axes: Vec<ProtoScanAxisMetadata>) -> Result<Option<ScanPlan>, Status> {
    if axes.is_empty() {
        return Ok(None);
    }
    let axes = axes
        .into_iter()
        .map(scan_axis_from_proto)
        .collect::<Result<Vec<_>, _>>()?;
    let scan_plan = ScanPlan::new(axes);
    scan_plan
        .validate()
        .map_err(|error| Status::invalid_argument(error.to_string()))?;
    Ok(Some(scan_plan))
}

fn scan_axis_from_proto(value: ProtoScanAxisMetadata) -> Result<ScanAxis, Status> {
    let mode = match value.axis.ok_or_else(|| {
        Status::invalid_argument(format!("scan axis {} is missing a mode", value.name))
    })? {
        scan_axis_metadata::Axis::StaticAxis(axis) => {
            let values = axis
                .values
                .into_iter()
                .map(scan_axis_value_from_proto)
                .collect::<Result<Vec<_>, _>>()?;
            ScanAxisMode::Static { values }
        }
        scan_axis_metadata::Axis::IndexAxis(_) => ScanAxisMode::ImplicitIndex,
    };
    Ok(ScanAxis {
        name: value.name,
        label: value.label,
        mode,
    })
}

fn scan_axis_value_from_proto(value: ProtoScanAxisValue) -> Result<ScanAxisValue, Status> {
    match value.value.ok_or_else(|| {
        Status::invalid_argument("scan axis static value is missing a scalar value")
    })? {
        scan_axis_value::Value::IntValue(value) => Ok(ScanAxisValue::Int(value)),
        scan_axis_value::Value::FloatValue(value) => Ok(ScanAxisValue::Float(value)),
        scan_axis_value::Value::BoolValue(value) => Ok(ScanAxisValue::Bool(value)),
        scan_axis_value::Value::StringValue(value) => Ok(ScanAxisValue::String(value)),
    }
}

fn column_metadata_from_proto(value: ProtoColumnMetadata) -> ColumnMetadata {
    ColumnMetadata {
        name: value.name,
        unit: value.unit,
        label: value.label,
        hidden_by_default: value.hidden_by_default,
        chart_axis: value.chart_axis,
    }
}

async fn produce_create_events<S>(
    mut stream: S,
    shutdown_token: CancellationToken,
    events_tx: mpsc::Sender<CreateDatasetInput>,
    logical_index_sidecar: bool,
) -> Result<(), Status>
where
    S: Stream<Item = Result<CreateRequest, Status>> + Unpin,
{
    let mut state = CreateStreamDecodeState::new(logical_index_sidecar);

    loop {
        tokio::select! {
            item = stream.next() => {
                let terminal = match item {
                    Some(Ok(request)) => {
                        match handle_stream_message(
                            request.create_message,
                            &mut state,
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
                        .send(terminal)
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
            () = events_tx.closed() => {
                warn!("Create stream consumer dropped; stopping event producer");
                return Ok(());
            }
        }
    }
}

struct CreateStreamDecodeState {
    data_decoder: StreamDecoder,
    logical_index_decoder: StreamDecoder,
    schema_sent: bool,
    logical_index_sidecar: bool,
    pending_data: std::collections::VecDeque<arrow_array::RecordBatch>,
    pending_logical_indices: std::collections::VecDeque<arrow_array::RecordBatch>,
}

impl CreateStreamDecodeState {
    fn new(logical_index_sidecar: bool) -> Self {
        Self {
            data_decoder: StreamDecoder::new(),
            logical_index_decoder: StreamDecoder::new(),
            schema_sent: false,
            logical_index_sidecar,
            pending_data: std::collections::VecDeque::new(),
            pending_logical_indices: std::collections::VecDeque::new(),
        }
    }

    fn has_unpaired_batches(&self) -> bool {
        !self.pending_data.is_empty() || !self.pending_logical_indices.is_empty()
    }

    fn reject_if_unpaired_limit_exceeded(&self) -> Result<(), Status> {
        if self.pending_data.len() > MAX_UNPAIRED_SIDECAR_BATCHES
            || self.pending_logical_indices.len() > MAX_UNPAIRED_SIDECAR_BATCHES
        {
            return Err(Status::invalid_argument(
                "logical-index sidecar batches must be paired with data batches",
            ));
        }
        Ok(())
    }
}

async fn send_abort_and_error(
    events_tx: &mpsc::Sender<CreateDatasetInput>,
    status: Status,
) -> Result<(), Status> {
    events_tx
        .send(CreateDatasetInput::Abort)
        .await
        .map_err(|_| Status::internal("create ingest receiver dropped"))?;
    Err(status)
}

async fn handle_stream_message(
    message: Option<CreateMessage>,
    state: &mut CreateStreamDecodeState,
    events_tx: &mpsc::Sender<CreateDatasetInput>,
) -> Result<Option<CreateDatasetInput>, Status> {
    match message {
        Some(CreateMessage::Payload(payload)) => decode_payload(payload, state, events_tx).await,
        Some(CreateMessage::LogicalIndexPayload(payload)) => {
            decode_logical_index_payload(payload, state, events_tx).await
        }
        Some(CreateMessage::Metadata(_)) => {
            warn!("Unexpected metadata message after initial create metadata");
            Err(Status::invalid_argument(
                "unexpected metadata message after initial create metadata",
            ))
        }
        Some(CreateMessage::Finish(_)) => {
            match state.data_decoder.finish() {
                Ok(()) => {}
                Err(error) => {
                    error!(error = %error, "Failed to finalize Arrow stream on CreateFinish");
                    return Err(Status::invalid_argument(
                        "invalid Arrow stream at create finish",
                    ));
                }
            }
            if state.logical_index_sidecar {
                if let Err(error) = state.logical_index_decoder.finish() {
                    error!(error = %error, "Failed to finalize logical-index stream on CreateFinish");
                    return Err(Status::invalid_argument(
                        "invalid logical-index stream at create finish",
                    ));
                }
                flush_paired_batches(state, events_tx).await?;
                if state.has_unpaired_batches() {
                    return Err(Status::invalid_argument(
                        "logical-index sidecar batches do not match data batches",
                    ));
                }
            }
            Ok(Some(CreateDatasetInput::Finish))
        }
        Some(CreateMessage::Abort(_)) => Ok(Some(CreateDatasetInput::Abort)),
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
    state: &mut CreateStreamDecodeState,
    events_tx: &mpsc::Sender<CreateDatasetInput>,
) -> Result<Option<CreateDatasetInput>, Status> {
    let mut buffer = Buffer::from(payload);
    while !buffer.is_empty() {
        match state.data_decoder.decode(&mut buffer) {
            Ok(Some(batch)) => {
                send_schema_if_decoded(state, events_tx).await?;
                if state.logical_index_sidecar {
                    state.pending_data.push_back(batch);
                    flush_paired_batches(state, events_tx).await?;
                    state.reject_if_unpaired_limit_exceeded()?;
                } else {
                    events_tx
                        .send(CreateDatasetInput::Batch {
                            data: batch,
                            logical_indices: None,
                        })
                        .await
                        .map_err(|_| Status::internal("create ingest receiver dropped"))?;
                }
            }
            Ok(None) => {
                send_schema_if_decoded(state, events_tx).await?;
            }
            Err(error) => {
                error!(error = %error, "Failed to decode Arrow payload");
                return Err(Status::invalid_argument("failed to decode Arrow payload"));
            }
        }
    }
    Ok(None)
}

async fn decode_logical_index_payload(
    payload: bytes::Bytes,
    state: &mut CreateStreamDecodeState,
    events_tx: &mpsc::Sender<CreateDatasetInput>,
) -> Result<Option<CreateDatasetInput>, Status> {
    if !state.logical_index_sidecar {
        return Err(Status::invalid_argument(
            "unexpected logical-index payload without sidecar metadata",
        ));
    }
    let mut buffer = Buffer::from(payload);
    while !buffer.is_empty() {
        match state.logical_index_decoder.decode(&mut buffer) {
            Ok(Some(batch)) => {
                state.pending_logical_indices.push_back(batch);
                flush_paired_batches(state, events_tx).await?;
                state.reject_if_unpaired_limit_exceeded()?;
            }
            Ok(None) => {}
            Err(error) => {
                error!(error = %error, "Failed to decode logical-index payload");
                return Err(Status::invalid_argument(
                    "failed to decode logical-index payload",
                ));
            }
        }
    }
    Ok(None)
}

async fn flush_paired_batches(
    state: &mut CreateStreamDecodeState,
    events_tx: &mpsc::Sender<CreateDatasetInput>,
) -> Result<(), Status> {
    while !state.pending_data.is_empty() && !state.pending_logical_indices.is_empty() {
        let data = state
            .pending_data
            .pop_front()
            .expect("pending data should be present");
        let logical_indices = state
            .pending_logical_indices
            .pop_front()
            .expect("pending logical indices should be present");
        events_tx
            .send(CreateDatasetInput::Batch {
                data,
                logical_indices: Some(logical_indices),
            })
            .await
            .map_err(|_| Status::internal("create ingest receiver dropped"))?;
    }
    Ok(())
}

async fn send_schema_if_decoded(
    state: &mut CreateStreamDecodeState,
    events_tx: &mpsc::Sender<CreateDatasetInput>,
) -> Result<(), Status> {
    if !state.schema_sent
        && let Some(schema) = state.data_decoder.schema()
    {
        events_tx
            .send(CreateDatasetInput::Schema(schema))
            .await
            .map_err(|_| Status::internal("create ingest receiver dropped"))?;
        state.schema_sent = true;
    }
    Ok(())
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
    use crate::proto::{
        ColumnMetadata as ProtoColumnMetadata, CreateAbort, CreateFinish, StaticScanAxis,
    };

    fn payload_message(payload: bytes::Bytes) -> CreateRequest {
        CreateRequest {
            create_message: Some(CreateMessage::Payload(payload)),
        }
    }

    fn logical_index_payload_message(payload: bytes::Bytes) -> CreateRequest {
        CreateRequest {
            create_message: Some(CreateMessage::LogicalIndexPayload(payload)),
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
                columns: vec![],
                scan_axes: vec![],
                logical_index_sidecar: false,
            })),
        }
    }

    fn metadata_message_with_columns() -> CreateRequest {
        CreateRequest {
            create_message: Some(CreateMessage::Metadata(CreateMetadata {
                name: "name".to_string(),
                description: "desc".to_string(),
                tags: vec![],
                columns: vec![ProtoColumnMetadata {
                    name: "signal".to_string(),
                    unit: Some("V".to_string()),
                    label: Some("Voltage".to_string()),
                    hidden_by_default: true,
                    chart_axis: true,
                }],
                scan_axes: vec![],
                logical_index_sidecar: false,
            })),
        }
    }

    fn metadata_message_with_scan() -> CreateRequest {
        CreateRequest {
            create_message: Some(CreateMessage::Metadata(CreateMetadata {
                name: "name".to_string(),
                description: "desc".to_string(),
                tags: vec![],
                columns: vec![],
                scan_axes: vec![ProtoScanAxisMetadata {
                    name: "gate".to_string(),
                    label: None,
                    axis: Some(scan_axis_metadata::Axis::StaticAxis(StaticScanAxis {
                        values: vec![
                            ProtoScanAxisValue {
                                value: Some(scan_axis_value::Value::FloatValue(-0.2)),
                            },
                            ProtoScanAxisValue {
                                value: Some(scan_axis_value::Value::FloatValue(-0.1)),
                            },
                        ],
                    })),
                }],
                logical_index_sidecar: false,
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

    fn build_two_data_batch_payload_bytes() -> bytes::Bytes {
        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));
        let first_batch =
            RecordBatch::try_new(schema.clone(), vec![Arc::new(Int32Array::from(vec![1]))])
                .expect("first batch");
        let second_batch =
            RecordBatch::try_new(schema.clone(), vec![Arc::new(Int32Array::from(vec![2]))])
                .expect("second batch");
        let mut bytes = vec![];
        let mut writer = StreamWriter::try_new(&mut bytes, &schema).expect("stream writer");
        writer.write(&first_batch).expect("write first batch");
        writer.write(&second_batch).expect("write second batch");
        writer.finish().expect("finish stream writer");
        bytes::Bytes::from(bytes)
    }

    fn build_logical_index_payload_bytes() -> bytes::Bytes {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "step",
            DataType::Int64,
            false,
        )]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(arrow_array::Int64Array::from(vec![2, 0, 1]))],
        )
        .expect("logical batch");
        let mut bytes = vec![];
        let mut writer = StreamWriter::try_new(&mut bytes, &schema).expect("stream writer");
        writer.write(&batch).expect("write logical batch");
        writer.finish().expect("finish logical stream writer");
        bytes::Bytes::from(bytes)
    }

    fn build_two_logical_index_batch_payload_bytes() -> bytes::Bytes {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "step",
            DataType::Int64,
            false,
        )]));
        let first_batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(arrow_array::Int64Array::from(vec![0]))],
        )
        .expect("first logical batch");
        let second_batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(arrow_array::Int64Array::from(vec![1]))],
        )
        .expect("second logical batch");
        let mut bytes = vec![];
        let mut writer = StreamWriter::try_new(&mut bytes, &schema).expect("stream writer");
        writer
            .write(&first_batch)
            .expect("write first logical batch");
        writer
            .write(&second_batch)
            .expect("write second logical batch");
        writer.finish().expect("finish logical stream writer");
        bytes::Bytes::from(bytes)
    }

    async fn collect_events(
        stream: impl Stream<Item = Result<CreateRequest, Status>> + Unpin,
        shutdown_token: CancellationToken,
    ) -> (Vec<CreateDatasetInput>, Result<(), Status>) {
        let (tx, mut rx) = mpsc::channel(16);
        let result = produce_create_events(stream, shutdown_token, tx, false).await;
        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        (events, result)
    }

    async fn collect_events_with_sidecar(
        stream: impl Stream<Item = Result<CreateRequest, Status>> + Unpin,
        shutdown_token: CancellationToken,
    ) -> (Vec<CreateDatasetInput>, Result<(), Status>) {
        let (tx, mut rx) = mpsc::channel(16);
        let result = produce_create_events(stream, shutdown_token, tx, true).await;
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

        assert!(matches!(
            events.first(),
            Some(CreateDatasetInput::Schema(_))
        ));
        assert!(
            events
                .iter()
                .any(|event| matches!(event, CreateDatasetInput::Batch { .. }))
        );
        assert!(matches!(events.last(), Some(CreateDatasetInput::Finish)));
    }

    #[tokio::test]
    async fn sidecar_payloads_are_paired_with_data_batches() {
        let payload = build_payload_bytes();
        let logical_payload = build_logical_index_payload_bytes();
        let stream = stream::iter(vec![
            Ok(payload_message(payload)),
            Ok(logical_index_payload_message(logical_payload)),
            Ok(finish_message()),
        ]);
        let (events, result) = collect_events_with_sidecar(stream, CancellationToken::new()).await;
        assert!(result.is_ok());

        assert!(
            events
                .iter()
                .any(|event| matches!(event, CreateDatasetInput::Schema(_)))
        );
        let batch = events
            .iter()
            .find_map(|event| match event {
                CreateDatasetInput::Batch {
                    data,
                    logical_indices: Some(logical_indices),
                } => Some((data, logical_indices)),
                _ => None,
            })
            .expect("paired batch");
        assert_eq!(batch.0.num_rows(), 3);
        assert_eq!(batch.1.num_rows(), 3);
        assert!(matches!(events.last(), Some(CreateDatasetInput::Finish)));
    }

    #[tokio::test]
    async fn sidecar_rejects_multiple_unpaired_data_batches() {
        let payload = build_two_data_batch_payload_bytes();
        let stream = stream::iter(vec![Ok(payload_message(payload))]);
        let (events, result) = collect_events_with_sidecar(stream, CancellationToken::new()).await;

        assert_eq!(
            result
                .expect_err("unpaired data batches should fail")
                .code(),
            Code::InvalidArgument
        );
        assert!(matches!(events.last(), Some(CreateDatasetInput::Abort)));
    }

    #[tokio::test]
    async fn sidecar_rejects_multiple_unpaired_logical_index_batches() {
        let logical_payload = build_two_logical_index_batch_payload_bytes();
        let stream = stream::iter(vec![Ok(logical_index_payload_message(logical_payload))]);
        let (events, result) = collect_events_with_sidecar(stream, CancellationToken::new()).await;

        assert_eq!(
            result
                .expect_err("unpaired logical-index batches should fail")
                .code(),
            Code::InvalidArgument
        );
        assert!(matches!(events.last(), Some(CreateDatasetInput::Abort)));
    }

    #[test]
    fn column_metadata_conversion_preserves_fields() {
        let CreateMessage::Metadata(metadata) = metadata_message_with_columns()
            .create_message
            .expect("metadata message")
        else {
            panic!("expected metadata");
        };
        let column =
            column_metadata_from_proto(metadata.columns.into_iter().next().expect("column"));

        assert_eq!(column.name, "signal");
        assert_eq!(column.unit.as_deref(), Some("V"));
        assert_eq!(column.label.as_deref(), Some("Voltage"));
        assert!(column.hidden_by_default);
        assert!(column.chart_axis);
    }

    #[test]
    fn scan_metadata_conversion_preserves_fields() {
        let CreateMessage::Metadata(metadata) = metadata_message_with_scan()
            .create_message
            .expect("metadata message")
        else {
            panic!("expected metadata");
        };
        let scan_plan = scan_plan_from_proto(metadata.scan_axes)
            .expect("scan conversion")
            .expect("scan plan");

        assert_eq!(scan_plan.axes.len(), 1);
        assert_eq!(scan_plan.axes[0].name, "gate");
        let ScanAxisMode::Static { values } = &scan_plan.axes[0].mode else {
            panic!("expected static axis");
        };
        assert_eq!(
            values,
            &[ScanAxisValue::Float(-0.2), ScanAxisValue::Float(-0.1)]
        );
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

        assert!(matches!(events.last(), Some(CreateDatasetInput::Abort)));
    }

    #[tokio::test]
    async fn payload_then_abort_produces_abort_terminal() {
        let payload = build_payload_bytes();
        let stream = stream::iter(vec![Ok(payload_message(payload)), Ok(abort_message())]);
        let (events, result) = collect_events(stream, CancellationToken::new()).await;
        assert!(result.is_ok());

        assert!(matches!(events.last(), Some(CreateDatasetInput::Abort)));
    }

    #[tokio::test]
    async fn metadata_after_first_message_returns_invalid_argument() {
        let stream = stream::iter(vec![Ok(metadata_message())]);
        let (events, result) = collect_events(stream, CancellationToken::new()).await;
        assert_eq!(
            result.expect_err("should fail").code(),
            Code::InvalidArgument
        );

        assert!(matches!(events.last(), Some(CreateDatasetInput::Abort)));
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

        assert!(matches!(events.last(), Some(CreateDatasetInput::Finish)));
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

        assert!(matches!(events.last(), Some(CreateDatasetInput::Abort)));
    }

    #[tokio::test]
    async fn shutdown_returns_unavailable() {
        let token = CancellationToken::new();
        token.cancel();
        let stream = stream::pending::<Result<CreateRequest, Status>>();
        let (events, result) = collect_events(stream, token).await;
        assert_eq!(result.expect_err("should fail").code(), Code::Unavailable);

        assert!(matches!(events.last(), Some(CreateDatasetInput::Abort)));
    }

    #[tokio::test]
    async fn receiver_closed_stops_producer_without_waiting_stream_input() {
        let stream = stream::pending::<Result<CreateRequest, Status>>();
        let (tx, rx) = mpsc::channel(1);
        drop(rx);

        let result = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            produce_create_events(stream, CancellationToken::new(), tx, false),
        )
        .await;

        assert!(
            result.is_ok(),
            "producer should stop after receiver is closed"
        );
        assert!(result.expect("join result").is_ok());
    }
}
