use ::serde::{Deserialize, Serialize};
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::ToSql,
    sql_types::TimestamptzSqlite,
    sqlite::Sqlite,
};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[serde(crate = "rocket::serde")]
#[diesel(sql_type = diesel::sql_types::TimestamptzSqlite)]
pub struct Time(#[serde(with = "time::serde::iso8601")] pub time::OffsetDateTime);

impl ToSql<diesel::sql_types::TimestamptzSqlite, Sqlite> for Time {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        <OffsetDateTime as diesel::serialize::ToSql<TimestamptzSqlite, Sqlite>>::to_sql(
            &self.0, out,
        )
    }
}

impl FromSql<TimestamptzSqlite, Sqlite> for Time {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Ok(Self(time::OffsetDateTime::from_sql(bytes)?))
    }
}
