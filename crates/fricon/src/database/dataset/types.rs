use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};
use uuid::Uuid;

use crate::dataset::model::DatasetStatus;

#[derive(Debug, Clone, Copy, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub(super) struct DbDatasetStatus(pub(super) DatasetStatus);

impl From<DatasetStatus> for DbDatasetStatus {
    fn from(value: DatasetStatus) -> Self {
        Self(value)
    }
}

impl ToSql<Text, Sqlite> for DbDatasetStatus
where
    String: ToSql<Text, Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let status_str = match self.0 {
            DatasetStatus::Writing => "writing",
            DatasetStatus::Completed => "completed",
            DatasetStatus::Aborted => "aborted",
        };
        out.set_value(status_str.to_string());
        Ok(serialize::IsNull::No)
    }
}

impl<DB> FromSql<Text, DB> for DbDatasetStatus
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let string = String::from_sql(bytes)?;
        let status = match string.as_str() {
            "writing" => DatasetStatus::Writing,
            "completed" => DatasetStatus::Completed,
            "aborted" => DatasetStatus::Aborted,
            _ => return Err(format!("Unknown dataset status: {string}").into()),
        };
        Ok(Self(status))
    }
}

#[derive(Debug, Clone, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub(super) struct SimpleUuid(pub(super) Uuid);

impl ToSql<Text, Sqlite> for SimpleUuid
where
    String: ToSql<Text, Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let uuid = self.0.simple().to_string();
        out.set_value(uuid);
        Ok(serialize::IsNull::No)
    }
}

impl<DB> FromSql<Text, DB> for SimpleUuid
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let string = String::from_sql(bytes)?;
        let uuid = string.parse()?;
        Ok(SimpleUuid(uuid))
    }
}
