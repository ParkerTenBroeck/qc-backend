use ::serde::{Deserialize, Serialize};
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::ToSql,
    sql_types::Text,
    sqlite::Sqlite,
};
use serde_json::Value;


#[derive(Debug, Default, Clone, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[serde(crate = "rocket::serde")]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct JsonText(pub Value);


impl ToSql<Text, Sqlite> for JsonText {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        let val = serde_json::to_string(&self.0)?;
        out.set_value(val);
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for JsonText {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let val = <String as diesel::deserialize::FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(Self(serde_json::from_str(val.as_str())?))
    }
}
