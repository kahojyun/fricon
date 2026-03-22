use anyhow::{Context, bail};
use chrono::DateTime;

use crate::{
    dataset::model::{DatasetMetadata, DatasetRecord, DatasetStatus},
    proto::{self},
};

impl From<DatasetRecord> for proto::Dataset {
    fn from(record: DatasetRecord) -> Self {
        Self {
            id: record.id,
            metadata: Some(record.metadata.into()),
        }
    }
}

impl TryFrom<proto::Dataset> for DatasetRecord {
    type Error = anyhow::Error;

    fn try_from(dataset: proto::Dataset) -> Result<Self, Self::Error> {
        Ok(Self {
            id: dataset.id,
            metadata: dataset
                .metadata
                .context("metadata field is required")?
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
    type Error = anyhow::Error;

    fn try_from(metadata: proto::DatasetMetadata) -> Result<Self, Self::Error> {
        let uid = metadata.uid.parse()?;
        let created_at = metadata
            .created_at
            .map(from_proto_timestamp)
            .transpose()?
            .context("created_at is required")?;
        let proto_status =
            proto::DatasetStatus::try_from(metadata.status).context("Invalid dataset status")?;
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
) -> Result<DateTime<chrono::Utc>, anyhow::Error> {
    #[expect(
        clippy::cast_sign_loss,
        reason = "Negative values are explicitly checked and rejected above"
    )]
    let nanos = if value.nanos < 0 {
        bail!("invalid timestamp")
    } else {
        value.nanos as u32
    };
    DateTime::from_timestamp(value.seconds, nanos).context("invalid timestamp")
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
    type Error = anyhow::Error;

    fn try_from(status: proto::DatasetStatus) -> Result<Self, Self::Error> {
        match status {
            proto::DatasetStatus::Unspecified => bail!("Cannot convert unspecified dataset status"),
            proto::DatasetStatus::Writing => Ok(DatasetStatus::Writing),
            proto::DatasetStatus::Completed => Ok(DatasetStatus::Completed),
            proto::DatasetStatus::Aborted => Ok(DatasetStatus::Aborted),
        }
    }
}
