use std::collections::HashMap;

use ::serde::{Deserialize, Serialize};
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::ToSql,
    sql_types::Text,
    sqlite::Sqlite,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[serde(crate = "rocket::serde")]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct QCChecklist(pub HashMap<String, u8>);

impl QCChecklist {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl ToSql<Text, Sqlite> for QCChecklist {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        let val = serde_json::to_string(&self.0)?;
        out.set_value(val);
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for QCChecklist {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let val = <String as diesel::deserialize::FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(Self(serde_json::from_str(val.as_str())?))
    }
}
