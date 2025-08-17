use std::fmt;

use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[derive(Debug, FromSqlRow, AsExpression)]
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

#[derive(Debug, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub struct JsonValue<T>(pub T);

impl<T: Serialize + fmt::Debug> ToSql<Text, Sqlite> for JsonValue<T>
where
    String: ToSql<Text, Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let json = serde_json::to_string(&self.0)?;
        out.set_value(json);
        Ok(serialize::IsNull::No)
    }
}

impl<T: DeserializeOwned, DB> FromSql<Text, DB> for JsonValue<T>
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let string = String::from_sql(bytes)?;
        let value = serde_json::from_str(&string)?;
        Ok(JsonValue(value))
    }
}
