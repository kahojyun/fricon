use chrono::DateTime;
use thiserror::Error;

use crate::{
    dataset::model::{DatasetMetadata, DatasetRecord, DatasetStatus},
    proto::{self},
};

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    #[error("Invalid dataset status: {0:?}")]
    InvalidStatus(proto::DatasetStatus),
    #[error("Invalid dataset status value: {0}")]
    InvalidStatusValue(i32),
    #[error("Cannot convert unspecified dataset status")]
    UnspecifiedStatus,
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
}

impl From<DatasetRecord> for proto::Dataset {
    fn from(record: DatasetRecord) -> Self {
        Self {
            id: record.id,
            metadata: Some(record.metadata.into()),
        }
    }
}

impl TryFrom<proto::Dataset> for DatasetRecord {
    type Error = CodecError;

    fn try_from(dataset: proto::Dataset) -> Result<Self, Self::Error> {
        Ok(Self {
            id: dataset.id,
            metadata: dataset
                .metadata
                .ok_or(CodecError::MissingField("metadata"))?
                .try_into()?,
        })
    }
}

impl From<DatasetMetadata> for proto::DatasetMetadata {
    fn from(metadata: DatasetMetadata) -> Self {
        let created_at = to_proto_timestamp(metadata.created_at);
        Self {
            uid: metadata.uid.simple().to_string(),
            name: metadata.name,
            description: metadata.description,
            favorite: metadata.favorite,
            created_at: Some(created_at),
            tags: metadata.tags,
            status: proto::DatasetStatus::from(metadata.status) as i32,
            trashed_at: metadata.trashed_at.map(to_proto_timestamp),
            deleted_at: metadata.deleted_at.map(to_proto_timestamp),
        }
    }
}

impl TryFrom<proto::DatasetMetadata> for DatasetMetadata {
    type Error = CodecError;

    fn try_from(metadata: proto::DatasetMetadata) -> Result<Self, Self::Error> {
        let uid = metadata.uid.parse()?;
        let created_at = metadata
            .created_at
            .map(from_proto_timestamp)
            .transpose()?
            .ok_or(CodecError::MissingField("created_at"))?;
        let proto_status = proto::DatasetStatus::try_from(metadata.status)
            .map_err(|_| CodecError::InvalidStatusValue(metadata.status))?;
        let status = DatasetStatus::try_from(proto_status)?;

        Ok(Self {
            uid,
            name: metadata.name,
            description: metadata.description,
            favorite: metadata.favorite,
            status,
            created_at,
            trashed_at: metadata.trashed_at.map(from_proto_timestamp).transpose()?,
            deleted_at: metadata.deleted_at.map(from_proto_timestamp).transpose()?,
            tags: metadata.tags,
        })
    }
}

fn to_proto_timestamp(value: DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: value.timestamp(),
        #[expect(
            clippy::cast_possible_wrap,
            reason = "Nanos are always less than 2e9 and within i32 range"
        )]
        nanos: value.timestamp_subsec_nanos() as i32,
    }
}

fn from_proto_timestamp(
    value: prost_types::Timestamp,
) -> Result<DateTime<chrono::Utc>, CodecError> {
    #[expect(
        clippy::cast_sign_loss,
        reason = "Negative values are explicitly checked and rejected above"
    )]
    let nanos = if value.nanos < 0 {
        return Err(CodecError::InvalidTimestamp);
    } else {
        value.nanos as u32
    };
    DateTime::from_timestamp(value.seconds, nanos).ok_or(CodecError::InvalidTimestamp)
}

impl From<DatasetStatus> for proto::DatasetStatus {
    fn from(status: DatasetStatus) -> Self {
        match status {
            DatasetStatus::Writing => proto::DatasetStatus::Writing,
            DatasetStatus::Completed => proto::DatasetStatus::Completed,
            DatasetStatus::Aborted => proto::DatasetStatus::Aborted,
        }
    }
}

impl TryFrom<proto::DatasetStatus> for DatasetStatus {
    type Error = CodecError;

    fn try_from(status: proto::DatasetStatus) -> Result<Self, Self::Error> {
        match status {
            proto::DatasetStatus::Unspecified => Err(CodecError::UnspecifiedStatus),
            proto::DatasetStatus::Writing => Ok(DatasetStatus::Writing),
            proto::DatasetStatus::Completed => Ok(DatasetStatus::Completed),
            proto::DatasetStatus::Aborted => Ok(DatasetStatus::Aborted),
        }
    }
}
