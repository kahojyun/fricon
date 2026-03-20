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
        use prost_types::Timestamp;

        let created_at = Timestamp {
            seconds: metadata.created_at.timestamp(),
            #[expect(
                clippy::cast_possible_wrap,
                reason = "Nanos are always less than 2e9 and within i32 range"
            )]
            nanos: metadata.created_at.timestamp_subsec_nanos() as i32,
        };
        Self {
            uid: metadata.uid.simple().to_string(),
            name: metadata.name,
            description: metadata.description,
            favorite: metadata.favorite,
            created_at: Some(created_at),
            tags: metadata.tags,
            status: proto::DatasetStatus::from(metadata.status) as i32,
        }
    }
}

impl TryFrom<proto::DatasetMetadata> for DatasetMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: proto::DatasetMetadata) -> Result<Self, Self::Error> {
        let uid = metadata.uid.parse()?;
        let created_at = metadata.created_at.context("created_at is required")?;
        let seconds = created_at.seconds;
        #[expect(
            clippy::cast_sign_loss,
            reason = "Negative values are explicitly checked and rejected above"
        )]
        let nanos = if created_at.nanos < 0 {
            bail!("invalid created_at")
        } else {
            created_at.nanos as u32
        };
        let created_at = DateTime::from_timestamp(seconds, nanos).context("invalid created_at")?;
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
            trashed_at: None,
            tags: metadata.tags,
        })
    }
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
