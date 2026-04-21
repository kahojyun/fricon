use anyhow::{Context, bail, ensure};
use serde::Serialize;

use super::types::{
    ChartSnapshot, FlatSeries, FlatXYSeries, FlatXYZSeries, LiveChartAppendOperation,
    LiveChartDataResponse, XYDrawStyle, XYPlotMode,
};

const MAGIC: &[u8; 4] = b"FCHT";
/// Feature-local chart wire format version.
///
/// Bump this whenever the binary frame layout changes incompatibly, including
/// header shape, metadata encoding, numeric payload layout, or alignment rules.
const VERSION: u8 = 2;
/// Frame header layout:
/// - bytes 0..4: magic (`FCHT`)
/// - byte 4: version
/// - byte 5: payload kind
/// - bytes 6..8: reserved
/// - bytes 8..12: metadata byte length (little-endian u32)
/// - bytes 12..16: numeric payload start offset (little-endian u32)
const HEADER_LENGTH: usize = 16;

#[derive(Clone, Copy)]
#[repr(u8)]
enum PayloadKind {
    SnapshotXy = 1,
    SnapshotHeatmap = 2,
    LiveResetXy = 3,
    LiveResetHeatmap = 4,
    LiveAppend = 5,
}

impl PayloadKind {
    const fn as_byte(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum SeriesShape {
    Xy,
    Xyz,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SeriesMetadata<'a> {
    id: &'a str,
    label: &'a str,
    point_count: usize,
    value_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct XySnapshotMetadata<'a> {
    plot_mode: XYPlotMode,
    draw_style: XYDrawStyle,
    x_name: &'a str,
    y_name: Option<&'a str>,
    series: Vec<SeriesMetadata<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HeatmapSnapshotMetadata<'a> {
    x_name: &'a str,
    y_name: &'a str,
    series: Vec<SeriesMetadata<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LiveResetMetadata<T> {
    row_count: usize,
    #[serde(flatten)]
    snapshot: T,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum LiveAppendOperationMetadata<'a> {
    AppendPoints {
        series_id: &'a str,
        shape: SeriesShape,
        point_count: usize,
        value_count: usize,
    },
    AppendSeries {
        shape: SeriesShape,
        id: &'a str,
        label: &'a str,
        point_count: usize,
        value_count: usize,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LiveAppendMetadata<'a> {
    row_count: usize,
    ops: Vec<LiveAppendOperationMetadata<'a>>,
}

/// Encodes a chart snapshot into the feature-local `FCHT` binary frame.
///
/// The numeric payload is stored as host-native `f64` bytes and padded so the
/// numeric section begins at an 8-byte-aligned offset for typed-array views on
/// the frontend. This format is intentionally local to the desktop UI bridge
/// and is not a portable persistence or interchange format.
pub(crate) fn encode_chart_snapshot(snapshot: &ChartSnapshot) -> anyhow::Result<Vec<u8>> {
    match snapshot {
        ChartSnapshot::Xy(snapshot) => {
            let mut values = Vec::with_capacity(snapshot.series.len());
            let series = snapshot
                .series
                .iter()
                .map(|series| {
                    let meta = encode_xy_series_metadata(series)?;
                    values.push(series.values.as_slice());
                    Ok(meta)
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            encode_frame(
                PayloadKind::SnapshotXy,
                &XySnapshotMetadata {
                    plot_mode: snapshot.plot_mode,
                    draw_style: snapshot.draw_style,
                    x_name: &snapshot.x_name,
                    y_name: snapshot.y_name.as_deref(),
                    series,
                },
                values,
            )
        }
        ChartSnapshot::Heatmap(snapshot) => {
            let mut values = Vec::with_capacity(snapshot.series.len());
            let series = snapshot
                .series
                .iter()
                .map(|series| {
                    let meta = encode_xyz_series_metadata(series)?;
                    values.push(series.values.as_slice());
                    Ok(meta)
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            encode_frame(
                PayloadKind::SnapshotHeatmap,
                &HeatmapSnapshotMetadata {
                    x_name: &snapshot.x_name,
                    y_name: &snapshot.y_name,
                    series,
                },
                values,
            )
        }
    }
}

/// Encodes a live chart update into the same `FCHT` frame family used for
/// snapshots, with payload kind distinguishing reset and append variants.
pub(crate) fn encode_live_chart_data(response: &LiveChartDataResponse) -> anyhow::Result<Vec<u8>> {
    match response {
        LiveChartDataResponse::Reset {
            row_count,
            snapshot: ChartSnapshot::Xy(snapshot),
        } => {
            let mut values = Vec::with_capacity(snapshot.series.len());
            let series = snapshot
                .series
                .iter()
                .map(|series| {
                    let meta = encode_xy_series_metadata(series)?;
                    values.push(series.values.as_slice());
                    Ok(meta)
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            encode_frame(
                PayloadKind::LiveResetXy,
                &LiveResetMetadata {
                    row_count: *row_count,
                    snapshot: XySnapshotMetadata {
                        plot_mode: snapshot.plot_mode,
                        draw_style: snapshot.draw_style,
                        x_name: &snapshot.x_name,
                        y_name: snapshot.y_name.as_deref(),
                        series,
                    },
                },
                values,
            )
        }
        LiveChartDataResponse::Reset {
            row_count,
            snapshot: ChartSnapshot::Heatmap(snapshot),
        } => {
            let mut values = Vec::with_capacity(snapshot.series.len());
            let series = snapshot
                .series
                .iter()
                .map(|series| {
                    let meta = encode_xyz_series_metadata(series)?;
                    values.push(series.values.as_slice());
                    Ok(meta)
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            encode_frame(
                PayloadKind::LiveResetHeatmap,
                &LiveResetMetadata {
                    row_count: *row_count,
                    snapshot: HeatmapSnapshotMetadata {
                        x_name: &snapshot.x_name,
                        y_name: &snapshot.y_name,
                        series,
                    },
                },
                values,
            )
        }
        LiveChartDataResponse::Append { row_count, ops } => {
            let mut value_blocks = Vec::with_capacity(ops.len());
            let ops = ops
                .iter()
                .map(|operation| {
                    let metadata = encode_live_append_metadata(operation)?;
                    match operation {
                        LiveChartAppendOperation::AppendPoints { values, .. } => {
                            value_blocks.push(values.as_slice());
                        }
                        LiveChartAppendOperation::AppendSeries { series } => match series {
                            FlatSeries::Xy(series) => value_blocks.push(series.values.as_slice()),
                            FlatSeries::Xyz(series) => value_blocks.push(series.values.as_slice()),
                        },
                    }
                    Ok(metadata)
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            encode_frame(
                PayloadKind::LiveAppend,
                &LiveAppendMetadata {
                    row_count: *row_count,
                    ops,
                },
                value_blocks,
            )
        }
    }
}

fn encode_frame(
    payload_kind: PayloadKind,
    metadata: &impl Serialize,
    value_blocks: Vec<&[f64]>,
) -> anyhow::Result<Vec<u8>> {
    let metadata =
        serde_json::to_vec(metadata).context("Failed to serialize chart wire metadata")?;
    let metadata_len = u32::try_from(metadata.len()).context("Chart wire metadata too large")?;
    let metadata_end = HEADER_LENGTH
        .checked_add(metadata.len())
        .context("Chart wire metadata too large")?;
    let numeric_offset = align_up(metadata_end, 8)?;
    let numeric_offset_u32 =
        u32::try_from(numeric_offset).context("Chart wire numeric payload offset too large")?;
    let padding_len = numeric_offset
        .checked_sub(metadata_end)
        .context("Chart wire padding underflow")?;
    let value_bytes_len = value_blocks
        .iter()
        .try_fold(0usize, |acc, block| {
            acc.checked_add(block.len().checked_mul(8)?)
        })
        .context("Chart wire numeric payload too large")?;

    let mut bytes = Vec::with_capacity(numeric_offset + value_bytes_len);
    bytes.extend_from_slice(MAGIC);
    bytes.push(VERSION);
    bytes.push(payload_kind.as_byte());
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(&metadata_len.to_le_bytes());
    bytes.extend_from_slice(&numeric_offset_u32.to_le_bytes());
    bytes.extend_from_slice(&metadata);
    bytes.extend(std::iter::repeat_n(0, padding_len));
    for block in value_blocks {
        for value in block {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
    }
    Ok(bytes)
}

fn align_up(value: usize, alignment: usize) -> anyhow::Result<usize> {
    let remainder = value % alignment;
    if remainder == 0 {
        Ok(value)
    } else {
        value
            .checked_add(alignment - remainder)
            .context("Chart wire payload offset overflowed")
    }
}

fn encode_xy_series_metadata(series: &FlatXYSeries) -> anyhow::Result<SeriesMetadata<'_>> {
    validate_value_count(series.values.len(), series.point_count, SeriesShape::Xy)
        .with_context(|| format!("XY series '{}' has invalid value count", series.id))?;
    Ok(SeriesMetadata {
        id: &series.id,
        label: &series.label,
        point_count: series.point_count,
        value_count: series.values.len(),
    })
}

fn encode_xyz_series_metadata(series: &FlatXYZSeries) -> anyhow::Result<SeriesMetadata<'_>> {
    validate_value_count(series.values.len(), series.point_count, SeriesShape::Xyz)
        .with_context(|| format!("XYZ series '{}' has invalid value count", series.id))?;
    Ok(SeriesMetadata {
        id: &series.id,
        label: &series.label,
        point_count: series.point_count,
        value_count: series.values.len(),
    })
}

fn encode_live_append_metadata(
    operation: &LiveChartAppendOperation,
) -> anyhow::Result<LiveAppendOperationMetadata<'_>> {
    match operation {
        LiveChartAppendOperation::AppendPoints {
            series_id,
            values,
            point_count,
        } => {
            let shape = infer_shape(values.len(), *point_count).with_context(|| {
                format!("Live append points for series '{series_id}' have invalid value count")
            })?;
            Ok(LiveAppendOperationMetadata::AppendPoints {
                series_id,
                shape,
                point_count: *point_count,
                value_count: values.len(),
            })
        }
        LiveChartAppendOperation::AppendSeries { series } => match series {
            FlatSeries::Xy(series) => {
                let metadata = encode_xy_series_metadata(series)?;
                Ok(LiveAppendOperationMetadata::AppendSeries {
                    shape: SeriesShape::Xy,
                    id: metadata.id,
                    label: metadata.label,
                    point_count: metadata.point_count,
                    value_count: metadata.value_count,
                })
            }
            FlatSeries::Xyz(series) => {
                let metadata = encode_xyz_series_metadata(series)?;
                Ok(LiveAppendOperationMetadata::AppendSeries {
                    shape: SeriesShape::Xyz,
                    id: metadata.id,
                    label: metadata.label,
                    point_count: metadata.point_count,
                    value_count: metadata.value_count,
                })
            }
        },
    }
}

fn infer_shape(value_count: usize, point_count: usize) -> anyhow::Result<SeriesShape> {
    ensure!(
        point_count > 0,
        "Point count must be greater than zero for append payloads"
    );
    if value_count == point_count.saturating_mul(2) {
        Ok(SeriesShape::Xy)
    } else if value_count == point_count.saturating_mul(3) {
        Ok(SeriesShape::Xyz)
    } else {
        bail!("Value count does not match XY or XYZ point layout");
    }
}

fn validate_value_count(
    value_count: usize,
    point_count: usize,
    shape: SeriesShape,
) -> anyhow::Result<()> {
    let expected = point_count
        .checked_mul(match shape {
            SeriesShape::Xy => 2,
            SeriesShape::Xyz => 3,
        })
        .context("Point count overflowed while validating chart wire payload")?;
    ensure!(
        value_count == expected,
        "Expected {expected} values for {point_count} points, got {value_count}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{MAGIC, VERSION, encode_chart_snapshot, encode_live_chart_data};
    use crate::features::charts::types::{
        ChartSnapshot, FlatSeries, FlatXYSeries, FlatXYZSeries, LiveChartAppendOperation,
        LiveChartDataResponse, XYChartSnapshot,
    };

    fn parse_metadata(bytes: &[u8]) -> Value {
        let metadata_len =
            u32::from_le_bytes(bytes[8..12].try_into().expect("metadata length")) as usize;
        serde_json::from_slice(&bytes[16..16 + metadata_len]).expect("metadata json")
    }

    fn numeric_payload(bytes: &[u8]) -> &[u8] {
        let numeric_offset =
            u32::from_le_bytes(bytes[12..16].try_into().expect("numeric offset")) as usize;
        &bytes[numeric_offset..]
    }

    #[test]
    fn encodes_xy_snapshot_frame() {
        let bytes = encode_chart_snapshot(&ChartSnapshot::Xy(XYChartSnapshot {
            plot_mode: crate::features::charts::types::XYPlotMode::Xy,
            draw_style: crate::features::charts::types::XYDrawStyle::Points,
            x_name: "time".to_string(),
            y_name: Some("value".to_string()),
            series: vec![FlatXYSeries::new(
                "signal",
                "signal",
                vec![1.0, 2.0, 3.0, 4.0],
                2,
            )],
        }))
        .expect("frame");

        assert_eq!(&bytes[..4], MAGIC);
        assert_eq!(bytes[4], VERSION);
        assert_eq!(bytes[5], 1);

        let metadata = parse_metadata(&bytes);
        assert_eq!(metadata["plotMode"], "xy");
        assert_eq!(metadata["drawStyle"], "points");
        assert_eq!(metadata["series"][0]["valueCount"], 4);
        assert_eq!(
            u32::from_le_bytes(bytes[12..16].try_into().expect("numeric offset")) as usize % 8,
            0
        );
        assert_eq!(numeric_payload(&bytes).len(), 4 * 8);
    }

    #[test]
    fn encodes_heatmap_snapshot_frame() {
        let bytes = encode_chart_snapshot(&ChartSnapshot::Heatmap(
            crate::features::charts::types::HeatmapChartSnapshot {
                x_name: "x".to_string(),
                y_name: "y".to_string(),
                series: vec![FlatXYZSeries::new(
                    "heat",
                    "heat",
                    vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
                    2,
                )],
            },
        ))
        .expect("frame");

        assert_eq!(bytes[5], 2);
        let metadata = parse_metadata(&bytes);
        assert_eq!(metadata["series"][0]["pointCount"], 2);
        assert_eq!(metadata["series"][0]["valueCount"], 6);
        assert_eq!(numeric_payload(&bytes).len(), 6 * 8);
    }

    #[test]
    fn encodes_live_reset_and_append_frames() {
        let reset = encode_live_chart_data(&LiveChartDataResponse::Reset {
            row_count: 5,
            snapshot: ChartSnapshot::Xy(XYChartSnapshot {
                plot_mode: crate::features::charts::types::XYPlotMode::QuantityVsSweep,
                draw_style: crate::features::charts::types::XYDrawStyle::Line,
                x_name: "step".to_string(),
                y_name: None,
                series: vec![FlatXYSeries::new("signal", "signal", vec![0.0, 1.0], 1)],
            }),
        })
        .expect("reset frame");
        assert_eq!(reset[5], 3);
        assert_eq!(parse_metadata(&reset)["rowCount"], 5);

        let append = encode_live_chart_data(&LiveChartDataResponse::Append {
            row_count: 6,
            ops: vec![
                LiveChartAppendOperation::AppendPoints {
                    series_id: "signal".to_string(),
                    values: vec![2.0, 3.0],
                    point_count: 1,
                },
                LiveChartAppendOperation::AppendSeries {
                    series: FlatSeries::Xyz(FlatXYZSeries::new(
                        "heat",
                        "heat",
                        vec![0.0, 1.0, 2.0],
                        1,
                    )),
                },
            ],
        })
        .expect("append frame");
        assert_eq!(append[5], 5);
        let metadata = parse_metadata(&append);
        assert_eq!(metadata["rowCount"], 6);
        assert_eq!(metadata["ops"][0]["kind"], "append_points");
        assert_eq!(metadata["ops"][1]["shape"], "xyz");
    }

    #[test]
    fn rejects_invalid_series_lengths() {
        let error = encode_chart_snapshot(&ChartSnapshot::Xy(XYChartSnapshot {
            plot_mode: crate::features::charts::types::XYPlotMode::Xy,
            draw_style: crate::features::charts::types::XYDrawStyle::Points,
            x_name: "x".to_string(),
            y_name: Some("y".to_string()),
            series: vec![FlatXYSeries::new(
                "signal",
                "signal",
                vec![1.0, 2.0, 3.0],
                2,
            )],
        }))
        .expect_err("invalid frame");

        assert!(error.to_string().contains("invalid value count"));
    }

    #[test]
    fn rejects_invalid_append_point_lengths() {
        let error = encode_live_chart_data(&LiveChartDataResponse::Append {
            row_count: 1,
            ops: vec![LiveChartAppendOperation::AppendPoints {
                series_id: "signal".to_string(),
                values: vec![1.0, 2.0, 3.0, 4.0],
                point_count: 1,
            }],
        })
        .expect_err("invalid append");

        assert!(error.to_string().contains("invalid value count"));
    }
}
