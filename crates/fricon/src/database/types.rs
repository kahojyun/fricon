use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum DatasetStatus {
    Writing,
    Completed,
    Aborted,
}

impl ToSql<Text, Sqlite> for DatasetStatus
where
    String: ToSql<Text, Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let status_str = match self {
            DatasetStatus::Writing => "writing",
            DatasetStatus::Completed => "completed",
            DatasetStatus::Aborted => "aborted",
        };
        out.set_value(status_str.to_string());
        Ok(serialize::IsNull::No)
    }
}

impl<DB> FromSql<Text, DB> for DatasetStatus
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let string = String::from_sql(bytes)?;
        match string.as_str() {
            "writing" => Ok(DatasetStatus::Writing),
            "completed" => Ok(DatasetStatus::Completed),
            "aborted" => Ok(DatasetStatus::Aborted),
            _ => Err(format!("Unknown dataset status: {string}").into()),
        }
    }
}

#[derive(Debug, Clone, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub struct SimpleUuid(pub Uuid);

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
